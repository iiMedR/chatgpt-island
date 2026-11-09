use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureMethod {
    UiAutomation,
    ClipboardFallback,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationError {
    Empty,
    Whitespace,
    TooLong,
    ContainsNul,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureError {
    Unavailable,
    ProtectedInput,
    ForegroundChanged,
    Timeout,
    Invalid(ValidationError),
    ClipboardDisabled,
    ClipboardNotEmpty,
    ClipboardUnavailable,
    CopyFailed,
    ClipboardChanged,
    RestoreFailed,
}
impl CaptureError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::ProtectedInput => {
                "Selection capture was skipped for protected or inaccessible input."
            }
            Self::ForegroundChanged => {
                "Selection capture was canceled because the foreground application changed."
            }
            Self::Timeout => {
                "Selection capture timed out; the private browser is available without selection context."
            }
            Self::Invalid(ValidationError::TooLong) => {
                "The selection exceeds 10,000 Unicode characters; it was not truncated or captured."
            }
            Self::Invalid(_) => "The selected text was empty or invalid.",
            Self::ClipboardDisabled => {
                "UI Automation could not read the selection; optional copy fallback is disabled."
            }
            Self::ClipboardNotEmpty => {
                "Copy fallback was skipped to preserve the existing clipboard data."
            }
            Self::ClipboardChanged => {
                "The clipboard changed during capture; the newer clipboard content was left untouched."
            }
            Self::RestoreFailed => {
                "Copy fallback could not safely restore the initially empty clipboard. It may contain the copied selection."
            }
            Self::ClipboardUnavailable => {
                "The clipboard is unavailable; no selection context was captured."
            }
            Self::CopyFailed => "Copy fallback did not provide a valid selection.",
            Self::Unavailable => {
                "Selected text is unavailable from this application's accessible focused control."
            }
        }
    }
}
// This type is intentionally not Debug/Serialize: selected text must not enter logs/settings.
pub struct CapturedText {
    pub text: String,
    pub captured_at: SystemTime,
    pub method: CaptureMethod,
}
#[derive(Default)]
pub struct PendingSelection {
    session_id: Option<String>,
    value: Option<CapturedText>,
}
impl PendingSelection {
    pub fn replace(&mut self, session_id: &str, value: Option<CapturedText>) {
        self.clear();
        self.session_id = Some(session_id.into());
        self.value = value;
    }
    pub fn for_session(&self, session_id: &str) -> Option<&CapturedText> {
        if self.session_id.as_deref() == Some(session_id) {
            self.value.as_ref()
        } else {
            None
        }
    }
    pub fn clear(&mut self) {
        self.value = None;
        self.session_id = None;
    }
    pub fn clear_session(&mut self, session_id: &str) {
        if self.session_id.as_deref() == Some(session_id) {
            self.clear();
        }
    }
}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Foreground {
    pub hwnd: i64,
    pub process_id: u32,
}
// Private anonymous-pipe protocol only; never printed by the main application.
#[derive(Serialize, Deserialize)]
pub struct WireSelection {
    pub text: String,
    pub method: CaptureMethod,
}
#[derive(Serialize, Deserialize)]
pub struct WireReport {
    pub result: Result<Option<WireSelection>, CaptureError>,
    pub clipboard_warning: bool,
}
pub struct CaptureReport {
    pub selection: Option<CapturedText>,
    pub error: Option<CaptureError>,
    pub clipboard_warning: bool,
    pub elapsed_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pending_selection_is_session_scoped_replaced_and_released() {
        let mut pending = PendingSelection::default();
        let make = |text: &str| CapturedText {
            text: text.into(),
            captured_at: SystemTime::now(),
            method: CaptureMethod::UiAutomation,
        };
        pending.replace("one", Some(make("sample one")));
        assert_eq!(pending.for_session("one").unwrap().text, "sample one");
        assert!(pending.for_session("stale").is_none());
        pending.clear_session("stale");
        assert!(pending.for_session("one").is_some());
        pending.replace("two", Some(make("sample two")));
        assert!(pending.for_session("one").is_none());
        assert_eq!(pending.for_session("two").unwrap().text, "sample two");
        pending.clear_session("two");
        assert!(pending.for_session("two").is_none());
        pending.replace("three", None);
        assert!(pending.for_session("three").is_none());
    }
}
