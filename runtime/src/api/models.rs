use axum::Json;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::fs;
#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub backend: String,
    pub size: usize,
    pub filename: String,
    pub download_url: String,
    pub languages: Vec<String>,
}

pub async fn available_models() -> Json<Vec<Model>> {
    let json_str = include_str!("../models.json");

    let models: Vec<Model> = serde_json::from_str(json_str).unwrap();

    Json(models)
}

pub async fn download_model(url: &str) {}
