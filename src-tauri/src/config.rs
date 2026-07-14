use crate::app_path;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PasteMode {
    Copy,
    Paste,
    Both,
}

impl Default for PasteMode {
    fn default() -> Self {
        Self::Copy
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub shortcut: String,
    pub ai_model: String,
    pub transcription_model: String,
    pub ai_polish_prompt: String,
    pub launch_on_startup: bool,
    pub auto_polish: bool,
    #[serde(default)]
    pub paste_mode: PasteMode,
}

pub fn initialize_config() -> Result<(), Box<dyn std::error::Error>> {
    let path = app_path::config_path();

    if !path.exists() {
        let config = Config {
            shortcut: "Ctrl+Shift+Space".to_string(),
            ai_model: "None".to_string(),
            transcription_model: "None".to_string(),
            ai_polish_prompt: "Clean and polish the following transcription. Correct spelling, grammar, punctuation, and formatting. Remove filler words and accidental repetitions. Fix obvious transcription mistakes using context, but do not change the meaning or add new information. Keep the tone natural and preserve speaker labels and timestamps if they are present. Output only the polished transcript. \n Transcript: \n {{transcription}}".to_string(),
            launch_on_startup: false,
            auto_polish: false,
            paste_mode: PasteMode::default(),
        };

        let json = serde_json::to_string_pretty(&config)?;
        fs::write(path, json)?;
    }

    Ok(())
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let json = fs::read_to_string(app_path::config_path())?;
    Ok(serde_json::from_str(&json)?)
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write(app_path::config_path(), json)?;
    Ok(())
}

#[tauri::command]
pub fn load_config_cmd() -> Result<Config, String> {
    load_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_config_cmd(config: Config) -> Result<(), String> {
    save_config(&config).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
pub struct AppPaths {
    pub models_dir: String,
    pub db_path: String,
    pub config_path: String,
    pub whisper_bin: String,
}

#[tauri::command]
pub fn get_app_paths_cmd() -> Result<AppPaths, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("failed to get exe parent")?
        .to_path_buf();
    let cwd = std::env::current_dir().unwrap_or_default();

    let whisper_dir = [
        exe_dir.join("_up_").join("whisper"),
        exe_dir.join("whisper"),
        cwd.join("whisper"),
        cwd.join("..").join("whisper"),
    ]
    .into_iter()
    .find(|p| p.join("whisper-cli.exe").exists());

    Ok(AppPaths {
        models_dir: app_path::app_data_dir()
            .join("models")
            .to_string_lossy()
            .to_string(),
        db_path: app_path::db_path().to_string_lossy().to_string(),
        config_path: app_path::config_path().to_string_lossy().to_string(),
        whisper_bin: whisper_dir
            .map(|d| d.join("whisper-cli.exe").to_string_lossy().to_string())
            .unwrap_or_default(),
    })
}
