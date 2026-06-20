use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
}

pub async fn health_check() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "ok".to_string(),
    })
}
