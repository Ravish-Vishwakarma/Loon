mod app_path;
mod audio;
mod cancel;
mod clipboard;
mod config;
mod db;
mod ollama;
mod proxy;
mod recorder;
mod shortcut;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

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
            let path = match app.path().app_local_data_dir() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[loon] failed to get app data directory: {e}");
                    return Ok(());
                }
            };

            app_path::initialize(path);

            // CANCEL TOKEN -------------------------------- //
            cancel::init();
            // ---------------------------------------------- //

            // CONFIG --------------------------------------- //
            if let Err(e) = config::initialize_config() {
                eprintln!("[loon] config init failed: {e}");
                return Ok(());
            }
            // ---------------------------------------------- //

            // DATABASE ------------------------------------- //
            if let Err(e) = db::initialize_database() {
                eprintln!("[loon] db init failed: {e}");
                return Ok(());
            }
            // ---------------------------------------------- //

            // SHORTCUT  ------------------------------------- //
            if let Err(e) = shortcut::initialize_shortcut(app.app_handle()) {
                eprintln!("[loon] shortcut init failed: {e}");
            }
            // ---------------------------------------------- //

            // RUNTIME -------------------------------------- //
            let models_dir = app_path::app_data_dir().join("models");

            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_default();
            let cwd = std::env::current_dir().unwrap_or_default();
            let resource_dir = app.path().resource_dir().unwrap_or_default();

            // On Windows NSIS, bundled resources land in a _up_ subdirectory.
            // The "resources" config copies the whisper/ directory contents
            // into the resource dir. Check all plausible locations.
            let candidates: Vec<std::path::PathBuf> = [
                Some(exe_dir.join("_up_").join("whisper")),  // NSIS updater layout
                Some(exe_dir.join("whisper")),                // production (directory preserved)
                Some(exe_dir.clone()),                         // production (flat fallback)
                Some(resource_dir.join("whisper")),            // Tauri bundled resource
                Some(resource_dir.clone()),
                Some(cwd.join("whisper")),                    // dev: CWD = project root
                Some(cwd.join("..").join("whisper")),         // dev: CWD = src-tauri
            ]
            .into_iter()
            .flatten()
            .collect();

            eprintln!("[loon] exe_dir: {}", exe_dir.display());
            eprintln!("[loon] cwd: {}", cwd.display());
            eprintln!("[loon] resource_dir: {}", resource_dir.display());
            for c in &candidates {
                eprintln!("[loon]   checking: {} (exists={})", c.display(), c.join("whisper-cli.exe").exists());
            }

            let whisper_bin = candidates
                .iter()
                .find(|p| p.join("whisper-cli.exe").exists())
                .map(|p| p.join("whisper-cli.exe"))
                .unwrap_or_else(|| {
                    let fallback = exe_dir.join("whisper").join("whisper-cli.exe");
                    eprintln!(
                        "[loon] WARNING: whisper-cli.exe not found in any candidate, falling back to {}",
                        fallback.display()
                    );
                    fallback
                });

            eprintln!("[loon] models_dir: {}", models_dir.display());
            eprintln!("[loon] whisper_bin: {}", whisper_bin.display());

            let _ = std::fs::create_dir_all(&models_dir);
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
            cancel::reset_cancel_pending_cmd,
            proxy::list_available_models,
            proxy::list_downloaded_models,
            proxy::download_model,
            proxy::delete_model,
            proxy::list_ollama_models,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
