pub mod clipboard;
pub mod commands;
pub mod crypto;
pub mod hotkey;
pub mod storage;
pub mod types;

use once_cell::sync::OnceCell;
use storage::{persistent::PersistentStorage, volatile::VolatileStorage};
use tokio::sync::RwLock;

pub static APP_STATE: OnceCell<AppState> = OnceCell::new();

pub struct AppState {
    pub volatile_storage: VolatileStorage,
    pub persistent_storage: RwLock<Option<PersistentStorage>>,
    pub crypto_key: RwLock<Option<[u8; 32]>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            volatile_storage: VolatileStorage::new(),
            persistent_storage: RwLock::new(None),
            crypto_key: RwLock::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();
    if APP_STATE.set(app_state).is_err() {
        panic!("Failed to initialize app state");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            hotkey::register_global_shortcuts(app)?;

            // Initialize persistent storage in background
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match PersistentStorage::new().await {
                    Ok(storage) => {
                        if let Some(state) = APP_STATE.get() {
                            let mut ps = state.persistent_storage.write().await;
                            *ps = Some(storage);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize persistent storage: {}", e);
                    }
                }
            });

            // Start clipboard monitoring
            clipboard::monitor::start_monitoring(app_handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_clipboard_history,
            commands::get_recent_items,
            commands::copy_to_clipboard,
            commands::get_whiteboard,
            commands::add_to_whiteboard,
            commands::update_whiteboard_item,
            commands::remove_from_whiteboard,
            commands::create_group,
            commands::update_group,
            commands::delete_group,
            commands::set_item_shortcut,
            commands::get_items_with_shortcuts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
