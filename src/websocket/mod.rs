pub mod handler;
pub mod state;

use axum::{
    extract::ws::{Message, WebSocket},
    extract::{State, WebSocketUpgrade},
    response::Response,
};
use std::sync::Arc;

use crate::websocket::state::WsState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<WsState>>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<WsState>) {
    // Dodaj klienta do stanu
    let client_id = state.add_client().await;

    // Powitanie
    let welcome = format!(
        "<{{\"type\":\"connected\",\"client_id\":\"{}\"}}>",
        client_id
    );
    let _ = socket.send(Message::Text(welcome)).await;

    // Odbieranie wiadomości
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Przetwórz wiadomość
                if let Err(e) =
                    handler::process_message(&client_id, &text, &state, &mut socket).await
                {
                    let error_msg = format!("<{{\"type\":\"error\",\"message\":\"{}\"}}>", e);
                    let _ = socket.send(Message::Text(error_msg)).await;
                }
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }

    // Usuń klienta przy rozłączeniu
    state.remove_client(&client_id).await;
}
