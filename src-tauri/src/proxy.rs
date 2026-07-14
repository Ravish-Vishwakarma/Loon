use serde::{Deserialize, Serialize};

const RUNTIME_URL: &str = "http://127.0.0.1:15000";
const OLLAMA_URL: &str = "http://127.0.0.1:11434";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub backend: String,
    pub size: usize,
    pub filename: String,
    pub download_url: String,
    pub languages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadRequest {
    pub model_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadResponse {
    pub model_id: String,
    pub status: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub model: String,
    pub details: OllamaModelDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaModelDetails {
    pub parameter_size: String,
    pub quantization_level: String,
}

#[tauri::command]
pub async fn list_available_models() -> Result<Vec<Model>, String> {
    let resp = reqwest::get(format!("{RUNTIME_URL}/v1/models/available"))
        .await
        .map_err(|e| format!("runtime server unavailable: {e}"))?;
    resp.json().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_downloaded_models() -> Result<Vec<Model>, String> {
    let resp = reqwest::get(format!("{RUNTIME_URL}/v1/models/downloaded"))
        .await
        .map_err(|e| format!("runtime server unavailable: {e}"))?;
    resp.json().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_model(model_id: String) -> Result<DownloadResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{RUNTIME_URL}/v1/models/download"))
        .json(&DownloadRequest { model_id })
        .send()
        .await
        .map_err(|e| format!("runtime server unavailable: {e}"))?;
    resp.json().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_model(model_id: String) -> Result<DownloadResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .delete(format!("{RUNTIME_URL}/v1/models/{model_id}"))
        .send()
        .await
        .map_err(|e| format!("runtime server unavailable: {e}"))?;
    resp.json().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_ollama_models() -> Result<Vec<OllamaModel>, String> {
    let resp = reqwest::get(format!("{OLLAMA_URL}/api/tags"))
        .await
        .map_err(|e| format!("ollama not running: {e}"))?;
    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let models = data
        .get("models")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    Ok(models)
}
