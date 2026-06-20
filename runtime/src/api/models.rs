use axum::Json;
use futures_util::StreamExt;
use reqwest;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
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

pub async fn download_model(
    url: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
            "Downloaded: {:.2}% ({}/{})",
            percentage, downloaded, total_size
        );
    }

    file.flush().await?;

    Ok(())
}
