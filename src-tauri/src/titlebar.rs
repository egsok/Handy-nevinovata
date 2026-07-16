//! Native title bar tinting.
//!
//! The window frame is drawn by the OS, not the webview, so a themed app with a
//! stock frame wears a white (or OS-grey) strip above its own palette. Windows
//! 11 lets us hand DWM the caption/text/border colours, so the frame can carry
//! the ink wall or the kraft sheet like the rest of the window.
//!
//! Elsewhere (macOS/Linux, and Windows 10, where the attributes did not exist)
//! this is a no-op and the platform's own frame stands.

use tauri::WebviewWindow;

/// Palette for the frame, mirroring `src/styles/theme.css`. Kept here rather
/// than read from CSS because DWM needs the colours before the webview paints.
#[cfg(windows)]
mod palette {
    /// COLORREF is 0x00BBGGRR — byte order reversed from CSS hex.
    const fn colorref(r: u8, g: u8, b: u8) -> u32 {
        (b as u32) << 16 | (g as u32) << 8 | (r as u32)
    }

    /// --dark-color-background #1e1640
    pub const WALL: u32 = colorref(0x1e, 0x16, 0x40);
    /// --dark-color-text #ddd4bd
    pub const CREAM: u32 = colorref(0xdd, 0xd4, 0xbd);
    /// --light-color-background #e9dfc8
    pub const KRAFT: u32 = colorref(0xe9, 0xdf, 0xc8);
    /// --light-color-text #2c1a72
    pub const VIOLET: u32 = colorref(0x2c, 0x1a, 0x72);
}

/// Tint the window's native frame to match the in-app theme.
#[cfg(windows)]
pub fn apply(window: &WebviewWindow, dark: bool) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR,
    };

    let hwnd = match window.hwnd() {
        Ok(hwnd) => HWND(hwnd.0 as _),
        Err(e) => {
            log::warn!("titlebar: no HWND for window: {e}");
            return;
        }
    };

    let (caption, text) = if dark {
        (palette::WALL, palette::CREAM)
    } else {
        (palette::KRAFT, palette::VIOLET)
    };

    // Pre-22000 builds reject these attributes; that is expected, not an error
    // worth surfacing — the window simply keeps the stock frame.
    for (attr, value) in [
        (DWMWA_CAPTION_COLOR, caption),
        (DWMWA_TEXT_COLOR, text),
        (DWMWA_BORDER_COLOR, caption),
    ] {
        let result = unsafe {
            DwmSetWindowAttribute(
                hwnd,
                attr,
                &value as *const u32 as *const std::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            )
        };
        if let Err(e) = result {
            log::debug!("titlebar: DWM attribute {attr:?} not applied: {e}");
            return;
        }
    }
}

#[cfg(not(windows))]
pub fn apply(_window: &WebviewWindow, _dark: bool) {}

/// Called by the frontend on boot and whenever the resolved theme flips, since
/// only the webview knows what `system` currently resolves to. Always targets
/// the main window explicitly: the overlay is frameless and calls this too.
#[tauri::command]
#[specta::specta]
pub fn set_titlebar_theme(app: tauri::AppHandle, dark: bool) -> Result<(), String> {
    use tauri::Manager;
    if let Some(main) = app.get_webview_window("main") {
        apply(&main, dark);
    }
    Ok(())
}
