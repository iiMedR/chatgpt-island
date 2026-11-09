use crate::text_capture::types::Foreground;
use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
pub fn placement(work: Bounds, scale: f64) -> Bounds {
    let scale = if scale.is_finite() {
        scale.clamp(0.5, 8.0)
    } else {
        1.0
    };
    let margin = ((18.0 * scale) as u32)
        .min(work.width / 8)
        .min(work.height / 8);
    let width = ((520.0 * scale) as u32)
        .min(work.width.saturating_sub(margin * 2))
        .max(1);
    let height = ((660.0 * scale) as u32)
        .min(work.height.saturating_sub(margin * 2))
        .max(1);
    Bounds {
        x: work
            .x
            .saturating_add(work.width.saturating_sub(width + margin) as i32),
        y: work.y.saturating_add(margin as i32),
        width,
        height,
    }
}
#[cfg(windows)]
fn work_area(
    origin: Option<Foreground>,
    window: Option<&WebviewWindow>,
) -> Result<(Bounds, f64), String> {
    use windows::Win32::{
        Foundation::{HWND, POINT},
        Graphics::Gdi::*,
        UI::{HiDpi::*, WindowsAndMessaging::GetCursorPos},
    };
    unsafe {
        let hwnd = origin
            .map(|f| HWND(f.hwnd as *mut _))
            .or_else(|| window.and_then(|w| w.hwnd().ok()));
        let monitor = if let Some(hwnd) = hwnd {
            MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST)
        } else {
            let mut cursor = POINT::default();
            GetCursorPos(&mut cursor).map_err(|_| "Cannot locate active display.")?;
            MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST)
        };
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return Err("Cannot read display working area.".into());
        }
        let mut x = 96;
        let mut y = 96;
        let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y);
        let r = info.rcWork;
        Ok((
            Bounds {
                x: r.left,
                y: r.top,
                width: (r.right - r.left).max(1) as u32,
                height: (r.bottom - r.top).max(1) as u32,
            },
            x as f64 / 96.0,
        ))
    }
}
#[cfg(windows)]
pub fn configure(window: &WebviewWindow, origin: Option<Foreground>) -> Result<(), String> {
    let (work, scale) = work_area(origin, None)?;
    let b = placement(work, scale);
    window
        .set_min_size(Some(PhysicalSize::new(
            ((380.0 * scale) as u32).min(b.width),
            ((420.0 * scale) as u32).min(b.height),
        )))
        .map_err(|e| e.to_string())?;
    window
        .set_size(PhysicalSize::new(b.width, b.height))
        .map_err(|e| e.to_string())?;
    window
        .set_position(PhysicalPosition::new(b.x, b.y))
        .map_err(|e| e.to_string())?;
    // Native rounded-corner preference is Windows 11+. Failure on Windows 10 is benign.
    use windows::Win32::Graphics::Dwm::*;
    if let Ok(hwnd) = window.hwnd() {
        unsafe {
            let preference = DWMWCP_ROUND;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                (&preference as *const DWM_WINDOW_CORNER_PREFERENCE).cast(),
                std::mem::size_of_val(&preference) as u32,
            );
        }
    }
    Ok(())
}
#[cfg(windows)]
pub fn ensure_visible(window: &WebviewWindow) {
    let Ok((work, _)) = work_area(None, Some(window)) else {
        return;
    };
    let (Ok(pos), Ok(size)) = (window.outer_position(), window.outer_size()) else {
        return;
    };
    let width = size.width.min(work.width);
    let height = size.height.min(work.height);
    let x = pos.x.clamp(
        work.x,
        work.x
            .saturating_add(work.width.saturating_sub(width) as i32),
    );
    let y = pos.y.clamp(
        work.y,
        work.y
            .saturating_add(work.height.saturating_sub(height) as i32),
    );
    if width != size.width || height != size.height {
        let _ = window.set_min_size(Some(PhysicalSize::new(380.min(width), 420.min(height))));
        let _ = window.set_size(PhysicalSize::new(width, height));
    }
    if x != pos.x || y != pos.y {
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
}
#[cfg(windows)]
pub fn keyboard_move_resize(window: &WebviewWindow, resize: bool) {
    use windows::Win32::{
        Foundation::{LPARAM, WPARAM},
        UI::WindowsAndMessaging::*,
    };
    if let Ok(hwnd) = window.hwnd() {
        unsafe {
            let _ = PostMessageW(
                Some(hwnd),
                WM_SYSCOMMAND,
                WPARAM(if resize { 0xf000 } else { 0xf010 }),
                LPARAM(0),
            );
        }
    }
}
#[cfg(not(windows))]
pub fn configure(_: &WebviewWindow, _: Option<Foreground>) -> Result<(), String> {
    Err("Windows is required.".into())
}
#[cfg(not(windows))]
pub fn ensure_visible(_: &WebviewWindow) {}
#[cfg(not(windows))]
pub fn keyboard_move_resize(_: &WebviewWindow, _: bool) {}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scaled_negative_and_small_work_areas_are_contained() {
        for (work, scale) in [
            (
                Bounds {
                    x: -1920,
                    y: -400,
                    width: 1920,
                    height: 1040,
                },
                1.5,
            ),
            (
                Bounds {
                    x: 100,
                    y: 40,
                    width: 320,
                    height: 300,
                },
                2.0,
            ),
            (
                Bounds {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1040,
                },
                1.0,
            ),
        ] {
            let b = placement(work, scale);
            assert!(b.x >= work.x && b.y >= work.y);
            assert!(b.x as i64 + b.width as i64 <= work.x as i64 + work.width as i64);
            assert!(b.y as i64 + b.height as i64 <= work.y as i64 + work.height as i64);
        }
    }
}
