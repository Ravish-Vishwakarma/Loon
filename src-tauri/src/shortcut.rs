use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

pub fn initialize_shortcut(app: &AppHandle) -> Result<(), String> {
    let config = crate::config::load_config().map_err(|e| e.to_string())?;

    if config.shortcut == "None" {
        return Ok(());
    }

    register_shortcut(app, &config.shortcut)
}

pub fn register_shortcut(app: &AppHandle, shortcut: &str) -> Result<(), String> {
    let shortcut: Shortcut = shortcut
        .parse()
        .map_err(|_| format!("Invalid shortcut: {shortcut}"))?;
    app.global_shortcut()
        .register(shortcut)
        .map_err(|e| e.to_string())?;
    println!("Registered shortcut!");

    Ok(())
}

pub fn on_shortcut_pressed(app: &AppHandle, event: ShortcutEvent) {
    if event.state() != ShortcutState::Pressed {
        return;
    }

    if let Some(window) = app.get_webview_window("loon") {
        let visible = window.is_visible().unwrap_or(false);

        if visible {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
