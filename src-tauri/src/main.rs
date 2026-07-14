// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Write panics to a log file so they're visible in production
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {info}\n");
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            eprintln!("{s}");
        }
        let _ = std::fs::write("loon-crash.log", &msg);
        // Also append to existing log
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("loon-crash.log")
        {
            let _ = f.write_all(msg.as_bytes());
        }
    }));
    loon_lib::run()
}
