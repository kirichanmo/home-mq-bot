use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    PubSub,
    Queue, // Day1では使わないけど、後で使うので残す
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Frame {
    Publish {
        topic: String,
        msg_id: String,
        payload: serde_json::Value,
    },
    Subscribe {
        topic: String,
        consumer: String,
        mode: Mode,
        group: Option<String>,
    },

    // server → client
    Delivery {
        topic: String,
        msg_id: String,
        payload: serde_json::Value,
    },
    Ok {
        request: String,
    },
    Error {
        message: String,
    },
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub mod client;
