use anyhow::Result;
use mq::client::MqClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. broker に接続
    let mut client = MqClient::connect("127.0.0.1:5555").await?;

    // 2. 送るメッセージを作る
    let payload = json!({
        "text": "買い物行く？",
        "targets": ["living", "my", "husband"]
    });

    // 3. publish（topic は自由）
    client
        .publish_json("prompts.broadcast", "msg-001", payload)
        .await?;

    println!("published!");

    Ok(())
}
