//! Windows-specific clipboard operations for better compatibility
//! Uses native Win32 API to set images in DIB format and as files

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*,
    Win32::System::DataExchange::*,
    Win32::System::Memory::*,
    Win32::Graphics::Gdi::*,
};

#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

#[cfg(target_os = "windows")]
const CF_DIB: u32 = 8;    // Device Independent Bitmap
#[cfg(target_os = "windows")]
const CF_HDROP: u32 = 15; // File drop format

/// DROPFILES structure for CF_HDROP
#[cfg(target_os = "windows")]
#[repr(C, packed)]
struct DROPFILES {
    p_files: u32,  // Offset to file list
    pt_x: i32,     // Drop point x
    pt_y: i32,     // Drop point y
    f_nc: i32,     // Is it on non-client area
    f_wide: i32,   // Wide character flag (1 = Unicode)
}

#[cfg(target_os = "windows")]
pub fn set_image_to_clipboard(rgba_bytes: &[u8], width: u32, height: u32) -> Result<(), String> {
    // First, save image to temp file for CF_HDROP
    let temp_path = save_to_temp_png(rgba_bytes, width, height)?;

    unsafe {
        // Open clipboard
        OpenClipboard(Some(HWND::default())).map_err(|e| format!("Failed to open clipboard: {}", e))?;

        // Empty clipboard
        if let Err(e) = EmptyClipboard() {
            let _ = CloseClipboard();
            return Err(format!("Failed to empty clipboard: {}", e));
        }

        // Set CF_DIB format (for Paint, image editors, etc.)
        if let Err(e) = set_dib_data(rgba_bytes, width, height) {
            eprintln!("[windows_clipboard] Failed to set DIB: {}", e);
        }

        // Set CF_HDROP format (for file explorers, VS Code folders, etc.)
        if let Err(e) = set_hdrop_data(&temp_path) {
            eprintln!("[windows_clipboard] Failed to set HDROP: {}", e);
        }

        let _ = CloseClipboard();
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn save_to_temp_png(rgba_bytes: &[u8], width: u32, height: u32) -> Result<String, String> {
    use image::{ImageBuffer, Rgba};
    // Create image from RGBA bytes
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, rgba_bytes.to_vec())
        .ok_or("Failed to create image buffer")?;

    // Get temp directory
    let temp_dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let temp_path = temp_dir.join(format!("clitter_clipboard_{}.png", timestamp));

    // Save as PNG
    img.save(&temp_path).map_err(|e| format!("Failed to save temp PNG: {}", e))?;

    Ok(temp_path.to_string_lossy().to_string())
}

#[cfg(target_os = "windows")]
unsafe fn set_dib_data(rgba_bytes: &[u8], width: u32, height: u32) -> Result<(), String> {
    // Create DIB (Device Independent Bitmap)
    let row_size = ((width * 32 + 31) / 32) * 4;
    let pixel_data_size = row_size * height;
    let header_size = std::mem::size_of::<BITMAPINFOHEADER>();
    let total_size = header_size + pixel_data_size as usize;

    let hglobal = GlobalAlloc(GMEM_MOVEABLE, total_size)
        .map_err(|e| format!("Failed to allocate DIB memory: {}", e))?;

    let ptr = GlobalLock(hglobal);
    if ptr.is_null() {
        let _ = GlobalFree(Some(hglobal));
        return Err("Failed to lock DIB memory".to_string());
    }

    // Write BITMAPINFOHEADER
    let header = BITMAPINFOHEADER {
        biSize: header_size as u32,
        biWidth: width as i32,
        biHeight: height as i32,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0 as u32,
        biSizeImage: pixel_data_size,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    };

    std::ptr::copy_nonoverlapping(
        &header as *const _ as *const u8,
        ptr as *mut u8,
        header_size,
    );

    // Write pixel data (RGBA to BGRA, flip vertically)
    let pixel_ptr = (ptr as *mut u8).add(header_size);
    for y in 0..height {
        let src_row = (height - 1 - y) as usize;
        let src_offset = src_row * (width as usize) * 4;
        let dst_offset = (y as usize) * (row_size as usize);

        for x in 0..width as usize {
            let src_pixel = src_offset + x * 4;
            let dst_pixel = dst_offset + x * 4;
            *pixel_ptr.add(dst_pixel) = rgba_bytes[src_pixel + 2];     // B
            *pixel_ptr.add(dst_pixel + 1) = rgba_bytes[src_pixel + 1]; // G
            *pixel_ptr.add(dst_pixel + 2) = rgba_bytes[src_pixel];     // R
            *pixel_ptr.add(dst_pixel + 3) = rgba_bytes[src_pixel + 3]; // A
        }
    }

    let _ = GlobalUnlock(hglobal);

    SetClipboardData(CF_DIB, Some(HANDLE(hglobal.0 as *mut std::ffi::c_void)))
        .map_err(|_| "Failed to set DIB clipboard data".to_string())?;

    Ok(())
}

#[cfg(target_os = "windows")]
unsafe fn set_hdrop_data(file_path: &str) -> Result<(), String> {
    // Convert path to wide string (UTF-16)
    let wide_path: Vec<u16> = OsStr::new(file_path)
        .encode_wide()
        .chain(std::iter::once(0)) // null terminator
        .collect();

    // DROPFILES structure + file path + double null terminator
    let dropfiles_size = std::mem::size_of::<DROPFILES>();
    let path_size = wide_path.len() * 2; // UTF-16 = 2 bytes per char
    let total_size = dropfiles_size + path_size + 2; // +2 for final null

    let hglobal = GlobalAlloc(GMEM_MOVEABLE, total_size)
        .map_err(|e| format!("Failed to allocate HDROP memory: {}", e))?;

    let ptr = GlobalLock(hglobal);
    if ptr.is_null() {
        let _ = GlobalFree(Some(hglobal));
        return Err("Failed to lock HDROP memory".to_string());
    }

    // Write DROPFILES header
    let dropfiles = DROPFILES {
        p_files: dropfiles_size as u32,
        pt_x: 0,
        pt_y: 0,
        f_nc: 0,
        f_wide: 1, // Unicode
    };
    std::ptr::copy_nonoverlapping(
        &dropfiles as *const _ as *const u8,
        ptr as *mut u8,
        dropfiles_size,
    );

    // Write file path (UTF-16)
    let path_ptr = (ptr as *mut u8).add(dropfiles_size) as *mut u16;
    std::ptr::copy_nonoverlapping(wide_path.as_ptr(), path_ptr, wide_path.len());

    // Final null terminator (already included in wide_path, but add extra for safety)
    *path_ptr.add(wide_path.len()) = 0;

    let _ = GlobalUnlock(hglobal);

    SetClipboardData(CF_HDROP, Some(HANDLE(hglobal.0 as *mut std::ffi::c_void)))
        .map_err(|_| "Failed to set HDROP clipboard data".to_string())?;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn set_image_to_clipboard(_rgba_bytes: &[u8], _width: u32, _height: u32) -> Result<(), String> {
    Err("Windows-specific clipboard not available on this platform".to_string())
}
