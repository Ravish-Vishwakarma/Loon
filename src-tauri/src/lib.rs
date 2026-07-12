mod app_path;
mod audio;
mod config;
mod db;
mod ollama;
mod recorder;
mod shortcut;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_global_shortcut::ShortcutState;

fn open_setting_window(app: &tauri::AppHandle) {
    if app.get_webview_window("settings").is_some() {
        return;
    }
    tauri::WebviewWindowBuilder::new(app, "settings", tauri::WebviewUrl::App("/home".into()))
        .title("Settings")
        .build()
        .unwrap();
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _, event| shortcut::on_shortcut_pressed(app, event))
                .build(),
        )
        .setup(|app| {
            // APP PATH ------------------------------------- //
            let path = app
                .path()
                .app_local_data_dir()
                .expect("failed to get app data directory");

            app_path::initialize(path);

            // CONFIG --------------------------------------- //
            config::initialize_config()?;
            // ---------------------------------------------- //

            // DATABASE ------------------------------------- //
            db::initialize_database()?;
            // ---------------------------------------------- //

            // SHORTCUT  ------------------------------------- //
            shortcut::initialize_shortcut(app.app_handle())?;
            // ---------------------------------------------- //

            // RUNTIME -------------------------------------- //
            let app_dir = app.path().app_data_dir().expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let models_dir = app_dir.join("models");
            let whisper_dir = std::env::current_exe()
                .expect("failed to get exe path")
                .parent()
                .expect("failed to get exe parent")
                .join("whisper");
            let whisper_bin = whisper_dir.join("whisper-cli");
            std::fs::create_dir_all(&models_dir).expect("failed to create models dir");
            tauri::async_runtime::spawn(async move {
                runtime::start_server(
                    models_dir.to_string_lossy().to_string(),
                    whisper_bin.to_string_lossy().to_string(),
                )
                .await;
            });
            // ---------------------------------------------- //

            // SYSTEM TRAY ---------------------------------- //
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let new_i = MenuItem::with_id(app, "setting", "Setting", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&quit_i, &new_i])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "setting" => open_setting_window(&app.app_handle()),
                    _ => {
                        println!("Menu Not Found")
                    }
                })
                .tooltip("Loon")
                .build(app)?;
            // ---------------------------------------------- //

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::load_config_cmd,
            config::save_config_cmd,
            config::get_app_paths_cmd,
            db::read_transcriptions,
            audio::process_audio,
            recorder::start_recording_cmd,
            recorder::stop_recording_cmd,
            recorder::is_recording_cmd,
            recorder::polish_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
