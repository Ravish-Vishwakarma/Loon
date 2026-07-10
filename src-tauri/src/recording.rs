use std::time::{SystemTime, UNIX_EPOCH};

pub fn new_recording_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    recordings_dir().join(format!("{timestamp}.wav"))
}

pub fn start_recording() -> Result<(), String>;

pub fn stop_recording() -> Result<PathBuf, String>;

pub fn is_recording() -> bool;
