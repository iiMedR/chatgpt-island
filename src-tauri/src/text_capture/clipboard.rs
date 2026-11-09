use super::types::*;
use std::time::{Duration, Instant};
use windows::Win32::{
    Foundation::HGLOBAL,
    System::{DataExchange::*, Memory::*},
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::GetWindowThreadProcessId},
};
struct ClipboardGuard;
impl ClipboardGuard {
    fn open() -> Result<Self, CaptureError> {
        unsafe { OpenClipboard(None) }.map_err(|_| CaptureError::ClipboardUnavailable)?;
        Ok(Self)
    }
}
impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}
pub fn may_restore(expected: u32, current: u32, owner_matches: bool) -> bool {
    expected == current && owner_matches
}
fn owner_matches(context: Foreground) -> bool {
    let Ok(owner) = (unsafe { GetClipboardOwner() }) else {
        return false;
    };
    let mut process = 0;
    unsafe {
        GetWindowThreadProcessId(owner, Some(&mut process));
    }
    process == context.process_id
}
fn key(key: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                ..Default::default()
            },
        },
    }
}
fn restore_empty(context: Foreground, expected: u32) -> Result<(), CaptureError> {
    let _guard = ClipboardGuard::open().map_err(|_| CaptureError::RestoreFailed)?;
    if !may_restore(
        expected,
        unsafe { GetClipboardSequenceNumber() },
        owner_matches(context),
    ) {
        return Err(CaptureError::ClipboardChanged);
    }
    unsafe { EmptyClipboard() }.map_err(|_| CaptureError::RestoreFailed)
}
pub fn capture(context: Foreground) -> Result<String, CaptureError> {
    // Existing clipboard formats are never read, snapshotted, cleared or replaced.
    let initial = {
        let _guard = ClipboardGuard::open()?;
        unsafe {
            windows::Win32::Foundation::SetLastError(windows::Win32::Foundation::ERROR_SUCCESS);
        }
        let formats = unsafe { CountClipboardFormats() };
        if formats == 0
            && unsafe { windows::Win32::Foundation::GetLastError() }
                != windows::Win32::Foundation::ERROR_SUCCESS
        {
            return Err(CaptureError::ClipboardUnavailable);
        }
        if formats != 0 {
            return Err(CaptureError::ClipboardNotEmpty);
        }
        unsafe { GetClipboardSequenceNumber() }
    };
    let start = Instant::now();
    loop {
        if !super::foreground_matches(context) {
            return Err(CaptureError::ForegroundChanged);
        }
        let held = [VK_MENU, VK_SHIFT, VK_CONTROL, VK_LWIN, VK_RWIN, VK_E]
            .iter()
            .any(|key| unsafe { GetAsyncKeyState(key.0 as i32) } < 0);
        if !held {
            break;
        }
        if start.elapsed() >= Duration::from_millis(300) {
            return Err(CaptureError::CopyFailed);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    if unsafe { GetClipboardSequenceNumber() } != initial || !super::foreground_matches(context) {
        return Err(CaptureError::ForegroundChanged);
    }
    let inputs = [
        key(VK_CONTROL, false),
        key(VK_C, false),
        key(VK_C, true),
        key(VK_CONTROL, true),
    ];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        // Only release keys we injected after verifying they were physically up.
        let _ = unsafe {
            SendInput(
                &[key(VK_C, true), key(VK_CONTROL, true)],
                std::mem::size_of::<INPUT>() as i32,
            )
        };
        return Err(CaptureError::CopyFailed);
    }
    let deadline = Instant::now() + Duration::from_millis(300);
    let sequence = loop {
        let now = unsafe { GetClipboardSequenceNumber() };
        if now != initial {
            break now;
        }
        if Instant::now() >= deadline {
            return Err(CaptureError::CopyFailed);
        }
        if !super::foreground_matches(context) {
            return Err(CaptureError::ForegroundChanged);
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    if !owner_matches(context) {
        return Err(CaptureError::ClipboardChanged);
    }
    let text = (|| {
        let _guard = ClipboardGuard::open()?;
        if unsafe { GetClipboardSequenceNumber() } != sequence {
            return Err(CaptureError::ClipboardChanged);
        }
        if !super::foreground_matches(context) {
            return Err(CaptureError::ForegroundChanged);
        }
        let data = unsafe { GetClipboardData(13) }.map_err(|_| CaptureError::CopyFailed)?;
        let global = HGLOBAL(data.0);
        let bytes = unsafe { GlobalSize(global) };
        if bytes == 0 || bytes > 1_048_576 || bytes % 2 != 0 {
            return Err(CaptureError::CopyFailed);
        }
        let pointer = unsafe { GlobalLock(global) };
        if pointer.is_null() {
            return Err(CaptureError::CopyFailed);
        }
        // SAFETY: GlobalSize bounds this clipboard-owned Unicode buffer. Never free it.
        let units = unsafe { std::slice::from_raw_parts(pointer.cast::<u16>(), bytes / 2) };
        let end = units.iter().position(|value| *value == 0);
        let result = end.ok_or(CaptureError::CopyFailed).and_then(|end| {
            String::from_utf16(&units[..end]).map_err(|_| CaptureError::CopyFailed)
        });
        unsafe {
            let _ = GlobalUnlock(global);
        }
        if unsafe { GetClipboardSequenceNumber() } != sequence {
            return Err(CaptureError::ClipboardChanged);
        }
        result
    })();
    // Clear only our source-owned, unchanged copy of an initially empty clipboard.
    restore_empty(context, sequence)?;
    text
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_clipboard_updates_or_owners_are_never_overwritten() {
        assert!(may_restore(7, 7, true));
        assert!(!may_restore(7, 8, true));
        assert!(!may_restore(7, 7, false));
    }
}
