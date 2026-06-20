use axum::{Router, routing::get, routing::post};
use tower_http::cors::{Any, CorsLayer};

mod api;
mod backends;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/v1/health", get(api::health::health_check))
        .route("/v1/models/available", get(api::models::available_models));
    // .route("/transcribe", post(api::transcriptions::transcribe))

    let listener = tokio::net::TcpListener::bind("127.0.0.1:15000")
        .await
        .unwrap();

    tracing::info!("Runtime server running on http://127.0.0.1:15000");

    axum::serve(listener, app).await.unwrap();
}
