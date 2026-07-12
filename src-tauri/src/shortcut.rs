use tauri::{AppHandle, Emitter, Manager};
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

        if crate::recorder::is_recording() {
            match crate::recorder::stop_recording() {
                Ok(wav_path) => {
                    let _ = window.emit("transcribing", ());
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let model_id = crate::config::load_config()
                            .map(|c| c.transcription_model)
                            .unwrap_or_else(|_| "ggml-base.en".into());

                        match crate::recorder::transcribe(&wav_path, &model_id).await {
                            Ok(text) => {
                                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                    let _ = clipboard.set_text(&text);
                                }
                                let _ = app.emit("transcription-done", text);
                            }
                            Err(e) => {
                                eprintln!("transcription failed: {e}");
                                let _ = app.emit("transcription-error", e);
                            }
                        }

                        let _ = std::fs::remove_file(&wav_path);
                    });
                }
                Err(e) => {
                    eprintln!("stop_recording failed: {e}");
                }
            }
            return;
        }

        if !visible {
            let _ = window.show();
            let _ = window.set_focus();
        }

        match crate::recorder::start_recording() {
            Ok(()) => {
                let _ = window.emit("start-recording", ());
            }
            Err(e) => {
                eprintln!("start_recording failed: {e}");
            }
        }
    }
}
