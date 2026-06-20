use api::state::AppState;
use axum::{Router, routing::delete, routing::get, routing::post};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod api;
mod backends;

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        models_dir: "models".to_string(),
        whisper_binary: "..\\whisper\\whisper-cli".to_string(),
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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:15000")
        .await
        .unwrap();

    tracing::info!("Runtime server running on http://127.0.0.1:15000");

    axum::serve(listener, app).await.unwrap();
}
