use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use crate::window_focus;

pub fn register_global_shortcuts(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Alt+V: Toggle window visibility
    let toggle_shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyV);

    match app.global_shortcut()
        .on_shortcut(toggle_shortcut, |app, _shortcut, event| {
            // Only respond to key press, not release
            if event.state != ShortcutState::Pressed {
                return;
            }

            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    // Save the current foreground window before showing Clitter
                    window_focus::save_previous_window();
                    let _ = window.show();
                    // Don't set focus - let the window appear without stealing focus
                    // This allows inline editing (like file rename) to continue
                }
            }
        }) {
        Ok(_) => println!("Global shortcut Alt+V registered successfully"),
        Err(e) => eprintln!("Failed to register global shortcut: {}", e),
    }

    Ok(())
}
