// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Handle Ctrl+C gracefully on Windows
    #[cfg(windows)]
    {
        let _ = ctrlc::set_handler(|| {
            std::process::exit(0);
        });
    }

    clitter_lib::run()
}
