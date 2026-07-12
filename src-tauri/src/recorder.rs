use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use hound::{WavSpec, WavWriter};
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use std::time::Instant;

struct Recording {
    samples: Arc<Mutex<Vec<f32>>>,
    _stream: Stream,
    start: Instant,
}

static ACTIVE: once_cell::sync::OnceCell<Mutex<Option<Recording>>> =
    once_cell::sync::OnceCell::new();

fn active() -> &'static Mutex<Option<Recording>> {
    ACTIVE.get_or_init(|| Mutex::new(None))
}

fn default_input_device() -> Result<Device, String> {
    let host = cpal::default_host();
    host.default_input_device()
        .ok_or_else(|| "no input device available".into())
}

fn build_input_stream(device: &Device, samples: Arc<Mutex<Vec<f32>>>) -> Result<Stream, String> {
    let supported = device.default_input_config().map_err(|e| e.to_string())?;

    let sample_format = supported.sample_format();
    let config: StreamConfig = supported.into();

    let err_fn = |err: cpal::StreamError| eprintln!("input stream error: {err}");

    match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                samples.lock().unwrap().extend_from_slice(data);
            },
            err_fn,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                let mapped = data.iter().map(|&s| s as f32 / 32768.0);
                samples.lock().unwrap().extend(mapped);
            },
            err_fn,
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data: &[u16], _: &cpal::InputCallbackInfo| {
                let mapped = data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0);
                samples.lock().unwrap().extend(mapped);
            },
            err_fn,
            None,
        ),
        SampleFormat::I32 => device.build_input_stream(
            &config,
            move |data: &[i32], _: &cpal::InputCallbackInfo| {
                let mapped = data.iter().map(|&s| s as f32 / 2147483648.0);
                samples.lock().unwrap().extend(mapped);
            },
            err_fn,
            None,
        ),
        fmt => return Err(format!("unsupported sample format: {fmt:?}")),
    }
    .map_err(|e| e.to_string())
}

fn native_sample_rate(device: &Device) -> u32 {
    device
        .default_input_config()
        .ok()
        .map(|c| c.sample_rate().0)
        .unwrap_or(44100)
}

fn downsample(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to {
        return input.to_vec();
    }
    let ratio = to as f64 / from as f64;
    let out_len = (input.len() as f64 * ratio) as usize;
    (0..out_len)
        .map(|i| {
            let src = i as f64 / ratio;
            let lo = src.floor() as usize;
            let hi = (lo + 1).min(input.len().saturating_sub(1));
            let frac = src - lo as f64;
            input[lo] as f64 * (1.0 - frac) + input[hi] as f64 * frac
        })
        .map(|s| s as f32)
        .collect()
}

fn to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels == 1 {
        return samples.to_vec();
    }
    let ch = channels as usize;
    samples
        .chunks(ch)
        .map(|frame| frame.iter().sum::<f32>() / ch as f32)
        .collect()
}

pub fn start_recording() -> Result<(), String> {
    let mut lock = active().lock().map_err(|e| e.to_string())?;
    if lock.is_some() {
        return Err("already recording".into());
    }

    let device = default_input_device()?;
    let native_rate = native_sample_rate(&device);
    let channels = device
        .default_input_config()
        .ok()
        .map(|c| c.channels())
        .unwrap_or(1);

    let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let stream = build_input_stream(&device, Arc::clone(&samples))?;
    stream.play().map_err(|e| e.to_string())?;

    let rec = Recording {
        samples,
        _stream: stream,
        start: Instant::now(),
    };
    *lock = Some(rec);

    println!("recording started (native rate: {native_rate} Hz, channels: {channels})");
    Ok(())
}

pub fn stop_recording() -> Result<String, String> {
    let rec = {
        let mut lock = active().lock().map_err(|e| e.to_string())?;
        lock.take().ok_or("not recording")?
    };

    let elapsed = rec.start.elapsed();
    drop(rec._stream);
    let raw = Arc::try_unwrap(rec.samples)
        .map_err(|_| "failed to unwrap samples".to_string())?
        .into_inner()
        .map_err(|e| e.to_string())?;

    println!(
        "recording stopped ({:.1}s, {} raw samples)",
        elapsed.as_secs_f32(),
        raw.len()
    );

    let device = default_input_device()?;
    let native_rate = native_sample_rate(&device);
    let channels = device
        .default_input_config()
        .ok()
        .map(|c| c.channels())
        .unwrap_or(1);

    let mono = to_mono(&raw, channels);
    let samples_16k = downsample(&mono, native_rate, 16000);

    let peak = samples_16k.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let gain = if peak > 0.001 { 0.8 / peak } else { 1.0 };
    let normalized: Vec<f32> = samples_16k.iter().map(|&s| (s * gain).clamp(-1.0, 1.0)).collect();

    println!("audio peak: {peak:.4}, gain applied: {gain:.2}x");

    let mut path = std::env::temp_dir();
    let filename = format!("loon_recording_{}.wav", chrono_timestamp());
    path.push(&filename);

    let spec = WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    let buf = BufWriter::new(file);
    let mut writer = WavWriter::new(buf, spec).map_err(|e| e.to_string())?;

    for &s in &normalized {
        let amp = (s * 32768.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(amp).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;

    println!(
        "saved wav: {} ({} samples)",
        path.display(),
        normalized.len()
    );

    Ok(path.to_string_lossy().into_owned())
}

pub fn is_recording() -> bool {
    active().lock().map(|lock| lock.is_some()).unwrap_or(false)
}

fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{secs}")
}

// -- Tauri commands ---------------------------------------------------------- //

#[tauri::command]
pub fn start_recording_cmd() -> Result<(), String> {
    start_recording()
}

#[tauri::command]
pub fn stop_recording_cmd() -> Result<String, String> {
    stop_recording()
}

#[tauri::command]
pub fn is_recording_cmd() -> Result<bool, String> {
    Ok(is_recording())
}

// -- Transcription ----------------------------------------------------------- //

pub async fn transcribe(wav_path: &str, model_id: &str) -> Result<String, String> {
    let bytes = std::fs::read(wav_path).map_err(|e| e.to_string())?;
    let part = reqwest::multipart::Part::bytes(bytes).file_name(wav_path.to_string());

    let form = reqwest::multipart::Form::new()
        .text("model_id", model_id.to_string())
        .part("file", part);

    let client = reqwest::Client::new();
    let resp = client
        .post("http://127.0.0.1:15000/v1/transcribe")
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    body.get("text")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("unexpected response: {body}"))
}

#[tauri::command]
pub async fn polish_cmd(id: i64, text: String) -> Result<String, String> {
    let config = crate::config::load_config().map_err(|e| e.to_string())?;
    let paste_mode = config.paste_mode.clone();
    let polished = crate::ollama::polish(&text, &config.ai_model, &config.ai_polish_prompt).await?;
    let _ = crate::db::update_transcription_ai(id, &polished);
    crate::clipboard::apply_paste_mode(&polished, &paste_mode);
    Ok(polished)
}

#[tauri::command]
pub fn hide_launcher_cmd(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("loon") {
        win.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}
