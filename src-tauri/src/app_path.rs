use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

static APP_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn initialize(path: PathBuf) {
    fs::create_dir_all(&path).expect("failed to create app data directory");

    APP_DATA_DIR
        .set(path)
        .expect("app data directory already initialized");
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

pub fn recordings_dir() -> PathBuf {
    let path = app_data_dir().join("recordings");

    std::fs::create_dir_all(&path).expect("failed to create recordings directory");

    path
}
