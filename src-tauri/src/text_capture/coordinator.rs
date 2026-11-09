use super::{types::*, validation};
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime},
};
pub const CAPTURE_TIMEOUT: Duration = Duration::from_millis(1500);
const MAX_PROTOCOL_BYTES: u64 = 96 * 1024;

pub enum Attempt {
    Text(String),
    None,
    ClipboardCandidate,
}
pub trait CaptureBackend {
    fn uia(&mut self) -> Result<Attempt, CaptureError>;
    fn clipboard(&mut self) -> Result<String, CaptureError>;
}
pub fn orchestrate(backend: &mut impl CaptureBackend, fallback: bool) -> WireReport {
    let mut clipboard_attempted = false;
    let result = (|| {
        let (text, method) = match backend.uia()? {
            Attempt::Text(text) => (text, CaptureMethod::UiAutomation),
            Attempt::None => return Ok(None),
            Attempt::ClipboardCandidate if fallback => {
                clipboard_attempted = true;
                (backend.clipboard()?, CaptureMethod::ClipboardFallback)
            }
            Attempt::ClipboardCandidate => return Err(CaptureError::ClipboardDisabled),
        };
        let text = validation::validate(&text).map_err(CaptureError::Invalid)?;
        Ok(Some(WireSelection { text, method }))
    })();
    let clipboard_warning = clipboard_attempted
        && matches!(
            &result,
            Err(CaptureError::RestoreFailed
                | CaptureError::CopyFailed
                | CaptureError::ForegroundChanged
                | CaptureError::ClipboardUnavailable
                | CaptureError::ClipboardChanged)
        );
    WireReport {
        result,
        clipboard_warning,
    }
}

pub fn capture(context: Option<Foreground>, fallback: bool) -> CaptureReport {
    let start = Instant::now();
    let result = context
        .ok_or(CaptureError::Unavailable)
        .and_then(|context| capture_process(context, fallback));
    let (selection, error, clipboard_warning) = match result {
        Ok(report) => match report.result {
            Ok(value) => match value
                .map(|wire| {
                    validation::validate(&wire.text).map(|text| CapturedText {
                        text,
                        captured_at: SystemTime::now(),
                        method: wire.method,
                    })
                })
                .transpose()
            {
                Ok(selection) => (selection, None, report.clipboard_warning),
                Err(error) => (
                    None,
                    Some(CaptureError::Invalid(error)),
                    report.clipboard_warning,
                ),
            },
            Err(error) => (None, Some(error), report.clipboard_warning),
        },
        Err(error) => {
            // A transport/focus failure can lose the helper's restoration result.
            let warning = fallback && context.is_some();
            (None, Some(error), warning)
        }
    };
    CaptureReport {
        selection,
        error,
        clipboard_warning,
        elapsed_ms: start.elapsed().as_millis().min(u64::MAX as u128) as u64,
    }
}
fn capture_process(context: Foreground, fallback: bool) -> Result<WireReport, CaptureError> {
    let executable = std::env::current_exe().map_err(|_| CaptureError::Unavailable)?;
    let mut command = Command::new(executable);
    command
        .arg("--island-selection-helper")
        .arg(context.hwnd.to_string())
        .arg(context.process_id.to_string())
        .arg(if fallback { "copy-enabled" } else { "uia-only" });
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let bytes = run_bounded(&mut command)?;
    let report = serde_json::from_slice(&bytes).map_err(|_| CaptureError::Unavailable)?;
    if !super::foreground_matches(context) {
        return Err(CaptureError::ForegroundChanged);
    }
    Ok(report)
}

// The helper does not spawn descendants, so killing it also closes the pipe writer.
fn run_bounded(command: &mut Command) -> Result<Vec<u8>, CaptureError> {
    let mut child = command.spawn().map_err(|_| CaptureError::Unavailable)?;
    let stdout = child.stdout.take().ok_or(CaptureError::Unavailable)?;
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stdout.take(MAX_PROTOCOL_BYTES + 1).read_to_end(&mut bytes);
        (result, bytes)
    });
    let start = Instant::now();
    let completed = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.success(),
            Ok(None) if start.elapsed() < CAPTURE_TIMEOUT => {
                std::thread::sleep(Duration::from_millis(10))
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                return Err(CaptureError::Timeout);
            }
        }
    };
    let (read, bytes) = reader.join().map_err(|_| CaptureError::Unavailable)?;
    if !completed || read.is_err() || bytes.len() as u64 > MAX_PROTOCOL_BYTES {
        return Err(CaptureError::Unavailable);
    }
    Ok(bytes)
}

