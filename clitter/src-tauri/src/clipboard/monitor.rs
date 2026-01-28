use arboard::Clipboard;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::clipboard::categorizer::Categorizer;
use crate::types::{ClipboardContent, ClipboardData};
use crate::APP_STATE;

static LAST_CONTENT_HASH: AtomicU64 = AtomicU64::new(0);
// Hash of content that was copied by Clitter itself (should be skipped)
static SELF_COPIED_HASH: AtomicU64 = AtomicU64::new(0);

/// Mark a content hash as self-copied (will be skipped by monitor)
pub fn mark_as_self_copied(hash: u64) {
    SELF_COPIED_HASH.store(hash, Ordering::Relaxed);
}

pub fn start_monitoring(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut clipboard = match Clipboard::new() {
            Ok(cb) => cb,
            Err(e) => {
                eprintln!("Failed to create clipboard: {}", e);
                return;
            }
        };

        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            if let Some(content) = check_clipboard_change(&mut clipboard) {
                // Add to volatile storage
                if let Some(state) = APP_STATE.get() {
                    state.volatile_storage.add(content.clone()).await;
                }

                // Emit event to frontend
                let _ = app_handle.emit("clipboard-changed", &content);
            }
        }
    });
}

fn check_clipboard_change(clipboard: &mut Clipboard) -> Option<ClipboardContent> {
    // Try to get text first
    if let Ok(text) = clipboard.get_text() {
        if !text.is_empty() {
            let preview = text.chars().take(100).collect();
            let data = ClipboardData::Text {
                text: text.clone(),
                preview,
            };
            let category = Categorizer::categorize(&data);
            let content = ClipboardContent {
                id: uuid::Uuid::new_v4(),
                category,
                data,
                copied_at: chrono::Utc::now(),
                source: None,
            };

            let hash = content.content_hash();
            let last_hash = LAST_CONTENT_HASH.load(Ordering::Relaxed);
            let self_copied_hash = SELF_COPIED_HASH.load(Ordering::Relaxed);

            if hash != last_hash {
                LAST_CONTENT_HASH.store(hash, Ordering::Relaxed);
                // Skip if this was copied by Clitter itself
                if hash == self_copied_hash {
                    SELF_COPIED_HASH.store(0, Ordering::Relaxed); // Reset
                    return None;
                }
                return Some(content);
            }
        }
    }

    // Try to get image
    if let Ok(img) = clipboard.get_image() {
        // Convert RGBA pixel data to PNG format
        let png_bytes = match encode_rgba_to_png(&img.bytes, img.width as u32, img.height as u32) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Failed to encode image to PNG: {}", e);
                return None;
            }
        };

        let base64 = STANDARD.encode(&png_bytes);
        let hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            base64.hash(&mut hasher);
            hasher.finish()
        };

        let last_hash = LAST_CONTENT_HASH.load(Ordering::Relaxed);
        let self_copied_hash = SELF_COPIED_HASH.load(Ordering::Relaxed);

        if hash != last_hash {
            LAST_CONTENT_HASH.store(hash, Ordering::Relaxed);
            // Skip if this was copied by Clitter itself
            if hash == self_copied_hash {
                SELF_COPIED_HASH.store(0, Ordering::Relaxed); // Reset
                return None;
            }

            let content = ClipboardContent::new_image(
                base64,
                img.width as u32,
                img.height as u32,
                "png".to_string(),
            );
            return Some(content);
        }
    }

    None
}

fn encode_rgba_to_png(rgba_bytes: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    use image::{ImageBuffer, Rgba};
    use std::io::Cursor;

    // Create an image buffer from RGBA bytes
    let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, rgba_bytes.to_vec())
            .ok_or_else(|| "Failed to create image buffer".to_string())?;

    // Encode to PNG
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);

    img_buffer
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;

    Ok(png_bytes)
}
