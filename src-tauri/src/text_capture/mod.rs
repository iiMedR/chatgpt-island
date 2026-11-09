#[cfg(windows)]
mod clipboard;
pub mod coordinator;
pub mod types;
#[cfg(windows)]
mod uia;
pub mod validation;
use types::*;

pub fn foreground() -> Option<Foreground> {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId,
        };
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() || !default_desktop() {
            return None;
        }
        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        if process_id == 0 || process_id == std::process::id() {
            return None;
        }
        Some(Foreground {
            hwnd: hwnd.0 as i64,
            process_id,
        })
    }
    #[cfg(not(windows))]
    {
        None
    }
}
pub fn foreground_matches(context: Foreground) -> bool {
    foreground().is_some_and(|now| now.hwnd == context.hwnd && now.process_id == context.process_id)
}
#[cfg(windows)]
fn default_desktop() -> bool {
    use windows::Win32::System::StationsAndDesktops::*;
    unsafe {
        let Ok(desktop) = OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_READOBJECTS)
        else {
            return false;
        };
        let mut name = [0u16; 64];
        let mut needed = 0;
        let read = GetUserObjectInformationW(
            windows::Win32::Foundation::HANDLE(desktop.0),
            UOI_NAME,
            Some(name.as_mut_ptr().cast()),
            std::mem::size_of_val(&name) as u32,
            Some(&mut needed),
        );
        let _ = CloseDesktop(desktop);
        read.is_ok()
            && String::from_utf16_lossy(
                &name[..name.iter().position(|c| *c == 0).unwrap_or(name.len())],
            )
            .eq_ignore_ascii_case("default")
    }
}
#[cfg(not(windows))]
fn default_desktop() -> bool {
    false
}
fn run_native(context: Foreground, fallback: bool) -> WireReport {
    #[cfg(windows)]
    {
        struct Backend(Foreground);
        impl coordinator::CaptureBackend for Backend {
            fn uia(&mut self) -> Result<coordinator::Attempt, CaptureError> {
                if !default_desktop() {
                    return Err(CaptureError::ProtectedInput);
                }
                uia::capture(self.0)
            }
            fn clipboard(&mut self) -> Result<String, CaptureError> {
                if !default_desktop() {
                    return Err(CaptureError::ProtectedInput);
                }
                clipboard::capture(self.0)
            }
        }
        coordinator::orchestrate(&mut Backend(context), fallback)
    }
    #[cfg(not(windows))]
    {
        let _ = (context, fallback);
        WireReport {
            result: Err(CaptureError::Unavailable),
            clipboard_warning: false,
        }
    }
}
