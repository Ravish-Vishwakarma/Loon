use crate::config::PasteMode;
use enigo::{Direction, Enigo, Keyboard, Settings};

pub fn copy_to_clipboard(text: &str) -> bool {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        clipboard.set_text(text).is_ok()
    } else {
        false
    }
}

pub fn simulate_paste() -> bool {
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(_) => return false,
    };
    let ctrl = enigo.key(enigo::Key::Control, Direction::Press);
    let v = enigo.key(enigo::Key::Unicode('v'), Direction::Click);
    let ctrl_release = enigo.key(enigo::Key::Control, Direction::Release);
    ctrl.is_ok() && v.is_ok() && ctrl_release.is_ok()
}

pub fn apply_paste_mode(text: &str, mode: &PasteMode) {
    match mode {
        PasteMode::Copy => {
            copy_to_clipboard(text);
        }
        PasteMode::Paste => {
            copy_to_clipboard(text);
            simulate_paste();
        }
        PasteMode::Both => {
            copy_to_clipboard(text);
            simulate_paste();
        }
    }
}
