pub mod handler;
pub mod state;

use axum::{
    extract::ws::{Message, WebSocket},
    extract::{FromRef, State, WebSocketUpgrade},
    response::Response,
    Extension,
};
use std::sync::Arc;

use crate::state::AppState;
use crate::tenant::{scope_request_context, TenantContext, TenantIdentity};
use crate::websocket::state::WsState;

/// Lets `ws_handler` extract just the WebSocket sub-state out of the
/// unified `AppState` the router is built with (Axum's substate pattern).
impl FromRef<AppState> for Arc<WsState> {
    fn from_ref(state: &AppState) -> Self {
        state.ws.clone()
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(identity): Extension<TenantIdentity>,
) -> Response {
    ws.on_upgrade(move |socket| async move {
        let context = TenantContext::carrier(identity.carrier_id);
        scope_request_context(Some(context), handle_socket(socket, state, identity)).await;
    })
}

async fn handle_socket(mut socket: WebSocket, state: AppState, identity: TenantIdentity) {
    // Dodaj klienta do stanu
    let user_id = identity.claims.as_ref().map(|claims| claims.sub);
    let client_id = state.ws.add_client(identity.carrier_id, user_id).await;

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
                    handler::process_message(&client_id, &text, &state, &identity, &mut socket)
                        .await
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
    state.ws.remove_client(&client_id).await;
}
