use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use tracing::{debug, info};

use crate::state::{AppState, LogStreamEvent};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.log_sender.subscribe();

    info!("New WebSocket client connected to /api/v1/ws");

    // Send initial connected handshake event
    let welcome = LogStreamEvent {
        channel: "system".to_string(),
        level: "info".to_string(),
        plan_id: None,
        message: "Connected to AgentGuard Real-Time Log Stream (Axum WebSocket)".to_string(),
        timestamp: chrono::Utc::now(),
    };
    if let Ok(json) = serde_json::to_string(&welcome) {
        let _ = sender.send(Message::Text(json)).await;
    }

    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&event) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) | Err(_) => break,
                Ok(Message::Ping(_)) => {
                    debug!("Received ping from WebSocket client");
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    info!("WebSocket client disconnected from /api/v1/ws");
}
