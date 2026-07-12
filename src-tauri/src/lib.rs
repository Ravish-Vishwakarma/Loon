mod app_path;
mod audio;
mod cancel;
mod clipboard;
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

            // CANCEL TOKEN -------------------------------- //
            cancel::init();
            // ---------------------------------------------- //

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
            let app_dir = app.path().app_local_data_dir().expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let models_dir = app_dir.join("models");

            let exe_dir = std::env::current_exe()
                .expect("failed to get exe path")
                .parent()
                .expect("failed to get exe parent")
                .to_path_buf();
            let cwd = std::env::current_dir().unwrap_or_default();

            let whisper_dir = [
                exe_dir.join("whisper"),          // production (directory preserved)
                exe_dir.clone(),                   // production (flat fallback)
                cwd.join("whisper"),              // dev: CWD = project root
                cwd.join("..").join("whisper"),   // dev: CWD = src-tauri
            ]
            .into_iter()
            .find(|p| p.join("whisper-cli.exe").exists())
            .expect("whisper-cli.exe not found next to exe, in CWD, or one level up");

            let whisper_bin = whisper_dir.join("whisper-cli.exe");

            println!("[loon] models_dir: {}", models_dir.display());
            println!("[loon] whisper_bin: {}", whisper_bin.display());

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

            // LAUNCHER POSITION ---------------------------- //
            if let Some(win) = app.get_webview_window("loon") {
                let win_w = 180.0_f64;
                let win_h = 36.0_f64;
                let (screen_w, screen_h) = win
                    .primary_monitor()
                    .ok()
                    .flatten()
                    .map(|m| {
                        let s = m.size();
                        (s.width as f64, s.height as f64)
                    })
                    .unwrap_or((1920.0, 1080.0));
                let x = (screen_w - win_w) / 2.0;
                let y = screen_h - win_h - 60.0;
                let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: x as i32,
                    y: y as i32,
                }));
            }
            // ---------------------------------------------- //

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::load_config_cmd,
            config::save_config_cmd,
            config::get_app_paths_cmd,
            db::read_transcriptions,
            db::clear_transcriptions,
            audio::process_audio,
            recorder::start_recording_cmd,
            recorder::stop_recording_cmd,
            recorder::is_recording_cmd,
            recorder::polish_cmd,
            recorder::hide_launcher_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
