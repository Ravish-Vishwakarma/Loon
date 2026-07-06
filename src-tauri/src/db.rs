use crate::app_path;
use rusqlite::{params, Connection, Result};
use serde::Serialize;

#[derive(Serialize)]
pub struct Transcription {
    pub id: i64,
    pub transcription: String,
    pub ai: String,
    pub audio: String,
    pub created_at: String,
}

pub fn open_database() -> Result<Connection> {
    Connection::open(app_path::db_path())
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
pub fn create_transcription(transcription: &str, ai: &str, audio: &str) -> Result<i64, String> {
    let conn = open_database().map_err(|err| err.to_string())?;

    conn.execute(
        "INSERT INTO transcriptions (transcription, ai, audio) VALUES (?1, ?2, ?3)",
        params![transcription, ai, audio],
    )
    .map_err(|err| err.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn read_transcriptions() -> Result<Vec<Transcription>, String> {
    let conn = open_database().map_err(|err| err.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, transcription, ai, audio, created_at
             FROM transcriptions
             ORDER BY created_at DESC",
        )
        .map_err(|err| err.to_string())?;

    let transcription_iter = stmt
        .query_map([], |row| {
            Ok(Transcription {
                id: row.get(0)?,
                transcription: row.get(1)?,
                ai: row.get(2)?,
                audio: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|err| err.to_string())?;

    let mut transcriptions = Vec::new();

    for transcription in transcription_iter {
        transcriptions.push(transcription.map_err(|err| err.to_string())?);
    }

    Ok(transcriptions)
}
