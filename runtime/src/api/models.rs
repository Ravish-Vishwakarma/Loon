use axum::{Json, extract::Path, extract::State};
use futures_util::StreamExt;
use reqwest;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub backend: String,
    pub size: usize,
    pub filename: String,
    pub download_url: String,
    pub languages: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadRequest {
    pub model_id: String,
}

#[derive(Debug, Serialize)]
pub struct DownloadResponse {
    pub model_id: String,
    pub status: String,
    pub path: String,
}

pub async fn downloaded_models(
    State(models_dir): State<Arc<String>>,
) -> Json<Vec<Model>> {
    let models: Vec<Model> = serde_json::from_str(include_str!("../models.json")).unwrap();

    let mut downloaded = Vec::new();
    for model in models {
        let file_path = format!("{}\\{}", models_dir, model.filename);
        if fs::try_exists(&file_path).await.unwrap_or(false) {
            downloaded.push(model);
        }
    }

    Json(downloaded)
}

pub async fn available_models() -> Json<Vec<Model>> {
    let json_str = include_str!("../models.json");

    let models: Vec<Model> = serde_json::from_str(json_str).unwrap();

    Json(models)
}

pub async fn download_model_by_id(
    State(models_dir): State<Arc<String>>,
    Json(req): Json<DownloadRequest>,
) -> Json<DownloadResponse> {
    let models: Vec<Model> = serde_json::from_str(include_str!("../models.json")).unwrap();

    let model = match models.iter().find(|m| m.id == req.model_id) {
        Some(m) => m,
        None => {
            return Json(DownloadResponse {
                model_id: req.model_id,
                status: "error: model not found".to_string(),
                path: String::new(),
            });
        }
    };

    let output_path = format!("{}\\{}", models_dir, model.filename);

    match download_model(&model.download_url, &output_path).await {
        Ok(_) => Json(DownloadResponse {
            model_id: model.id.clone(),
            status: "downloaded".to_string(),
            path: output_path,
        }),
        Err(e) => Json(DownloadResponse {
            model_id: model.id.clone(),
            status: format!("error: {}", e),
            path: String::new(),
        }),
    }
}

pub async fn delete_model_by_id(
    State(models_dir): State<Arc<String>>,
    Path(model_id): Path<String>,
) -> Json<DownloadResponse> {
    let models: Vec<Model> = serde_json::from_str(include_str!("../models.json")).unwrap();

    let model = match models.iter().find(|m| m.id == model_id) {
        Some(m) => m,
        None => {
            return Json(DownloadResponse {
                model_id,
                status: "error: model not found".to_string(),
                path: String::new(),
            });
        }
    };

    let file_path = format!("{}\\{}", models_dir, model.filename);

    match fs::remove_file(&file_path).await {
        Ok(_) => Json(DownloadResponse {
            model_id,
            status: "deleted".to_string(),
            path: file_path,
        }),
        Err(e) => Json(DownloadResponse {
            model_id,
            status: format!("error: {}", e),
            path: file_path,
        }),
    }
}

async fn download_model(url: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    let total_size = response
        .content_length()
        .ok_or("Failed to get content length")?;

    let mut file = File::create(output_path).await?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        let percentage = (downloaded as f64 / total_size as f64) * 100.0;
        println!(
            "Downloading {}: {:.2}% ({}/{})",
            output_path, percentage, downloaded, total_size
        );
    }

    file.flush().await?;
    Ok(())
}
