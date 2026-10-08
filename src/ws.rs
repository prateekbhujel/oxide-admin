//! Real-time WebSocket broadcasting system for OxideAdmin (Laravel Echo & Reverb parity)
//! Provides pub/sub channels, instant record update broadcasts, and Axum WebSocket handlers.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Event payload sent over WebSockets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BroadcastEvent {
    pub channel: String,
    pub event: String,
    pub data: serde_json::Value,
}

/// Thread-safe event broadcaster powered by Tokio async channels
#[derive(Clone)]
pub struct Broadcaster {
    sender: broadcast::Sender<BroadcastEvent>,
}

impl Broadcaster {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Broadcast an event to all connected WebSocket clients (Laravel `broadcast(new OrderCreated($order))`)
    pub fn broadcast(&self, channel: &str, event: &str, data: serde_json::Value) -> usize {
        let msg = BroadcastEvent {
            channel: channel.to_string(),
            event: event.to_string(),
            data,
        };
        self.sender.send(msg).unwrap_or(0)
    }

    /// Subscribe to real-time events
    pub fn subscribe(&self) -> broadcast::Receiver<BroadcastEvent> {
        self.sender.subscribe()
    }

    /// Get current active subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new(256)
    }
}

/// Axum WebSocket upgrade handler
pub async fn ws_handler(ws: WebSocketUpgrade, broadcaster: Arc<Broadcaster>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, broadcaster))
}

async fn handle_socket(mut socket: WebSocket, broadcaster: Arc<Broadcaster>) {
    let mut rx = broadcaster.subscribe();

    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(evt) => {
                        if let Ok(json_str) = serde_json::to_string(&evt) {
                            if socket.send(Message::Text(json_str)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            client_msg = socket.recv() => {
                match client_msg {
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
