use crate::app_path;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub shortcut: String,
    pub ai_model: String,
    pub transcription_model: String,
    pub ai_polish_prompt: String,
    pub launch_on_startup: bool,
    pub auto_polish: bool,
}

pub fn initialize_config() -> Result<(), Box<dyn std::error::Error>> {
    let path = app_path::config_path();

    if !path.exists() {
        let config = Config {
            shortcut: "None".to_string(),
            ai_model: "None".to_string(),
            transcription_model: "None".to_string(),
            ai_polish_prompt: "None".to_string(),
            launch_on_startup: false,
            auto_polish: false,
        };

        let json = serde_json::to_string_pretty(&config)?;
        fs::write(path, json)?;
    }

    Ok(())
}
