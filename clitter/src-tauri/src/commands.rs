use arboard::Clipboard;
use uuid::Uuid;

use crate::clipboard::monitor::mark_as_self_copied;
use crate::types::{Category, ClipboardContent, ClipboardData, Group, Position, Size, WhiteboardItem, WhiteboardState};
use crate::window_focus;
use crate::APP_STATE;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedItem {
    pub shortcut: Option<String>,
    pub value: String,
    pub parent_group: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedGroup {
    pub name: String,
    pub shortcut: Option<String>,
    pub color: Option<String>,
    pub parent_group: Option<String>,
    pub items: Option<Vec<ImportedItem>>,
    pub groups: Option<Vec<ImportedGroup>>,
}

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
    // Mark this content as self-copied so monitor will skip it
    mark_as_self_copied(content.content_hash());

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;

    match &content.data {
        ClipboardData::Text { text, .. } => {
            clipboard.set_text(text).map_err(|e| e.to_string())?;
        }
        ClipboardData::Image { base64, width: stored_w, height: stored_h, .. } => {
            use base64::{engine::general_purpose::STANDARD, Engine};
            let png_bytes = STANDARD.decode(base64).map_err(|e| e.to_string())?;
            eprintln!("[copy_to_clipboard] PNG bytes: {}, stored size: {}x{}", png_bytes.len(), stored_w, stored_h);

            // Decode PNG to RGBA
            let img = image::load_from_memory(&png_bytes).map_err(|e| format!("Failed to decode PNG: {}", e))?;
            let rgba = img.to_rgba8();

            let width = rgba.width() as usize;
            let height = rgba.height() as usize;
            let raw_bytes = rgba.into_raw();
            eprintln!("[copy_to_clipboard] RGBA size: {}x{}, bytes: {}", width, height, raw_bytes.len());

            // Verify buffer size matches expected RGBA size
            let expected_size = width * height * 4;
            if raw_bytes.len() != expected_size {
                return Err(format!(
                    "RGBA buffer size mismatch: expected {} ({}x{}x4), got {}",
                    expected_size, width, height, raw_bytes.len()
                ));
            }

            let img_data = arboard::ImageData {
                width,
                height,
                bytes: raw_bytes.into(),
            };

            // Use catch_unwind to prevent panic from crashing the app
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                clipboard.set_image(img_data)
            }));

            match result {
                Ok(Ok(())) => eprintln!("[copy_to_clipboard] Image set successfully"),
                Ok(Err(e)) => return Err(format!("Failed to set clipboard image: {}", e)),
                Err(_) => return Err("Clipboard set_image panicked - this may be a bug in arboard".to_string()),
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn paste_to_previous_window(content: ClipboardContent) -> Result<(), String> {
    // Mark this content as self-copied so monitor will skip it
    mark_as_self_copied(content.content_hash());

    // First copy to clipboard
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;

    match &content.data {
        ClipboardData::Text { text, .. } => {
            clipboard.set_text(text).map_err(|e| e.to_string())?;
        }
        ClipboardData::Image { base64, width: stored_w, height: stored_h, .. } => {
            use base64::{engine::general_purpose::STANDARD, Engine};
            let png_bytes = STANDARD.decode(base64).map_err(|e| e.to_string())?;
            eprintln!("[paste_to_previous_window] PNG bytes: {}, stored size: {}x{}", png_bytes.len(), stored_w, stored_h);

            // Decode PNG to RGBA
            let img = image::load_from_memory(&png_bytes).map_err(|e| format!("Failed to decode PNG: {}", e))?;
            let rgba = img.to_rgba8();

            let width = rgba.width() as usize;
            let height = rgba.height() as usize;
            let raw_bytes = rgba.into_raw();
            eprintln!("[paste_to_previous_window] RGBA size: {}x{}, bytes: {}", width, height, raw_bytes.len());

            // Verify buffer size matches expected RGBA size
            let expected_size = width * height * 4;
            if raw_bytes.len() != expected_size {
                return Err(format!(
                    "RGBA buffer size mismatch: expected {} ({}x{}x4), got {}",
                    expected_size, width, height, raw_bytes.len()
                ));
            }

            let img_data = arboard::ImageData {
                width,
                height,
                bytes: raw_bytes.into(),
            };

            // Use catch_unwind to prevent panic from crashing the app
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                clipboard.set_image(img_data)
            }));

            match result {
                Ok(Ok(())) => eprintln!("[paste_to_previous_window] Image set successfully"),
                Ok(Err(e)) => return Err(format!("Failed to set clipboard image: {}", e)),
                Err(_) => return Err("Clipboard set_image panicked - this may be a bug in arboard".to_string()),
            }
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

    let mut item = WhiteboardItem::new(content, position);

    // Auto-generate sequential shortcut
    if let Some(storage) = storage.as_ref() {
        let next_num = storage.get_next_shortcut_number().await.map_err(|e| e.to_string())?;
        item.shortcut = Some(format!("{}", next_num));

        storage
            .save_whiteboard_item(&item)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(item)
}

#[tauri::command]
pub async fn add_text_to_whiteboard(
    text: String,
    position: Position,
    parent_group: Option<String>,
    label: Option<String>,
) -> Result<WhiteboardItem, String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    let content = ClipboardContent::new_text(text, Category::Text);
    let mut item = WhiteboardItem::new(content, position);

    // Set parent group if provided
    if let Some(pg) = parent_group {
        item.parent_group = Some(Uuid::parse_str(&pg).map_err(|e| e.to_string())?);
    }

    // Set label if provided
    item.label = label;

    // Auto-generate sequential shortcut
    if let Some(storage) = storage.as_ref() {
        let next_num = storage.get_next_shortcut_number().await.map_err(|e| e.to_string())?;
        item.shortcut = Some(format!("{}", next_num));

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
pub async fn create_group(name: String, position: Position, parent_group: Option<String>) -> Result<Group, String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    let mut group = Group::new(name, position);

    // Set parent group if provided
    if let Some(pg) = parent_group {
        group.parent_group = Some(Uuid::parse_str(&pg).map_err(|e| e.to_string())?);
    }

    // Auto-generate sequential shortcut
    if let Some(storage) = storage.as_ref() {
        let next_num = storage.get_next_group_shortcut_number().await.map_err(|e| e.to_string())?;
        group.shortcut = Some(format!("g{}", next_num));

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
pub async fn set_group_color(id: String, color: Option<String>) -> Result<(), String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    if let Some(storage) = storage.as_ref() {
        storage
            .update_group_color(id, color)
            .await
            .map_err(|e| e.to_string())?;
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

#[tauri::command]
pub async fn import_whiteboard_json(json: String) -> Result<WhiteboardState, String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    // Parse the JSON as a list of groups
    let imported_groups: Vec<ImportedGroup> = serde_json::from_str(&json)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    if let Some(storage) = storage.as_ref() {
        // Process each top-level group recursively
        for group in imported_groups {
            import_group_recursive(storage, group, None).await?;
        }

        // Return the updated whiteboard state
        storage.load_whiteboard().await.map_err(|e| e.to_string())
    } else {
        Err("Storage not available".to_string())
    }
}

async fn import_group_recursive(
    storage: &crate::storage::persistent::PersistentStorage,
    imported: ImportedGroup,
    parent_id: Option<Uuid>,
) -> Result<(), String> {
    // Create the group
    let mut group = Group::new(imported.name, Position { x: 0.0, y: 0.0 });
    group.shortcut = imported.shortcut;
    group.color = imported.color;
    group.parent_group = parent_id;

    let group_id = group.id;

    storage.save_group(&group).await.map_err(|e| e.to_string())?;

    // Import items in this group
    if let Some(items) = imported.items {
        for item in items {
            let content = ClipboardContent::new_text(item.value, Category::Text);
            let mut whiteboard_item = WhiteboardItem::new(content, Position { x: 0.0, y: 0.0 });
            whiteboard_item.shortcut = item.shortcut;
            whiteboard_item.parent_group = Some(group_id);

            storage.save_whiteboard_item(&whiteboard_item).await.map_err(|e| e.to_string())?;
        }
    }

    // Recursively import child groups
    if let Some(groups) = imported.groups {
        for child_group in groups {
            Box::pin(import_group_recursive(storage, child_group, Some(group_id))).await?;
        }
    }

    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedItem {
    pub shortcut: Option<String>,
    pub label: Option<String>,
    pub value: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedGroup {
    pub name: String,
    pub shortcut: Option<String>,
    pub color: Option<String>,
    pub items: Vec<ExportedItem>,
    pub groups: Vec<ExportedGroup>,
}

#[tauri::command]
pub async fn export_whiteboard_json() -> Result<String, String> {
    let state = APP_STATE.get().ok_or("App state not initialized")?;
    let storage = state.persistent_storage.read().await;

    if let Some(storage) = storage.as_ref() {
        let whiteboard = storage.load_whiteboard().await.map_err(|e| e.to_string())?;

        // Build export structure from root groups
        let mut root_groups: Vec<ExportedGroup> = Vec::new();

        for group in whiteboard.groups.values() {
            if group.parent_group.is_none() {
                let exported = export_group_recursive(&whiteboard, group.id);
                root_groups.push(exported);
            }
        }

        let json = serde_json::to_string_pretty(&root_groups).map_err(|e| e.to_string())?;
        Ok(json)
    } else {
        Err("Storage not available".to_string())
    }
}

fn export_group_recursive(whiteboard: &WhiteboardState, group_id: Uuid) -> ExportedGroup {
    let group = &whiteboard.groups[&group_id];

    // Get items in this group
    let items: Vec<ExportedItem> = whiteboard
        .items
        .values()
        .filter(|item| item.parent_group == Some(group_id))
        .map(|item| {
            let value = match &item.content.data {
                ClipboardData::Text { text, .. } => text.clone(),
                ClipboardData::Image { .. } => "[image]".to_string(),
            };
            ExportedItem {
                shortcut: item.shortcut.clone(),
                label: item.label.clone(),
                value,
            }
        })
        .collect();

    // Get child groups recursively
    let child_groups: Vec<ExportedGroup> = whiteboard
        .groups
        .values()
        .filter(|g| g.parent_group == Some(group_id))
        .map(|g| export_group_recursive(whiteboard, g.id))
        .collect();

    ExportedGroup {
        name: group.name.clone(),
        shortcut: group.shortcut.clone(),
        color: group.color.clone(),
        items,
        groups: child_groups,
    }
}
