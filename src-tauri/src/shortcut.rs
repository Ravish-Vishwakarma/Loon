use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

use crate::cancel;

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
                    cancel::reset();
                    let _ = window.emit("transcribing", ());
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let config = crate::config::load_config().ok();
                        let model_id = config
                            .as_ref()
                            .map(|c| c.transcription_model.clone())
                            .unwrap_or_else(|| "ggml-base.en".into());

                        match crate::recorder::transcribe(&wav_path, &model_id).await {
                            Ok(text) => {
                                if cancel::is_cancelled() {
                                    let _ = app.emit("transcription-cancelled", ());
                                    let _ = std::fs::remove_file(&wav_path);
                                    return;
                                }

                                let id = crate::db::insert_transcription(&text, "").unwrap_or(0);
                                let paste_mode = config.as_ref().map(|c| c.paste_mode.clone()).unwrap_or_default();
                                let auto = config.as_ref().map(|c| c.auto_polish).unwrap_or(false);

                                if auto {
                                    let ai_model = config.as_ref().map(|c| c.ai_model.clone()).unwrap_or_default();
                                    let prompt = config.as_ref().map(|c| c.ai_polish_prompt.clone()).unwrap_or_default();

                                    let _ = app.emit("polishing", ());
                                    match crate::ollama::polish(&text, &ai_model, &prompt).await {
                                        Ok(polished) => {
                                            if cancel::is_cancelled() {
                                                let _ = app.emit("transcription-cancelled", ());
                                                let _ = std::fs::remove_file(&wav_path);
                                                return;
                                            }
                                            let _ = crate::db::update_transcription_ai(id, &polished);
                                            crate::clipboard::apply_paste_mode(&polished, &paste_mode);
                                            let _ = app.emit("polish-done", polished);
                                            if let Some(win) = app.get_webview_window("loon") {
                                                let _ = win.hide();
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("auto-polish failed: {e}");
                                            let _ = app.emit("transcription-done", serde_json::json!({"id": id, "text": text}));
                                            if let Some(win) = app.get_webview_window("loon") {
                                                let _ = win.hide();
                                            }
                                        }
                                    }
                                } else {
                                    crate::clipboard::apply_paste_mode(&text, &paste_mode);
                                    let _ = app.emit("transcription-done", serde_json::json!({"id": id, "text": text}));
                                    if let Some(win) = app.get_webview_window("loon") {
                                        let _ = win.hide();
                                    }
                                }
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

        if visible {
            // Window is visible and not recording — cancel flow
            if cancel::request_cancel() {
                // Second press — actually cancel and hide
                let _ = app.emit("transcription-cancelled", ());
                let _ = window.hide();
            } else {
                // First press — show "Press Again to Cancel"
                let _ = window.emit("cancel-requested", ());
            }
            return;
        }

        match crate::recorder::start_recording() {
            Ok(()) => {
                let _ = window.show();
                let _ = window.emit("start-recording", ());
            }
            Err(e) => {
                eprintln!("start_recording failed: {e}");
            }
        }
    }
}
