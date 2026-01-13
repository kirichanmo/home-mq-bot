use anyhow::Result;
use mq::{Frame, Mode};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{mpsc, RwLock},
};

type Tx = mpsc::Sender<Frame>;
type Topics = Arc<RwLock<HashMap<String, Vec<Tx>>>>;

#[tokio::main]
async fn main() -> Result<()> {
    let topics: Topics = Arc::new(RwLock::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:5555").await?;

    println!("broker (Day1) listening on 127.0.0.1:5555");

    loop {
        let (sock, _) = listener.accept().await?;
        let topics = topics.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_conn(sock, topics).await {
                eprintln!("connection error: {e}");
            }
        });
    }
}

async fn handle_conn(sock: TcpStream, topics: Topics) -> Result<()> {
    let (r, mut w) = sock.into_split();
    let mut lines = BufReader::new(r).lines();

    let (out_tx, mut out_rx) = mpsc::channel::<Frame>(256);

    // writer task
    tokio::spawn(async move {
        while let Some(frame) = out_rx.recv().await {
            if let Ok(s) = serde_json::to_string(&frame) {
                let _ = w.write_all(s.as_bytes()).await;
                let _ = w.write_all(b"\n").await;
            }
        }
    });

    while let Some(line) = lines.next_line().await? {
        let frame: Frame = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                let _ = out_tx
                    .send(Frame::Error {
                        message: "invalid json".into(),
                    })
                    .await;
                continue;
            }
        };

        match frame {
            Frame::Subscribe { topic, mode, .. } => {
                if mode != Mode::PubSub {
                    let _ = out_tx
                        .send(Frame::Error {
                            message: "Day1 broker supports pubsub only".into(),
                        })
                        .await;
                    continue;
                }

                topics
                    .write()
                    .await
                    .entry(topic.clone())
                    .or_default()
                    .push(out_tx.clone());

                let _ = out_tx
                    .send(Frame::Ok {
                        request: "subscribe".into(),
                    })
                    .await;
            }

            Frame::Publish {
                topic,
                msg_id,
                payload,
            } => {
                let subs = topics.read().await.get(&topic).cloned().unwrap_or_default();
                for tx in subs {
                    let _ = tx
                        .send(Frame::Delivery {
                            topic: topic.clone(),
                            msg_id: msg_id.clone(),
                            payload: payload.clone(),
                        })
                        .await;
                }

                let _ = out_tx
                    .send(Frame::Ok {
                        request: "publish".into(),
                    })
                    .await;
            }

            _ => {
                let _ = out_tx
                    .send(Frame::Error {
                        message: "unsupported frame in Day1".into(),
                    })
                    .await;
            }
        }
    }

    Ok(())
}
