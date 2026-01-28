//! Windows-specific clipboard operations for better compatibility
//! Uses native Win32 API to set images in DIB format

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*,
    Win32::System::DataExchange::*,
    Win32::System::Memory::*,
    Win32::Graphics::Gdi::*,
};

#[cfg(target_os = "windows")]
const CF_DIB: u32 = 8; // Standard Windows clipboard format for DIB

#[cfg(target_os = "windows")]
pub fn set_image_to_clipboard(rgba_bytes: &[u8], width: u32, height: u32) -> Result<(), String> {
    unsafe {
        // Open clipboard
        OpenClipboard(HWND::default()).map_err(|e| format!("Failed to open clipboard: {}", e))?;

        // Empty clipboard
        if let Err(e) = EmptyClipboard() {
            let _ = CloseClipboard();
            return Err(format!("Failed to empty clipboard: {}", e));
        }

        // Create DIB (Device Independent Bitmap)
        // DIB uses BGRA format (bottom-up by default)
        let row_size = ((width * 32 + 31) / 32) * 4; // 32-bit aligned rows
        let pixel_data_size = row_size * height;
        let header_size = std::mem::size_of::<BITMAPINFOHEADER>();
        let total_size = header_size + pixel_data_size as usize;

        // Allocate global memory
        let hglobal = GlobalAlloc(GMEM_MOVEABLE, total_size).map_err(|e| {
            let _ = CloseClipboard();
            format!("Failed to allocate global memory: {}", e)
        })?;

        let ptr = GlobalLock(hglobal);
        if ptr.is_null() {
            let _ = GlobalFree(hglobal);
            let _ = CloseClipboard();
            return Err("Failed to lock global memory".to_string());
        }

        // Write BITMAPINFOHEADER
        let header = BITMAPINFOHEADER {
            biSize: header_size as u32,
            biWidth: width as i32,
            biHeight: height as i32, // Positive = bottom-up DIB
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

        // Write pixel data (convert RGBA to BGRA and flip vertically)
        let pixel_ptr = (ptr as *mut u8).add(header_size);

        for y in 0..height {
            let src_row = (height - 1 - y) as usize; // Flip vertically (bottom-up)
            let src_offset = src_row * (width as usize) * 4;
            let dst_offset = (y as usize) * (row_size as usize);

            for x in 0..width as usize {
                let src_pixel = src_offset + x * 4;
                let dst_pixel = dst_offset + x * 4;

                // RGBA to BGRA
                *pixel_ptr.add(dst_pixel) = rgba_bytes[src_pixel + 2];     // B
                *pixel_ptr.add(dst_pixel + 1) = rgba_bytes[src_pixel + 1]; // G
                *pixel_ptr.add(dst_pixel + 2) = rgba_bytes[src_pixel];     // R
                *pixel_ptr.add(dst_pixel + 3) = rgba_bytes[src_pixel + 3]; // A
            }
        }

        let _ = GlobalUnlock(hglobal);

        // Set clipboard data (CF_DIB = 8)
        let result = SetClipboardData(CF_DIB, HANDLE(hglobal.0 as *mut std::ffi::c_void));
        if result.is_err() {
            let _ = GlobalFree(hglobal);
            let _ = CloseClipboard();
            return Err("Failed to set clipboard data".to_string());
        }

        let _ = CloseClipboard();
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
pub fn set_image_to_clipboard(_rgba_bytes: &[u8], _width: u32, _height: u32) -> Result<(), String> {
    Err("Windows-specific clipboard not available on this platform".to_string())
}
