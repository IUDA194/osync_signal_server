use std::sync::Arc;
use axum::{
    extract::{ws::WebSocketUpgrade, Path, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber;

mod room;
use room::{RoomMap, handle_socket};

#[tokio::main]
async fn main() {
    // Настроим логирование
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(false)
        .init();

    let rooms = Arc::new(RoomMap::new());

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ws/:room_id", get(ws_handler))
        .with_state(rooms.clone());

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind");

    info!("Server started at ws://0.0.0.0:3000/ws/:room_id and http://0.0.0.0:3000/health");
    axum::serve(listener, app).await.unwrap();
}

async fn health_handler() -> impl IntoResponse {
    info!("Health check requested");
    axum::Json(serde_json::json!({ "status": "OK" }))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(rooms): State<Arc<RoomMap>>,
) -> impl IntoResponse {
    info!("WebSocket upgrade for room: {}", room_id);
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, rooms))
}
