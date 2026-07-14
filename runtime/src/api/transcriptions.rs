use crate::api::models::Model;
use crate::api::state::AppState;
use crate::backends::backend::Backend;
use crate::backends::whisper_cpp::WhisperCppBackend;
use axum::{Json, extract::Multipart, extract::State};
use serde::Serialize;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct TranscribeResponse {
    pub model_id: String,
    pub text: String,
}

pub async fn transcribe(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Json<TranscribeResponse> {
    let mut model_id = String::new();
    let mut audio_bytes: Vec<u8> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "model_id" => model_id = field.text().await.unwrap_or_default(),
            "file" => audio_bytes = field.bytes().await.unwrap_or_default().to_vec(),
            _ => {}
        }
    }

    if model_id.is_empty() || audio_bytes.is_empty() {
        return Json(TranscribeResponse {
            model_id,
            text: "error: missing model_id or file".to_string(),
        });
    }

    let models: Vec<Model> = serde_json::from_str(include_str!("../models.json")).unwrap();

    let model = match models.iter().find(|m| m.id == model_id) {
        Some(m) => m,
        None => {
            return Json(TranscribeResponse {
                model_id,
                text: "error: model not found".to_string(),
            });
        }
    };

    let model_path = PathBuf::from(&state.models_dir)
        .join(&model.filename)
        .to_string_lossy()
        .to_string();

    let samples = match decode_wav(&audio_bytes) {
        Ok(s) => s,
        Err(e) => {
            return Json(TranscribeResponse {
                model_id,
                text: format!("error: audio decode failed: {}", e),
            });
        }
    };

    let backend = WhisperCppBackend::new(&model_path, &state.whisper_binary);

    match backend.transcribe(&samples) {
        Ok(text) => Json(TranscribeResponse { model_id, text }),
        Err(e) => Json(TranscribeResponse {
            model_id,
            text: format!("error: transcription failed: {}", e),
        }),
    }
}

fn decode_wav(bytes: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let cursor = Cursor::new(bytes);
    let mut reader = hound::WavReader::new(cursor)?;
    let spec = reader.spec();

    let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap_or(0)).collect();

    let samples_f32: Vec<f32> = samples.iter().map(|&s| s as f32 / 32768.0).collect();

    let samples_f32 = if spec.channels == 2 {
        samples_f32.chunks(2).map(|c| (c[0] + c[1]) * 0.5).collect()
    } else {
        samples_f32
    };

    if spec.sample_rate != 16000 {
        Ok(resample(&samples_f32, spec.sample_rate, 16000))
    } else {
        Ok(samples_f32)
    }
}

fn resample(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let ratio = to_rate as f64 / from_rate as f64;
    let output_len = (input.len() as f64 * ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 / ratio;
        let lo = src_idx.floor() as usize;
        let hi = (lo + 1).min(input.len() - 1);
        let frac = src_idx - lo as f64;
        let sample = input[lo] as f64 * (1.0 - frac) + input[hi] as f64 * frac;
        output.push(sample as f32);
    }

    output
}
