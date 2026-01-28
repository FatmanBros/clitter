use arboard::Clipboard;
use uuid::Uuid;

use crate::types::{Category, ClipboardContent, ClipboardData, Group, Position, Size, WhiteboardItem, WhiteboardState};
use crate::window_focus;
use crate::APP_STATE;

#[tauri::command]
pub async fn get_clipboard_history(category: Option<String>) -> Result<Vec<ClipboardContent>, String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;

    match category {
        Some(cat) => {
            let category: Category = cat.parse().map_err(|e: String| e)?;
            Ok(state.volatile_storage.get_by_category(category).await)
        }
        None => Ok(state.volatile_storage.get_all().await),
    }
}

#[tauri::command]
pub async fn get_recent_items(count: usize) -> Result<Vec<ClipboardContent>, String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    Ok(state.volatile_storage.get_recent(count).await)
}

#[tauri::command]
pub async fn copy_to_clipboard(content: ClipboardContent) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;

    match &content.data {
        ClipboardData::Text { text, .. } => {
            clipboard.set_text(text).map_err(|e| e.to_string())?;
        }
        ClipboardData::Image { base64, width, height, .. } => {
            use base64::{engine::general_purpose::STANDARD, Engine};
            let bytes = STANDARD.decode(base64).map_err(|e| e.to_string())?;

            let img_data = arboard::ImageData {
                width: *width as usize,
                height: *height as usize,
                bytes: bytes.into(),
            };
            clipboard.set_image(img_data).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn paste_to_previous_window(content: ClipboardContent) -> Result<(), String> {
    // First copy to clipboard
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;

    match &content.data {
        ClipboardData::Text { text, .. } => {
            clipboard.set_text(text).map_err(|e| e.to_string())?;
        }
        ClipboardData::Image { base64, width, height, .. } => {
            use base64::{engine::general_purpose::STANDARD, Engine};
            let bytes = STANDARD.decode(base64).map_err(|e| e.to_string())?;

            let img_data = arboard::ImageData {
                width: *width as usize,
                height: *height as usize,
                bytes: bytes.into(),
            };
            clipboard.set_image(img_data).map_err(|e| e.to_string())?;
        }
    }

    // Then restore focus and paste
    window_focus::restore_and_paste()
}

#[tauri::command]
pub async fn get_whiteboard() -> Result<WhiteboardState, String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    match storage.as_ref() {
        Some(storage) => storage.load_whiteboard().await.map_err(|e| e.to_string()),
        None => Ok(WhiteboardState::default()),
    }
}

#[tauri::command]
pub async fn add_to_whiteboard(
    content: ClipboardContent,
    position: Position,
) -> Result<WhiteboardItem, String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    let item = WhiteboardItem::new(content, position);

    if let Some(storage) = storage.as_ref() {
        storage
            .save_whiteboard_item(&item)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(item)
}

#[tauri::command]
pub async fn update_whiteboard_item(
    id: String,
    position: Option<Position>,
    size: Option<Size>,
    parent_group: Option<String>,
    shortcut: Option<String>,
) -> Result<(), String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    if let Some(storage) = storage.as_ref() {
        let mut whiteboard = storage.load_whiteboard().await.map_err(|e| e.to_string())?;

        if let Some(item) = whiteboard.items.get_mut(&id) {
            if let Some(pos) = position {
                item.position = pos;
            }
            if let Some(s) = size {
                item.size = s;
            }
            if let Some(pg) = parent_group {
                item.parent_group = if pg.is_empty() {
                    None
                } else {
                    Some(Uuid::parse_str(&pg).map_err(|e| e.to_string())?)
                };
            }
            if shortcut.is_some() {
                item.shortcut = shortcut;
            }
            item.updated_at = chrono::Utc::now();

            storage
                .save_whiteboard_item(item)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn remove_from_whiteboard(id: String) -> Result<(), String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    if let Some(storage) = storage.as_ref() {
        storage
            .delete_whiteboard_item(id)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn create_group(name: String, position: Position) -> Result<Group, String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    let group = Group::new(name, position);

    if let Some(storage) = storage.as_ref() {
        storage.save_group(&group).await.map_err(|e| e.to_string())?;
    }

    Ok(group)
}

#[tauri::command]
pub async fn update_group(
    id: String,
    name: Option<String>,
    collapsed: Option<bool>,
    position: Option<Position>,
) -> Result<(), String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    if let Some(storage) = storage.as_ref() {
        let mut whiteboard = storage.load_whiteboard().await.map_err(|e| e.to_string())?;

        if let Some(group) = whiteboard.groups.get_mut(&id) {
            if let Some(n) = name {
                group.name = n;
            }
            if let Some(c) = collapsed {
                group.collapsed = c;
            }
            if let Some(pos) = position {
                group.position = pos;
            }
            group.updated_at = chrono::Utc::now();

            storage.save_group(group).await.map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_group(id: String) -> Result<(), String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    if let Some(storage) = storage.as_ref() {
        storage.delete_group(id).await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn set_item_shortcut(id: String, shortcut: Option<String>) -> Result<(), String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    if let Some(storage) = storage.as_ref() {
        storage
            .update_item_shortcut(id, shortcut)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_items_with_shortcuts() -> Result<Vec<(String, String)>, String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    if let Some(storage) = storage.as_ref() {
        let items = storage
            .get_items_with_shortcuts()
            .await
            .map_err(|e| e.to_string())?;

        Ok(items
            .into_iter()
            .map(|(shortcut, id)| (shortcut, id.to_string()))
            .collect())
    } else {
        Ok(Vec::new())
    }
}
