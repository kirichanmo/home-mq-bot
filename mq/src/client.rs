use crate::{Frame, Mode};
use anyhow::Context;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc,
};

pub struct MqClient {
    out: tokio::net::tcp::OwnedWriteHalf,
    in_rx: mpsc::Receiver<Frame>,
}

impl MqClient {
    pub async fn connect(addr: &str) -> anyhow::Result<Self> {
        let sock = TcpStream::connect(addr)
            .await
            .with_context(|| format!("connect to broker: {addr}"))?;
        let (r, w) = sock.into_split();

        let mut lines = BufReader::new(r).lines();
        let (tx, rx) = mpsc::channel::<Frame>(256);

        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(frame) = serde_json::from_str::<Frame>(&line) {
                    let _ = tx.send(frame).await;
                }
            }
        });

        Ok(Self { out: w, in_rx: rx })
    }

    pub async fn send(&mut self, frame: Frame) -> anyhow::Result<()> {
        let s = serde_json::to_string(&frame)?;
        self.out.write_all(s.as_bytes()).await?;
        self.out.write_all(b"\n").await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Option<Frame> {
        self.in_rx.recv().await
    }

    pub async fn subscribe_pubsub(&mut self, topic: &str, consumer: &str) -> anyhow::Result<()> {
        self.send(Frame::Subscribe {
            topic: topic.to_string(),
            consumer: consumer.to_string(),
            mode: Mode::PubSub,
            group: None,
        })
        .await
    }

    pub async fn publish_json(
        &mut self,
        topic: &str,
        msg_id: &str,
        payload: serde_json::Value,
    ) -> anyhow::Result<()> {
        self.send(Frame::Publish {
            topic: topic.to_string(),
            msg_id: msg_id.to_string(),
            payload,
        })
        .await
    }
}
