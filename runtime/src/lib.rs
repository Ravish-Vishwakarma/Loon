use api::state::AppState;
use axum::{Router, routing::delete, routing::get, routing::post};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub mod api;
pub mod backends;

pub async fn start_server(models_dir: String, whisper_binary: String) {
    let state = Arc::new(AppState {
        models_dir,
        whisper_binary,
    });

    let app = Router::new()
        .route("/v1/health", get(api::health::health_check))
        .route("/v1/models/available", get(api::models::available_models))
        .route("/v1/models/downloaded", get(api::models::downloaded_models))
        .route("/v1/models/download", post(api::models::download_model_by_id))
        .route("/v1/models/{model_id}", delete(api::models::delete_model_by_id))
        .route("/v1/transcribe", post(api::transcriptions::transcribe))
        .layer(CorsLayer::permissive())
        .with_state(state);

    for attempt in 1..=5 {
        match tokio::net::TcpListener::bind("127.0.0.1:15000").await {
            Ok(listener) => {
                eprintln!("[loon] runtime server running on http://127.0.0.1:15000");
                if let Err(e) = axum::serve(listener, app).await {
                    eprintln!("[loon] runtime server error: {e}");
                }
                return;
            }
            Err(e) => {
                eprintln!(
                    "[loon] failed to bind runtime server on port 15000 (attempt {attempt}/5): {e}"
                );
                if attempt < 5 {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }
    }
    eprintln!("[loon] runtime server failed to start after 5 attempts");
}
