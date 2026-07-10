mod app_path;
mod config;
mod db;
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
            tauri::async_runtime::spawn(async {
                runtime::start_server(
                    "../models".to_string(),
                    "../whisper/whisper-cli".to_string(),
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
            db::create_transcription,
            db::read_transcriptions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