pub fn run_helper_if_requested() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some("--island-selection-helper") {
        return false;
    }
    #[cfg(windows)]
    unsafe {
        use windows::Win32::{
            Storage::FileSystem::{FILE_TYPE_PIPE, GetFileType},
            System::Console::{GetStdHandle, STD_OUTPUT_HANDLE},
        };
        // Refuse terminal/file output. Normal startup never exports selected text.
        if !GetStdHandle(STD_OUTPUT_HANDLE)
            .is_ok_and(|handle| GetFileType(handle) == FILE_TYPE_PIPE)
        {
            return true;
        }
    }
    let report = match (
        args.get(2).and_then(|s| s.parse::<i64>().ok()),
        args.get(3).and_then(|s| s.parse::<u32>().ok()),
    ) {
        (Some(hwnd), Some(process_id)) => super::run_native(
            Foreground { hwnd, process_id },
            args.get(4).map(String::as_str) == Some("copy-enabled"),
        ),
        _ => WireReport {
            result: Err(CaptureError::Unavailable),
            clipboard_warning: false,
        },
    };
    // Anonymous stdout pipe is the sole data channel to the parent; stderr is null.
    if let Ok(bytes) = serde_json::to_vec(&report) {
        let _ = std::io::stdout().write_all(&bytes);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(windows)]
    fn stalled_child_is_terminated_within_bounded_time() {
        let mut command = Command::new("powershell.exe");
        command
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 10",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let start = Instant::now();
        assert!(matches!(
            run_bounded(&mut command),
            Err(CaptureError::Timeout)
        ));
        assert!(start.elapsed() < Duration::from_secs(5));
    }
    struct Mock {
        attempt: Result<Attempt, CaptureError>,
        copied: Result<String, CaptureError>,
        copies: usize,
    }
    impl CaptureBackend for Mock {
        fn uia(&mut self) -> Result<Attempt, CaptureError> {
            std::mem::replace(&mut self.attempt, Ok(Attempt::None))
        }
        fn clipboard(&mut self) -> Result<String, CaptureError> {
            self.copies += 1;
            self.copied.clone()
        }
    }
    fn mock(attempt: Result<Attempt, CaptureError>) -> Mock {
        Mock {
            attempt,
            copied: Ok("copied sample".into()),
            copies: 0,
        }
    }
    #[test]
    fn uia_success_and_no_selection_never_use_clipboard() {
        let mut backend = mock(Ok(Attempt::Text(" selection ".into())));
        assert_eq!(
            orchestrate(&mut backend, true)
                .result
                .unwrap()
                .unwrap()
                .text,
            "selection"
        );
        assert_eq!(backend.copies, 0);
        let mut backend = mock(Ok(Attempt::None));
        assert!(orchestrate(&mut backend, true).result.unwrap().is_none());
        assert_eq!(backend.copies, 0);
    }
    #[test]
    fn fallback_requires_opt_in_and_explicit_selection_evidence() {
        let mut backend = mock(Ok(Attempt::ClipboardCandidate));
        assert!(matches!(
            orchestrate(&mut backend, false).result,
            Err(CaptureError::ClipboardDisabled)
        ));
        assert_eq!(backend.copies, 0);
        let mut backend = mock(Ok(Attempt::ClipboardCandidate));
        assert_eq!(
            orchestrate(&mut backend, true)
                .result
                .unwrap()
                .unwrap()
                .method,
            CaptureMethod::ClipboardFallback
        );
        assert_eq!(backend.copies, 1);
    }
    #[test]
    fn protected_unsupported_changed_and_timeout_never_fall_back() {
        for error in [
            CaptureError::ProtectedInput,
            CaptureError::Unavailable,
            CaptureError::ForegroundChanged,
            CaptureError::Timeout,
        ] {
            let mut backend = mock(Err(error.clone()));
            assert!(matches!(orchestrate(&mut backend, true).result, Err(e) if e == error));
            assert_eq!(backend.copies, 0);
        }
    }
    #[test]
    fn clipboard_failures_and_restoration_warning_are_reported_without_text() {
        for error in [
            CaptureError::CopyFailed,
            CaptureError::ClipboardNotEmpty,
            CaptureError::ClipboardUnavailable,
            CaptureError::ClipboardChanged,
            CaptureError::RestoreFailed,
        ] {
            let mut backend = mock(Ok(Attempt::ClipboardCandidate));
            backend.copied = Err(error.clone());
            let report = orchestrate(&mut backend, true);
            assert_eq!(
                report.clipboard_warning,
                error != CaptureError::ClipboardNotEmpty
            );
            assert!(matches!(report.result, Err(e) if e == error));
        }
    }
}
