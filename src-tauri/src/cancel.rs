use std::sync::atomic::{AtomicBool, Ordering};
use once_cell::sync::OnceCell;

static CANCELLED: OnceCell<AtomicBool> = OnceCell::new();
static CANCEL_PENDING: OnceCell<AtomicBool> = OnceCell::new();

pub fn init() {
    CANCELLED.set(AtomicBool::new(false)).ok();
    CANCEL_PENDING.set(AtomicBool::new(false)).ok();
}

pub fn cancel() {
    if let Some(flag) = CANCELLED.get() {
        flag.store(true, Ordering::SeqCst);
    }
}

pub fn is_cancelled() -> bool {
    CANCELLED
        .get()
        .map(|f| f.load(Ordering::SeqCst))
        .unwrap_or(false)
}

pub fn reset() {
    if let Some(flag) = CANCELLED.get() {
        flag.store(false, Ordering::SeqCst);
    }
    if let Some(flag) = CANCEL_PENDING.get() {
        flag.store(false, Ordering::SeqCst);
    }
}

pub fn reset_cancel_pending() {
    if let Some(flag) = CANCEL_PENDING.get() {
        flag.store(false, Ordering::SeqCst);
    }
}

pub fn request_cancel() -> bool {
    if let Some(flag) = CANCEL_PENDING.get() {
        if flag.load(Ordering::SeqCst) {
            // Second press — actually cancel
            cancel();
            return true;
        } else {
            // First press — just mark pending
            flag.store(true, Ordering::SeqCst);
            return false;
        }
    }
    false
}

#[tauri::command]
pub fn reset_cancel_pending_cmd() {
    reset_cancel_pending();
}
