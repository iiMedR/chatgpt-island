use super::{coordinator::Attempt, types::*};
use windows::{
    Win32::{
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
            CoUninitialize,
        },
        UI::Accessibility::*,
    },
    core::Interface,
};
struct ComGuard;
impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

pub fn capture(context: Foreground) -> Result<Attempt, CaptureError> {
    // This helper's only thread is an MTA, owns no windows, and is process-bounded.
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
        .ok()
        .map_err(|_| CaptureError::Unavailable)?;
    let _com = ComGuard;
    unsafe {
        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)
                .map_err(|_| CaptureError::Unavailable)?;
        if let Ok(timing) = automation.cast::<IUIAutomation2>() {
            let _ = timing.SetConnectionTimeout(250);
            let _ = timing.SetTransactionTimeout(500);
        }
        if !super::foreground_matches(context) || !super::default_desktop() {
            return Err(CaptureError::ForegroundChanged);
        }
        let mut element = automation
            .GetFocusedElement()
            .map_err(|_| CaptureError::Unavailable)?;
        let walker = automation
            .ControlViewWalker()
            .map_err(|_| CaptureError::Unavailable)?;
        // Ascend the focused control's ancestors only; never scan the document/desktop.
        for _ in 0..8 {
            if !super::foreground_matches(context) {
                return Err(CaptureError::ForegroundChanged);
            }
            if element
                .CurrentProcessId()
                .map_err(|_| CaptureError::ProtectedInput)? as u32
                != context.process_id
            {
                return Err(CaptureError::ForegroundChanged);
            }
            if element
                .CurrentIsPassword()
                .map_err(|_| CaptureError::ProtectedInput)?
                .as_bool()
            {
                return Err(CaptureError::ProtectedInput);
            }
            if let Ok(pattern) =
                element.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)
            {
                let ranges = pattern
                    .GetSelection()
                    .map_err(|_| CaptureError::Unavailable)?;
                let count = ranges.Length().map_err(|_| CaptureError::Unavailable)?;
                if !(0..=32).contains(&count) {
                    return Err(CaptureError::Unavailable);
                }
                let mut text = String::new();
                for index in 0..count {
                    let range = ranges
                        .GetElement(index)
                        .map_err(|_| CaptureError::Unavailable)?;
                    if range
                        .CompareEndpoints(
                            TextPatternRangeEndpoint_Start,
                            &range,
                            TextPatternRangeEndpoint_End,
                        )
                        .map_err(|_| CaptureError::Unavailable)?
                        == 0
                    {
                        continue;
                    }
                    // Bounded selected-range reads, never DocumentRange or GetText(-1).
                    let selected = match range.GetText(20_002) {
                        Ok(value) => value,
                        Err(_) => return Ok(Attempt::ClipboardCandidate),
                    };
                    if selected.len() >= 20_002 {
                        return Err(CaptureError::Invalid(ValidationError::TooLong));
                    }
                    let selected =
                        String::from_utf16(&selected).map_err(|_| CaptureError::Unavailable)?;
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(&selected);
                    if text.chars().count() > super::validation::MAX_CHARACTERS {
                        return Err(CaptureError::Invalid(ValidationError::TooLong));
                    }
                }
                if !super::foreground_matches(context) {
                    return Err(CaptureError::ForegroundChanged);
                }
                return if text.is_empty() {
                    Ok(Attempt::None)
                } else {
                    Ok(Attempt::Text(text))
                };
            }
            let handle = element
                .CurrentNativeWindowHandle()
                .map_err(|_| CaptureError::Unavailable)?;
            if handle.0 as i64 == context.hwnd {
                break;
            }
            element = walker
                .GetParentElement(&element)
                .map_err(|_| CaptureError::Unavailable)?;
        }
    }
    Err(CaptureError::Unavailable)
}
