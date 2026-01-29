use std::sync::RwLock;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use crate::window_focus;
use crate::APP_STATE;

// Store current shortcut for unregistration
static CURRENT_SHORTCUT: RwLock<Option<Shortcut>> = RwLock::new(None);

pub fn register_global_shortcuts(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Default shortcut - will be updated after storage is initialized
    let default_shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyV);
    register_shortcut_internal(app.handle(), default_shortcut)?;
    Ok(())
}

fn register_shortcut_internal(app: &AppHandle, shortcut: Shortcut) -> Result<(), String> {
    match app.global_shortcut()
        .on_shortcut(shortcut.clone(), move |app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }

            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    window_focus::save_previous_window();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }) {
        Ok(_) => {
            // Store current shortcut
            if let Ok(mut current) = CURRENT_SHORTCUT.write() {
                *current = Some(shortcut);
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to register shortcut: {}", e)),
    }
}

pub fn change_shortcut(app: &AppHandle, new_shortcut_str: &str) -> Result<(), String> {
    let new_shortcut = parse_shortcut(new_shortcut_str)?;

    // Try to register new shortcut first
    match app.global_shortcut()
        .on_shortcut(new_shortcut.clone(), move |app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }

            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    window_focus::save_previous_window();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }) {
        Ok(_) => {
            // Success - unregister old shortcut
            if let Ok(current) = CURRENT_SHORTCUT.read() {
                if let Some(old_shortcut) = current.as_ref() {
                    let _ = app.global_shortcut().unregister(old_shortcut.clone());
                }
            }

            // Update current shortcut
            if let Ok(mut current) = CURRENT_SHORTCUT.write() {
                *current = Some(new_shortcut);
            }

            Ok(())
        }
        Err(e) => Err(format!("ショートカットを登録できません。他のアプリで使用中の可能性があります: {}", e)),
    }
}

