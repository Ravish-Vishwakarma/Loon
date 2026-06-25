use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, WebviewWindowBuilder,
};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
fn open_new_window(app: &tauri::AppHandle) {
    if app.get_webview_window("settings").is_some() {
        return;
    }
    tauri::WebviewWindowBuilder::new(
        app,
        "settings",
        tauri::WebviewUrl::App("settings.html".into()),
    )
    .title("Settings")
    .build()
    .unwrap();
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tauri::async_runtime::spawn(async {
                runtime::start_server(
                    "../models".to_string(),
                    "../whisper/whisper-cli".to_string(),
                )
                .await;
            });
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let new_i = MenuItem::with_id(app, "new", "New Window", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&quit_i, &new_i])?;
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "new" => open_new_window(&app.app_handle()),
                    _ => {
                        println!("Menu Not Found")
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
