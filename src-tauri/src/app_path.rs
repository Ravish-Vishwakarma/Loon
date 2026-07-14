use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

static APP_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn initialize(path: PathBuf) {
    let _ = std::fs::create_dir_all(&path);
    let _ = APP_DATA_DIR.set(path);
}

pub fn app_data_dir() -> &'static Path {
    APP_DATA_DIR
        .get()
        .expect("app data directory not initialized")
        .as_path()
}

pub fn db_path() -> PathBuf {
    app_data_dir().join("data.sqlite")
}

pub fn config_path() -> PathBuf {
    app_data_dir().join("config.json")
}
