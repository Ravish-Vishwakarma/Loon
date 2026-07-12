use crate::app_path;
use rusqlite::{params, Connection, Result};
use serde::Serialize;

#[derive(Serialize)]
pub struct Transcription {
    pub id: i64,
    pub transcription: String,
    pub ai: String,
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
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    Ok(())
}

pub fn insert_transcription(transcription: &str, ai: &str) -> Result<i64, String> {
    let conn = open_database().map_err(|err| err.to_string())?;

    conn.execute(
        "INSERT INTO transcriptions (transcription, ai) VALUES (?1, ?2)",
        params![transcription, ai],
    )
    .map_err(|err| err.to_string())?;

    Ok(conn.last_insert_rowid())
}

pub fn update_transcription_ai(id: i64, ai: &str) -> Result<(), String> {
    let conn = open_database().map_err(|err| err.to_string())?;

    conn.execute(
        "UPDATE transcriptions SET ai = ?1 WHERE id = ?2",
        params![ai, id],
    )
    .map_err(|err| err.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn read_transcriptions() -> Result<Vec<Transcription>, String> {
    let conn = open_database().map_err(|err| err.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, transcription, ai, created_at
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
                created_at: row.get(3)?,
            })
        })
        .map_err(|err| err.to_string())?;

    let mut transcriptions = Vec::new();

    for transcription in transcription_iter {
        transcriptions.push(transcription.map_err(|err| err.to_string())?);
    }

    Ok(transcriptions)
}

#[tauri::command]
pub fn clear_transcriptions() -> Result<(), String> {
    let conn = open_database().map_err(|err| err.to_string())?;
    conn.execute("DELETE FROM transcriptions", [])
        .map_err(|err| err.to_string())?;
    Ok(())
}
