use once_cell::sync::Lazy;
use regex::Regex;
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

static TIMESTAMP_LINE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*\[?\d{1,2}:\d{2}(:\d{2})?\]?\s*").unwrap()
});

fn strip_timestamps(text: &str) -> String {
    text.lines()
        .map(|line| TIMESTAMP_LINE.replace(line, "").trim().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn polish(text: &str, model: &str, prompt_template: &str) -> Result<String, String> {
    let clean = strip_timestamps(text);
    let prompt = prompt_template.replace("{{transcription}}", &clean);

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
