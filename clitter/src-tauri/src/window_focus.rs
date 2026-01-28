#[cfg(windows)]
use windows::Win32::{
    Foundation::HWND,
    UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
        KEYEVENTF_KEYUP, VK_CONTROL, VK_V,
    },
    UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow},
};

#[cfg(any(windows, target_os = "macos"))]
use once_cell::sync::Lazy;
#[cfg(any(windows, target_os = "macos"))]
use std::sync::Mutex;

#[cfg(windows)]
static PREVIOUS_WINDOW: Lazy<Mutex<Option<isize>>> = Lazy::new(|| Mutex::new(None));

#[cfg(target_os = "macos")]
static PREVIOUS_APP: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

/// Save the current foreground window/app before showing Clitter
pub fn save_previous_window() {
    #[cfg(windows)]
    {
        unsafe {
            let hwnd = GetForegroundWindow();
            if let Ok(mut prev) = PREVIOUS_WINDOW.lock() {
                *prev = Some(hwnd.0 as isize);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Get the frontmost application using AppleScript
        let output = std::process::Command::new("osascript")
            .args(["-e", "tell application \"System Events\" to get name of first application process whose frontmost is true"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let app_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if let Ok(mut prev) = PREVIOUS_APP.lock() {
                    *prev = Some(app_name);
                }
            }
        }
    }
}

/// Restore focus to the previous window and simulate paste
pub fn restore_and_paste() -> Result<(), String> {
    #[cfg(windows)]
    {
        unsafe {
            let hwnd = {
                let prev = PREVIOUS_WINDOW.lock().map_err(|e| e.to_string())?;
                prev.map(|h| HWND(h as *mut std::ffi::c_void))
            };

            if let Some(hwnd) = hwnd {
                // Small delay to ensure clipboard is ready
                std::thread::sleep(std::time::Duration::from_millis(50));

                // Restore focus to previous window
                let _ = SetForegroundWindow(hwnd);

                // Small delay to ensure window is focused
                std::thread::sleep(std::time::Duration::from_millis(50));

                // Simulate Ctrl+V
                let inputs = [
                    // Ctrl down
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VK_CONTROL,
                                wScan: 0,
                                dwFlags: KEYBD_EVENT_FLAGS(0),
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                    // V down
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VK_V,
                                wScan: 0,
                                dwFlags: KEYBD_EVENT_FLAGS(0),
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                    // V up
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VK_V,
                                wScan: 0,
                                dwFlags: KEYEVENTF_KEYUP,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                    // Ctrl up
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VK_CONTROL,
                                wScan: 0,
                                dwFlags: KEYEVENTF_KEYUP,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                ];

                let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                if sent != inputs.len() as u32 {
                    return Err("Failed to send all key inputs".to_string());
                }
            }

            return Ok(());
        }
    }

    #[cfg(target_os = "macos")]
    {
        let app_name = {
            let prev = PREVIOUS_APP.lock().map_err(|e| e.to_string())?;
            prev.clone()
        };

        if let Some(app_name) = app_name {
            // Small delay to ensure clipboard is ready
            std::thread::sleep(std::time::Duration::from_millis(50));

            // Activate the previous application and paste using AppleScript
            let script = format!(
                r#"
                tell application "{}" to activate
                delay 0.1
                tell application "System Events"
                    keystroke "v" using command down
                end tell
                "#,
                app_name.replace("\"", "\\\"")
            );

            let output = std::process::Command::new("osascript")
                .args(["-e", &script])
                .output()
                .map_err(|e| e.to_string())?;

            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                return Err(format!("AppleScript error: {}", err));
            }
        }

        return Ok(());
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    {
        Err("Auto-paste not supported on this platform".to_string())
    }
}
