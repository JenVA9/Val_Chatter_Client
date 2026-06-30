use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMsg};

use crate::models::Message;

#[derive(Debug, Clone)]
pub enum WsEvent {
    Connected,
    Disconnected,
    NewMessage(Message),
    Error(String),
}

#[derive(Serialize)]
struct AuthPacket<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    token: &'a str,
    #[serde(rename = "threadId")]
    thread_id: i64,
}

#[derive(Serialize)]
struct JoinPacket {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(rename = "threadId")]
    thread_id: i64,
}

#[derive(Deserialize)]
struct ServerEnvelope {
    #[serde(rename = "type")]
    kind: String,
    message: Option<Message>,
}

pub async fn connect_and_listen(
    ws_url: String,
    token: String,
    thread_id: i64,
    tx: mpsc::SyncSender<WsEvent>,
) {
    let url = format!("{}/ws", ws_url);

    let ws_stream = match connect_async(&url).await {
        Ok((s, _)) => s,
        Err(e) => {
            let _ = tx.send(WsEvent::Error(e.to_string()));
            return;
        }
    };

    let _ = tx.send(WsEvent::Connected);
    let (mut write, mut read) = ws_stream.split();

    let auth = AuthPacket { kind: "auth", token: &token, thread_id };
    if let Ok(json) = serde_json::to_string(&auth) {
        let _ = write.send(WsMsg::Text(json)).await;
    }

    while let Some(item) = read.next().await {
        match item {
            Ok(WsMsg::Text(text)) => {
                if let Ok(env) = serde_json::from_str::<ServerEnvelope>(&text) {
                    if env.kind == "new_message" {
                        if let Some(msg) = env.message {
                            if tx.send(WsEvent::NewMessage(msg)).is_err() { break; }
                        }
                    }
                }
            }
            Ok(WsMsg::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    let _ = tx.send(WsEvent::Disconnected);
}

pub async fn switch_thread(
    ws_url: String,
    _token: String,
    thread_id: i64,
    tx: mpsc::SyncSender<WsEvent>,
) {
    let url = format!("{}/ws", ws_url);
    let ws_stream = match connect_async(&url).await {
        Ok((s, _)) => s,
        Err(e) => {
            let _ = tx.send(WsEvent::Error(e.to_string()));
            return;
        }
    };

    let _ = tx.send(WsEvent::Connected);
    let (mut write, mut read) = ws_stream.split();

    let join = JoinPacket { kind: "join", thread_id };
    if let Ok(json) = serde_json::to_string(&join) {
        let _ = write.send(WsMsg::Text(json)).await;
    }

    while let Some(item) = read.next().await {
        match item {
            Ok(WsMsg::Text(text)) => {
                if let Ok(env) = serde_json::from_str::<ServerEnvelope>(&text) {
                    if env.kind == "new_message" {
                        if let Some(msg) = env.message {
                            if tx.send(WsEvent::NewMessage(msg)).is_err() { break; }
                        }
                    }
                }
            }
            Ok(WsMsg::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    let _ = tx.send(WsEvent::Disconnected);
}
