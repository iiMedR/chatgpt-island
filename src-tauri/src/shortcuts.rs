use crate::lifecycle::{Controller, Request};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[derive(Default)]
pub struct PressLatch(AtomicBool);
impl PressLatch {
    pub fn event(&self, pressed: bool) -> bool {
        if pressed {
            !self.0.swap(true, Ordering::SeqCst)
        } else {
            self.0.store(false, Ordering::SeqCst);
            false
        }
    }
}
pub fn shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyE)
}
pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    let latch = PressLatch::default();
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app, key, event| {
            if key == &shortcut() && latch.event(event.state() == ShortcutState::Pressed) {
                let controller = app.state::<Controller>();
                let closed = controller
                    .shared
                    .lock()
                    .map(|core| core.model.phase == crate::lifecycle::Phase::Closed)
                    .unwrap_or(false);
                // Snapshot foreground context synchronously before any popup focus.
                let context = if closed {
                    crate::text_capture::foreground()
                } else {
                    None
                };
                let _ = controller.send(Request::Toggle(context));
            }
        })
        .build()
}
#[derive(Default)]
pub struct Registration {
    pub registered: bool,
}
impl Registration {
    pub fn start(&mut self, operation: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
        if !self.registered {
            operation()?;
            self.registered = true;
        }
        Ok(())
    }
    pub fn stop(&mut self, operation: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
        if self.registered {
            operation()?;
            self.registered = false;
        }
        Ok(())
    }
}
pub fn register(app: &AppHandle) {
    let mut registration = Registration::default();
    let result = registration.start(|| {
        app.global_shortcut()
            .register(shortcut())
            .map_err(|e| e.to_string())
    });
    let failed = result.is_err();
    if let Ok(mut core) = app.state::<Controller>().shared.lock() {
        core.registration = registration;
        if let Err(error) = result {
            core.error = Some("Alt + Shift + E is unavailable. Another application may own it. Use the tray menu, or quit the conflicting app and restart Island.".into());
            eprintln!(
                "Global shortcut registration failed; tray controls remain available: {error}"
            );
        }
    }
    if failed {
        let _ = app.state::<Controller>().send(Request::Settings);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_toggle_per_physical_press_and_release() {
        let latch = PressLatch::default();
        assert!(latch.event(true));
        assert!(!latch.event(true));
        assert!(!latch.event(true));
        assert!(!latch.event(false));
        assert!(latch.event(true));
    }
    #[test]
    fn shortcut_is_the_approved_combination() {
        assert_eq!(shortcut(), "Alt+Shift+E".parse::<Shortcut>().unwrap());
    }
    #[test]
    fn registration_failure_retry_and_cleanup_are_explicit_and_idempotent() {
        let mut registration = Registration::default();
        assert!(registration.start(|| Err("conflict".into())).is_err());
        assert!(!registration.registered);
        registration.start(|| Ok(())).unwrap();
        assert!(registration.registered);
        registration
            .start(|| panic!("must not register twice"))
            .unwrap();
        assert!(registration.stop(|| Err("cleanup failed".into())).is_err());
        assert!(registration.registered);
        registration.stop(|| Ok(())).unwrap();
        assert!(!registration.registered);
        registration.stop(|| panic!("already released")).unwrap();
    }
}