pub fn parse_shortcut(shortcut_str: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = shortcut_str.split('+').map(|s| s.trim()).collect();

    if parts.is_empty() {
        return Err("Empty shortcut".to_string());
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for part in parts {
        match part.to_uppercase().as_str() {
            "ALT" => modifiers |= Modifiers::ALT,
            "CTRL" | "CONTROL" => modifiers |= Modifiers::CONTROL,
            "SHIFT" => modifiers |= Modifiers::SHIFT,
            "META" | "WIN" | "SUPER" | "CMD" => modifiers |= Modifiers::META,
            _ => {
                // This should be the key
                key_code = Some(parse_key_code(part)?);
            }
        }
    }

    let code = key_code.ok_or("No key specified in shortcut")?;

    if modifiers.is_empty() {
        return Err("At least one modifier (Alt, Ctrl, Shift) is required".to_string());
    }

    Ok(Shortcut::new(Some(modifiers), code))
}

fn parse_key_code(key: &str) -> Result<Code, String> {
    match key.to_uppercase().as_str() {
        "A" => Ok(Code::KeyA),
        "B" => Ok(Code::KeyB),
        "C" => Ok(Code::KeyC),
        "D" => Ok(Code::KeyD),
        "E" => Ok(Code::KeyE),
        "F" => Ok(Code::KeyF),
        "G" => Ok(Code::KeyG),
        "H" => Ok(Code::KeyH),
        "I" => Ok(Code::KeyI),
        "J" => Ok(Code::KeyJ),
        "K" => Ok(Code::KeyK),
        "L" => Ok(Code::KeyL),
        "M" => Ok(Code::KeyM),
        "N" => Ok(Code::KeyN),
        "O" => Ok(Code::KeyO),
        "P" => Ok(Code::KeyP),
        "Q" => Ok(Code::KeyQ),
        "R" => Ok(Code::KeyR),
        "S" => Ok(Code::KeyS),
        "T" => Ok(Code::KeyT),
        "U" => Ok(Code::KeyU),
        "V" => Ok(Code::KeyV),
        "W" => Ok(Code::KeyW),
        "X" => Ok(Code::KeyX),
        "Y" => Ok(Code::KeyY),
        "Z" => Ok(Code::KeyZ),
        "0" | "DIGIT0" => Ok(Code::Digit0),
        "1" | "DIGIT1" => Ok(Code::Digit1),
        "2" | "DIGIT2" => Ok(Code::Digit2),
        "3" | "DIGIT3" => Ok(Code::Digit3),
        "4" | "DIGIT4" => Ok(Code::Digit4),
        "5" | "DIGIT5" => Ok(Code::Digit5),
        "6" | "DIGIT6" => Ok(Code::Digit6),
        "7" | "DIGIT7" => Ok(Code::Digit7),
        "8" | "DIGIT8" => Ok(Code::Digit8),
        "9" | "DIGIT9" => Ok(Code::Digit9),
        "F1" => Ok(Code::F1),
        "F2" => Ok(Code::F2),
        "F3" => Ok(Code::F3),
        "F4" => Ok(Code::F4),
        "F5" => Ok(Code::F5),
        "F6" => Ok(Code::F6),
        "F7" => Ok(Code::F7),
        "F8" => Ok(Code::F8),
        "F9" => Ok(Code::F9),
        "F10" => Ok(Code::F10),
        "F11" => Ok(Code::F11),
        "F12" => Ok(Code::F12),
        "SPACE" => Ok(Code::Space),
        "ENTER" | "RETURN" => Ok(Code::Enter),
        "TAB" => Ok(Code::Tab),
        "BACKSPACE" => Ok(Code::Backspace),
        "ESCAPE" | "ESC" => Ok(Code::Escape),
        "DELETE" | "DEL" => Ok(Code::Delete),
        "INSERT" | "INS" => Ok(Code::Insert),
        "HOME" => Ok(Code::Home),
        "END" => Ok(Code::End),
        "PAGEUP" => Ok(Code::PageUp),
        "PAGEDOWN" => Ok(Code::PageDown),
        "UP" | "ARROWUP" => Ok(Code::ArrowUp),
        "DOWN" | "ARROWDOWN" => Ok(Code::ArrowDown),
        "LEFT" | "ARROWLEFT" => Ok(Code::ArrowLeft),
        "RIGHT" | "ARROWRIGHT" => Ok(Code::ArrowRight),
        "`" | "BACKQUOTE" => Ok(Code::Backquote),
        "-" | "MINUS" => Ok(Code::Minus),
        "=" | "EQUAL" => Ok(Code::Equal),
        "[" | "BRACKETLEFT" => Ok(Code::BracketLeft),
        "]" | "BRACKETRIGHT" => Ok(Code::BracketRight),
        "\\" | "BACKSLASH" => Ok(Code::Backslash),
        ";" | "SEMICOLON" => Ok(Code::Semicolon),
        "'" | "QUOTE" => Ok(Code::Quote),
        "," | "COMMA" => Ok(Code::Comma),
        "." | "PERIOD" => Ok(Code::Period),
        "/" | "SLASH" => Ok(Code::Slash),
        _ => Err(format!("Unknown key: {}", key)),
    }
}

/// Load and apply saved shortcut from storage
pub async fn load_saved_shortcut(app: &AppHandle) {
    if let Some(state) = APP_STATE.get() {
        let storage_guard = state.persistent_storage.read().await;
        if let Some(storage) = storage_guard.as_ref() {
            match storage.get_setting("global_shortcut").await {
                Ok(Some(shortcut_str)) => {
                    if let Err(e) = change_shortcut(app, &shortcut_str) {
                        eprintln!("Failed to load saved shortcut '{}': {}", shortcut_str, e);
                    } else {
                        println!("Loaded saved shortcut: {}", shortcut_str);
                    }
                }
                Ok(None) => {
                    println!("No saved shortcut found, using default");
                }
                Err(e) => {
                    eprintln!("Failed to read shortcut setting: {}", e);
                }
            }
        }
    }
}
