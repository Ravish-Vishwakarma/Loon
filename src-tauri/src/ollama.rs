use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

pub async fn polish(text: &str, model: &str, prompt_template: &str) -> Result<String, String> {
    let prompt = prompt_template.replace("{{transcription}}", text);

    let body = GenerateRequest {
        model: model.to_string(),
        prompt,
        stream: false,
    };

    let client = reqwest::Client::new();
    let resp = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: GenerateResponse = resp.json().await.map_err(|e| e.to_string())?;

    Ok(data.response)
}
