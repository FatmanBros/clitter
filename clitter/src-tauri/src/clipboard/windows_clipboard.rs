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
pub fn set_image_to_clipboard(rgba_bytes: &[u8], width: u32, height: u32) -> Result<(), String> {
    unsafe {
        // Open clipboard
        if !OpenClipboard(HWND::default()).as_bool() {
            return Err("Failed to open clipboard".to_string());
        }

        // Empty clipboard
        if !EmptyClipboard().as_bool() {
            CloseClipboard();
            return Err("Failed to empty clipboard".to_string());
        }

        // Create DIB (Device Independent Bitmap)
        // DIB uses BGRA format (bottom-up by default)
        let row_size = ((width * 32 + 31) / 32) * 4; // 32-bit aligned rows
        let pixel_data_size = row_size * height;
        let header_size = std::mem::size_of::<BITMAPINFOHEADER>();
        let total_size = header_size + pixel_data_size as usize;

        // Allocate global memory
        let hglobal = GlobalAlloc(GMEM_MOVEABLE, total_size);
        if hglobal.is_err() {
            CloseClipboard();
            return Err("Failed to allocate global memory".to_string());
        }
        let hglobal = hglobal.unwrap();

        let ptr = GlobalLock(hglobal);
        if ptr.is_null() {
            GlobalFree(hglobal);
            CloseClipboard();
            return Err("Failed to lock global memory".to_string());
        }

        // Write BITMAPINFOHEADER
        let header = BITMAPINFOHEADER {
            biSize: header_size as u32,
            biWidth: width as i32,
            biHeight: height as i32, // Positive = bottom-up DIB
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
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

        GlobalUnlock(hglobal);

        // Set clipboard data
        let result = SetClipboardData(CF_DIB.0 as u32, HANDLE(hglobal.0));
        if result.is_err() {
            GlobalFree(hglobal);
            CloseClipboard();
            return Err("Failed to set clipboard data".to_string());
        }

        CloseClipboard();
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
pub fn set_image_to_clipboard(_rgba_bytes: &[u8], _width: u32, _height: u32) -> Result<(), String> {
    Err("Windows-specific clipboard not available on this platform".to_string())
}
