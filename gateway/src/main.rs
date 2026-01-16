use axum::{
    extract::{Path, State},
    response::{
        sse::{Event, Sse},
        Html, IntoResponse,
    },
    routing::get,
    Router,
};
use futures::StreamExt;
use mq::{client::MqClient, Frame};
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // SSE用ブロードキャスト（全ブラウザに配る）
    let (tx, _) = broadcast::channel::<String>(100);

    // broker購読タスク（gatewayは broker の subscriber として動く）
    {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut client = MqClient::connect("127.0.0.1:5555")
                .await
                .expect("connect broker");

            client
                .subscribe_pubsub("prompts.broadcast", "gateway")
                .await
                .expect("subscribe");

            while let Some(frame) = client.recv().await {
                if let Frame::Delivery { payload, .. } = frame {
                    // payload(JSON)を文字列化してSSEに流す
                    let _ = tx.send(payload.to_string());
                }
            }
        });
    }

    let state = Arc::new(AppState { tx });

    let app = Router::new()
        .route("/device/:name", get(device_page))
        .route("/events", get(events))
        .with_state(state);

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    println!("gateway listening on http://{addr}");

    // axum 0.7 は axum::serve を使う
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn device_page(Path(name): Path<String>) -> impl IntoResponse {
    Html(format!(
        r#"<!doctype html>
<html>
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>device: {name}</title>
<style>
  body {{ font-family: system-ui, -apple-system, Segoe UI, sans-serif; margin: 24px; }}
  h1 {{ margin: 0 0 12px; }}
  .hint {{ color: #666; margin-bottom: 12px; }}
  ul {{ padding-left: 18px; }}
</style>
</head>
<body>
<h1>Device: {name}</h1>
<div class="hint">SSEで prompts.broadcast を受信して表示します</div>
<ul id="log"></ul>

<script>
const log = document.getElementById("log");
const es = new EventSource("/events");
es.onmessage = (e) => {{
  const li = document.createElement("li");
  li.textContent = e.data;
  log.appendChild(li);
}};
es.onerror = () => {{
  const li = document.createElement("li");
  li.textContent = "[SSE] disconnected (auto reconnect...)";
  log.appendChild(li);
}};
</script>
</body>
</html>
"#,
    ))
}

async fn events(
    State(state): State<Arc<AppState>>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();

    // broadcast receiver -> SSE stream
    let stream = BroadcastStream::new(rx).filter_map(|msg| async move {
        match msg {
            Ok(text) => Some(Ok(Event::default().data(text))),
            Err(_) => None,
        }
    });

    Sse::new(stream)
}
