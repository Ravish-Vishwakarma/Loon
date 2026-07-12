use std::{env, fs};

#[tauri::command]
pub fn process_audio(audio: Vec<u8>) -> Result<String, String> {
    let mut path = env::temp_dir();
    path.push("loon_recording.webm");

    fs::write(&path, audio).map_err(|e| e.to_string())?;

    println!("Saved recording to {:?}", path);

    // TODO:
    // 1. Call Whisper
    // 2. Delete the file
    // 3. Save to database

    Ok(path.to_string_lossy().into_owned())
}
