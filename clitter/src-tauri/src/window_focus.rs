#[cfg(windows)]
use windows::Win32::{
    Foundation::HWND,
    UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
        KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL, VK_V,
    },
    UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow},
};

use once_cell::sync::Lazy;
use std::sync::Mutex;

#[cfg(windows)]
static PREVIOUS_WINDOW: Lazy<Mutex<Option<isize>>> = Lazy::new(|| Mutex::new(None));

#[cfg(not(windows))]
#[allow(dead_code)]
static PREVIOUS_WINDOW: Lazy<Mutex<Option<()>>> = Lazy::new(|| Mutex::new(None));

/// Save the current foreground window before showing Clitter
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
}

/// Restore focus to the previous window and simulate Ctrl+V paste
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

            Ok(())
        }
    }

    #[cfg(not(windows))]
    {
        Err("Auto-paste not supported on this platform".to_string())
    }
}
