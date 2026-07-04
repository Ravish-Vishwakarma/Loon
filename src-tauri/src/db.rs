use dirs::data_local_dir;
use rusqlite::{params, Connection, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct Transcription {
    pub id: i64,
    pub transcription: String,
    pub ai: String,
    pub audio: String,
    pub created_at: String,
}

pub fn db_path() -> PathBuf {
    let mut path = data_local_dir().expect("failed to get local app data directory");
    path.push("loon");
    fs::create_dir_all(&path).expect("failed to create app data directory");
    path.push("data.sqlite");
    path
}

pub fn open_database() -> Result<Connection> {
    Connection::open(db_path())
}

pub fn initialize_database() -> Result<()> {
    let conn = open_database()?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS transcriptions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            transcription TEXT NOT NULL,
            ai TEXT NOT NULL,
            audio TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

#[tauri::command]
pub fn create_transcription(transcription: &str, ai: &str, audio: &str) -> Result<i64> {
    let conn = open_database()?;
    conn.execute(
        "INSERT INTO transcriptions (transcription,ai,audio) VALUES (?1,?2,?3)",
        params![transcription, ai, audio],
    )?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn read_transcriptions() -> Result<Vec<Transcription>> {
    let conn = open_database()?;
    let mut stmt = conn
        .prepare("SELECT id, transcription, ai, audio, created_at FROM transcriptions ORDER BY created_at DESC")?;
    let transcription_iter = stmt.query_map([], |row| {
        Ok(Transcription {
            id: row.get(0)?,
            transcription: row.get(1)?,
            ai: row.get(2)?,
            audio: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;

    let mut transcriptions = Vec::new();
    for transcription in transcription_iter {
        transcriptions.push(transcription?);
    }
    Ok(transcriptions)
}
