use crate::text_capture::{
    self,
    types::{Foreground, PendingSelection},
};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Phase {
    #[default]
    Closed,
    Opening,
    Open,
    AwaitingCloseConfirmation,
    Closing,
}
#[derive(Clone, Debug)]
pub struct Profile {
    pub id: String,
    pub path: PathBuf,
}
#[derive(Default)]
pub struct Model {
    pub phase: Phase,
    pub profile: Option<Profile>,
    pub quit_pending: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    None,
    Confirm,
    FocusConfirmation,
    Destroy,
    Quit,
}
impl Model {
    pub fn begin(&mut self, profile: Profile) -> bool {
        if self.phase != Phase::Closed {
            return false;
        }
        self.profile = Some(profile);
        self.phase = Phase::Opening;
        true
    }
    pub fn opened(&mut self, id: &str) {
        if self.matches(id) && self.phase == Phase::Opening {
            self.phase = Phase::Open;
        }
    }
    pub fn matches(&self, id: &str) -> bool {
        self.profile.as_ref().is_some_and(|p| p.id == id)
    }
    pub fn close(&mut self, quit: bool, skip: bool) -> Effect {
        self.quit_pending |= quit;
        match self.phase {
            Phase::Closed => {
                if quit {
                    Effect::Quit
                } else {
                    Effect::None
                }
            }
            Phase::Open => {
                if skip {
                    self.phase = Phase::Closing;
                    Effect::Destroy
                } else {
                    self.phase = Phase::AwaitingCloseConfirmation;
                    Effect::Confirm
                }
            }
            Phase::AwaitingCloseConfirmation => Effect::FocusConfirmation,
            Phase::Opening | Phase::Closing => Effect::None,
        }
    }
    pub fn cancel(&mut self) {
        if self.phase == Phase::AwaitingCloseConfirmation {
            self.phase = Phase::Open;
            self.quit_pending = false;
        }
    }
    pub fn confirm(&mut self) -> Effect {
        if self.phase == Phase::AwaitingCloseConfirmation {
            self.phase = Phase::Closing;
            Effect::Destroy
        } else {
            Effect::None
        }
    }
    pub fn destroyed(&mut self, id: &str) -> Effect {
        if !self.matches(id) {
            return Effect::None;
        }
        self.profile = None;
        self.phase = Phase::Closed;
        if std::mem::take(&mut self.quit_pending) {
            Effect::Quit
        } else {
            Effect::None
        }
    }
    pub fn destroy_failed(&mut self) {
        self.phase = Phase::Open;
        self.quit_pending = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn profile(id: &str) -> Profile {
        Profile {
            id: id.into(),
            path: id.into(),
        }
    }
    fn open() -> Model {
        let mut m = Model::default();
        assert!(m.begin(profile("one")));
        m.opened("one");
        m
    }
    #[test]
    fn starts_closed_and_enforces_single_popup_during_all_transitions() {
        let mut m = Model::default();
        assert_eq!(m.phase, Phase::Closed);
        assert!(m.profile.is_none());
        assert!(m.begin(profile("one")));
        assert!(!m.begin(profile("two")));
        assert_eq!(m.close(false, false), Effect::None);
        m.opened("stale");
        assert_eq!(m.phase, Phase::Opening);
        m.opened("one");
        assert!(!m.begin(profile("two")));
        assert_eq!(m.close(false, false), Effect::Confirm);
        assert_eq!(m.close(false, false), Effect::FocusConfirmation);
        assert_eq!(m.confirm(), Effect::Destroy);
        assert_eq!(m.close(false, false), Effect::None);
        assert!(!m.begin(profile("two")));
        m.destroyed("stale");
        assert_eq!(m.phase, Phase::Closing);
        m.destroyed("one");
        assert!(m.profile.is_none());
        assert!(m.begin(profile("two")));
    }
    #[test]
    fn cancel_preserves_session_and_cancels_quit() {
        let mut m = open();
        assert_eq!(m.close(true, false), Effect::Confirm);
        m.cancel();
        assert_eq!(m.phase, Phase::Open);
        assert!(!m.quit_pending);
        assert!(m.matches("one"));
        assert_eq!(m.close(false, false), Effect::Confirm);
    }
    #[test]
    fn quit_waits_for_confirmation_and_actual_destruction() {
        let mut m = open();
        m.close(true, false);
        assert_eq!(m.confirm(), Effect::Destroy);
        assert_eq!(m.destroyed("one"), Effect::Quit);
        let mut closed = Model::default();
        assert_eq!(closed.close(true, false), Effect::Quit);
    }
    #[test]
    fn warning_preference_skips_dialog_and_failure_allows_retry() {
        let mut m = open();
        assert_eq!(m.close(false, true), Effect::Destroy);
        m.destroy_failed();
        assert_eq!(m.phase, Phase::Open);
        assert_eq!(m.close(false, false), Effect::Confirm);
    }
    #[test]
    fn creation_failure_releases_reservation_and_can_be_retried() {
        let mut m = Model::default();
        m.begin(profile("failed"));
        m.destroyed("failed");
        assert_eq!(m.phase, Phase::Closed);
        assert!(m.begin(profile("retry")));
    }
}
use crate::{
    confirmation, popup,
    settings::{self, Settings},
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub struct Core {
    pub model: Model,
    pub settings: Settings,
    pub registration: crate::shortcuts::Registration,
    pub error: Option<String>,
    pub pending_selection: PendingSelection,
    pub capture_status: String,
    pub handoff: Option<crate::handoff::Handoff>,
}
impl Core {
    fn destroyed(&mut self, id: &str) -> Effect {
        self.pending_selection.clear_session(id);
        if self.handoff.as_ref().is_some_and(|h| h.session == id)
            && let Some(mut h) = self.handoff.take()
        {
            h.cancel();
        }
        if self.model.matches(id) {
            self.capture_status = "No pending selection; session closed.".into();
        }
        self.model.destroyed(id)
    }
}
pub struct Controller {
    sender: mpsc::Sender<Request>,
    pub shared: Arc<Mutex<Core>>,
    pub exiting: Arc<AtomicBool>,
}
pub enum Request {
    Open,
    Toggle(Option<Foreground>),
    OpenSelected(Option<Foreground>),
    Close,
    Quit,
    Settings,
    HandoffFailed(String),
    MoveResize(bool),
    Destroyed(String),
    Decision {
        session_id: String,
        confirm: bool,
        skip: bool,
        reply: Option<mpsc::Sender<Result<(), String>>>,
    },
    Reset(mpsc::Sender<Result<(), String>>),
    ClipboardPreference(bool, mpsc::Sender<Result<(), String>>),
}
#[derive(serde::Serialize)]
pub struct Snapshot {
    phase: Phase,
    session_id: Option<String>,
    skip_close_confirmation: bool,
    clipboard_fallback_enabled: bool,
    capture_status: String,
    shortcut_registered: bool,
    shortcut: &'static str,
    error: Option<String>,
}
impl Controller {
    pub fn install(app: &AppHandle) -> Result<(), String> {
        let path = app
            .path()
            .app_config_dir()
            .map_err(|e| e.to_string())?
            .join("settings.json");
        let (settings, error) = settings::load(&path);
        let shared = Arc::new(Mutex::new(Core {
            model: Model::default(),
            settings,
            registration: crate::shortcuts::Registration::default(),
            error,
            pending_selection: PendingSelection::default(),
            capture_status: "No selection context.".into(),
            handoff: None,
        }));
        let exiting = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = mpsc::channel();
        app.manage(Self {
            sender,
            shared: shared.clone(),
            exiting: exiting.clone(),
        });
        let handle = app.clone();
        std::thread::Builder::new()
            .name("island-lifecycle".into())
            .spawn(move || {
                for request in receiver {
                    match handle_request(&handle, &shared, &exiting, &path, request) {
                        Ok(true) => break,
                        Ok(false) => {}
                        Err(error) => {
                            eprintln!("ChatGPT Island operation failed: {error}");
                            if let Ok(mut core) = shared.lock() {
                                core.error = Some(error);
                            }
                            if let Err(error) = confirmation::show_settings(&handle) {
                                eprintln!("Cannot show diagnostics: {error}");
                            }
                        }
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn send(&self, request: Request) -> Result<(), String> {
        self.sender
            .send(request)
            .map_err(|_| "Lifecycle service unavailable.".into())
    }
    pub fn snapshot(&self) -> Result<Snapshot, String> {
        let c = self
            .shared
            .lock()
            .map_err(|_| "Lifecycle state unavailable.")?;
        Ok(Snapshot {
            phase: c.model.phase,
            session_id: c.model.profile.as_ref().map(|p| p.id.clone()),
            skip_close_confirmation: c.settings.skip_close_confirmation,
            clipboard_fallback_enabled: c.settings.clipboard_fallback_enabled,
            capture_status: c.capture_status.clone(),
            shortcut_registered: c.registration.registered,
            shortcut: "Alt + Shift + E",
            error: c.error.clone(),
        })
    }
    pub fn session_id(&self) -> Option<String> {
        self.shared
            .lock()
            .ok()?
            .model
            .profile
            .as_ref()
            .map(|p| p.id.clone())
    }
}

fn focus(app: &AppHandle, label: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(label) {
        window.unminimize().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn finish_exit(app: &AppHandle, shared: &Arc<Mutex<Core>>, exiting: &Arc<AtomicBool>) -> bool {
    if let Ok(mut c) = shared.lock() {
        c.pending_selection.clear();
        if let Some(mut h) = c.handoff.take() {
            h.cancel();
        }
    }
    eprintln!("Application Quit: releasing global shortcut.");
    let mut registration = shared
        .lock()
        .map(|mut c| std::mem::take(&mut c.registration))
        .unwrap_or_default();
    if let Err(error) = registration.stop(|| {
        app.global_shortcut()
            .unregister_all()
            .map_err(|e| e.to_string())
    }) {
        eprintln!("Shortcut cleanup failed: {error}");
    }
    exiting.store(true, Ordering::SeqCst);
    eprintln!("Application Quit: requesting runtime exit.");
    app.exit(0);
    true
}
fn apply(
    app: &AppHandle,
    shared: &Arc<Mutex<Core>>,
    exiting: &Arc<AtomicBool>,
    effect: Effect,
) -> Result<bool, String> {
    match effect {
        Effect::None => {}
        Effect::Quit => return Ok(finish_exit(app, shared, exiting)),
        Effect::Confirm => {
            let id = shared
                .lock()
                .map_err(|_| "Lifecycle state unavailable.")?
                .model
                .profile
                .as_ref()
                .ok_or("Missing session.")?
                .id
                .clone();
            if let Err(error) = confirmation::show_confirmation(app, &id) {
                shared
                    .lock()
                    .map_err(|_| "Lifecycle state unavailable.")?
                    .model
                    .cancel();
                return Err(error);
            }
        }
        Effect::FocusConfirmation => focus(app, "confirmation")?,
        Effect::Destroy => {
            let id = shared
                .lock()
                .map_err(|_| "Lifecycle state unavailable.")?
                .model
                .profile
                .as_ref()
                .ok_or("Missing session.")?
                .id
                .clone();
            if let Ok(mut c) = shared.lock()
                && let Some(h) = c.handoff.as_mut()
            {
                h.cancel();
            }
            if let Some(notice) = app.get_webview_window("handoff") {
                let _ = notice.destroy();
            }
            let result = if let Some(window) = app.get_webview_window(&id) {
                window.destroy().map_err(|e| e.to_string())
            } else {
                app.state::<Controller>()
                    .send(Request::Destroyed(id.clone()))
            };
            if let Err(error) = result {
                shared
                    .lock()
                    .map_err(|_| "Lifecycle state unavailable.")?
                    .model
                    .destroy_failed();
                return Err(format!("Popup destruction failed: {error}"));
            }
        }
    }
    Ok(false)
}
fn close(
    app: &AppHandle,
    shared: &Arc<Mutex<Core>>,
    exiting: &Arc<AtomicBool>,
    quit: bool,
) -> Result<bool, String> {
    let effect = {
        let mut c = shared.lock().map_err(|_| "Lifecycle state unavailable.")?;
        let skip = c.settings.skip_close_confirmation;
        c.model.close(quit, skip)
    };
    apply(app, shared, exiting, effect)
}
fn handle_request(
    app: &AppHandle,
    shared: &Arc<Mutex<Core>>,
    exiting: &Arc<AtomicBool>,
    settings_path: &std::path::Path,
    request: Request,
) -> Result<bool, String> {
    let capture_context = match &request {
        Request::OpenSelected(context) => Some(*context),
        _ => None,
    };
    match request {
        Request::Settings => confirmation::show_settings(app)?,
        Request::HandoffFailed(id) => {
            let show = shared
                .lock()
                .map_err(|_| "State unavailable.")?
                .handoff
                .as_ref()
                .is_some_and(|h| h.session == id && h.status == crate::handoff::Status::Failed);
            let current = shared.lock().map_err(|_| "State unavailable.")?;
            let show = show && current.model.matches(&id) && current.model.phase == Phase::Open;
            drop(current);
            if show {
                confirmation::show_handoff(app, &id)?;
            }
        }
        Request::MoveResize(resize) => {
            if let Some(id) = app.state::<Controller>().session_id()
                && let Some(window) = app.get_webview_window(&id)
            {
                crate::window_layout::keyboard_move_resize(&window, resize);
            }
        }
        Request::Quit => {
            eprintln!("Application Quit requested from a native control.");
            return close(app, shared, exiting, true);
        }
        Request::Close => return close(app, shared, exiting, false),
        Request::Toggle(context) => {
            let phase = shared
                .lock()
                .map_err(|_| "Lifecycle state unavailable.")?
                .model
                .phase;
            if phase == Phase::Closed {
                return handle_request(
                    app,
                    shared,
                    exiting,
                    settings_path,
                    Request::OpenSelected(context),
                );
            }
            return close(app, shared, exiting, false);
        }
        Request::Open | Request::OpenSelected(_) => {
            let (phase, id) = {
                let c = shared.lock().map_err(|_| "Lifecycle state unavailable.")?;
                (
                    c.model.phase,
                    c.model.profile.as_ref().map(|p| p.id.clone()),
                )
            };
            match phase {
                Phase::Open => {
                    if let Some(id) = id {
                        focus(app, &id)?;
                    }
                }
                Phase::AwaitingCloseConfirmation => focus(app, "confirmation")?,
                Phase::Opening | Phase::Closing => {}
                Phase::Closed => {
                    let root = app
                        .path()
                        .app_local_data_dir()
                        .map_err(|e| e.to_string())?
                        .join("private-sessions");
                    let (id, path) = crate::session::create_profile(&root)?;
                    let profile = Profile {
                        id: id.clone(),
                        path,
                    };
                    shared
                        .lock()
                        .map_err(|_| "Lifecycle state unavailable.")?
                        .model
                        .begin(profile.clone());
                    let fallback = shared
                        .lock()
                        .map_err(|_| "Lifecycle state unavailable.")?
                        .settings
                        .clipboard_fallback_enabled;
                    let report = capture_context
                        .map(|context| text_capture::coordinator::capture(context, fallback));
                    let mut clipboard_warning = false;
                    {
                        let mut c = shared.lock().map_err(|_| "Lifecycle state unavailable.")?;
                        c.pending_selection.clear();
                        if let Some(mut h) = c.handoff.take() {
                            h.cancel();
                        }
                        c.capture_status = "No selection context (opened without capture).".into();
                        if let Some(report) = report {
                            clipboard_warning = report.clipboard_warning;
                            c.capture_status = if let Some(selection) = report.selection.as_ref() {
                                format!(
                                    "Selection captured: {} Unicode characters via {:?} in {} ms.",
                                    selection.text.chars().count(),
                                    selection.method,
                                    report.elapsed_ms
                                )
                            } else if let Some(error) = report.error.as_ref() {
                                format!("{} ({} ms)", error.message(), report.elapsed_ms)
                            } else {
                                format!("No selected text ({} ms).", report.elapsed_ms)
                            };

                            if clipboard_warning {
                                c.error = Some("Copy fallback may have left copied text on the clipboard. Existing nonempty clipboard data was never replaced.".into());
                            }
                            c.pending_selection.replace(&id, report.selection);
                        } else {
                            c.pending_selection.replace(&id, None);
                        }
                    }
                    // All capture and state storage finish before WebView creation/focus.
                    match popup::create(app, &profile, capture_context.flatten()) {
                        Ok(()) => shared
                            .lock()
                            .map_err(|_| "Lifecycle state unavailable.")?
                            .model
                            .opened(&id),
                        Err(error) => {
                            shared
                                .lock()
                                .map_err(|_| "Lifecycle state unavailable.")?
                                .pending_selection
                                .clear_session(&id);
                            // A failed verification can leave a live native window. Keep
                            // ownership until its Destroyed event; never permit a duplicate.
                            if let Some(window) = app.get_webview_window(&id) {
                                shared
                                    .lock()
                                    .map_err(|_| "Lifecycle state unavailable.")?
                                    .model
                                    .phase = Phase::Closing;
                                if let Err(e) = window.destroy() {
                                    shared
                                        .lock()
                                        .map_err(|_| "Lifecycle state unavailable.")?
                                        .model
                                        .destroy_failed();
                                    return Err(format!(
                                        "{error}; could not destroy failed popup: {e}"
                                    ));
                                }
                            } else {
                                shared
                                    .lock()
                                    .map_err(|_| "Lifecycle state unavailable.")?
                                    .model
                                    .destroyed(&id);
                            }
                            return Err(error);
                        }
                    }
                    crate::handoff::start(app, &id);
                    if clipboard_warning {
                        confirmation::show_settings(app)?;
                    }
                }
            }
        }
        Request::Destroyed(id) => {
            if shared
                .lock()
                .map_err(|_| "State unavailable.")?
                .model
                .matches(&id)
                && let Some(window) = app.get_webview_window("handoff")
            {
                let _ = window.destroy();
            }
            let effect = shared
                .lock()
                .map_err(|_| "Lifecycle state unavailable.")?
                .destroyed(&id);
            eprintln!(
                "Popup destroyed; application-owned profile retained for safe offline cleanup."
            );
            return apply(app, shared, exiting, effect);
        }
        Request::Decision {
            session_id,
            confirm,
            skip,
            reply,
        } => {
            let result = (|| -> Result<Effect, String> {
                {
                    let c = shared.lock().map_err(|_| "Lifecycle state unavailable.")?;
                    if !c.model.matches(&session_id)
                        || c.model.phase != Phase::AwaitingCloseConfirmation
                    {
                        return Err("This confirmation is no longer active.".into());
                    }
                }
                if confirm && skip {
                    let settings = shared
                        .lock()
                        .map_err(|_| "Lifecycle state unavailable.")?
                        .settings
                        .after_decision(confirm, skip);
                    settings::save(settings_path, &settings)?;
                    shared
                        .lock()
                        .map_err(|_| "Lifecycle state unavailable.")?
                        .settings = settings;
                }
                // Preference is never written on Cancel, regardless of checkbox.
                if let Some(window) = app.get_webview_window("confirmation") {
                    window.destroy().map_err(|e| e.to_string())?;
                }
                let effect = {
                    let mut c = shared.lock().map_err(|_| "Lifecycle state unavailable.")?;
                    if confirm {
                        c.model.confirm()
                    } else {
                        c.model.cancel();
                        Effect::None
                    }
                };
                if !confirm {
                    focus(app, &session_id)?;
                    crate::handoff::show_failed_notice(app, &session_id);
                }
                Ok(effect)
            })();
            match result {
                Ok(effect) => {
                    if let Some(reply) = reply {
                        let _ = reply.send(Ok(()));
                    }
                    return apply(app, shared, exiting, effect);
                }
                Err(error) => {
                    if let Some(reply) = reply {
                        let _ = reply.send(Err(error.clone()));
                    }
                    return Err(error);
                }
            }
        }
        Request::Reset(reply) => {
            let mut settings = shared
                .lock()
                .map_err(|_| "Lifecycle state unavailable.")?
                .settings
                .clone();
            settings.skip_close_confirmation = false;
            let result = settings::save(settings_path, &settings);
            if result.is_ok() {
                shared
                    .lock()
                    .map_err(|_| "Lifecycle state unavailable.")?
                    .settings = settings;
            }
            let _ = reply.send(result.clone());
            result?;
        }
        Request::ClipboardPreference(enabled, reply) => {
            let mut settings = shared
                .lock()
                .map_err(|_| "Lifecycle state unavailable.")?
                .settings
                .clone();
            settings.clipboard_fallback_enabled = enabled;
            let result = settings::save(settings_path, &settings);
            if result.is_ok() {
                shared
                    .lock()
                    .map_err(|_| "Lifecycle state unavailable.")?
                    .settings = settings;
            }
            let _ = reply.send(result.clone());
            result?;
        }
    }
    Ok(false)
}
fn authorize(window: &WebviewWindow, labels: &[&str]) -> Result<(), String> {
    if labels.contains(&window.label()) {
        Ok(())
    } else {
        Err("This control is restricted to trusted local UI.".into())
    }
}
#[tauri::command]
pub fn open_popup(app: AppHandle, window: WebviewWindow) -> Result<String, String> {
    authorize(&window, &["main"])?;
    app.state::<Controller>().send(Request::Open)?;
    Ok("Open requested.".into())
}
#[tauri::command]
pub fn close_popup(app: AppHandle, window: WebviewWindow) -> Result<String, String> {
    authorize(&window, &["main"])?;
    app.state::<Controller>().send(Request::Close)?;
    Ok("Close requested; confirmation may be required.".into())
}
#[tauri::command]
pub fn popup_status(app: AppHandle, window: WebviewWindow) -> Result<String, String> {
    authorize(&window, &["main"])?;
    let snapshot = app.state::<Controller>().snapshot()?;
    Ok(format!("{:?}; {}", snapshot.phase, snapshot.capture_status))
}
#[tauri::command]
pub fn ui_snapshot(app: AppHandle, window: WebviewWindow) -> Result<Snapshot, String> {
    authorize(&window, &["settings"])?;
    app.state::<Controller>().snapshot()
}
#[tauri::command]
pub async fn resolve_confirmation(
    app: AppHandle,
    window: WebviewWindow,
    session_id: String,
    confirm: bool,
    skip: bool,
) -> Result<(), String> {
    authorize(&window, &["confirmation"])?;
    let (reply, receiver) = mpsc::channel();
    app.state::<Controller>().send(Request::Decision {
        session_id,
        confirm,
        skip,
        reply: Some(reply),
    })?;
    tauri::async_runtime::spawn_blocking(move || {
        receiver
            .recv()
            .map_err(|_| "Lifecycle service stopped.".to_string())?
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn reset_warning(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    authorize(&window, &["settings"])?;
    let (reply, receiver) = mpsc::channel();
    app.state::<Controller>().send(Request::Reset(reply))?;
    tauri::async_runtime::spawn_blocking(move || {
        receiver
            .recv()
            .map_err(|_| "Lifecycle service stopped.".to_string())?
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_clipboard_fallback(
    app: AppHandle,
    window: WebviewWindow,
    enabled: bool,
) -> Result<(), String> {
    authorize(&window, &["settings"])?;
    let (reply, receiver) = mpsc::channel();
    app.state::<Controller>()
        .send(Request::ClipboardPreference(enabled, reply))?;
    tauri::async_runtime::spawn_blocking(move || {
        receiver
            .recv()
            .map_err(|_| "Lifecycle service stopped.".to_string())?
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn selection_preview(app: AppHandle, window: WebviewWindow) -> Result<Option<String>, String> {
    authorize(&window, &["main"])?;
    if !cfg!(debug_assertions) || !std::env::args().any(|a| a == "--dev-selection-preview") {
        return Err("Selection preview is disabled. Enable the explicit debug-only flag for non-sensitive fixtures.".into());
    }
    let state = app.state::<Controller>();
    let core = state
        .shared
        .lock()
        .map_err(|_| "Lifecycle state unavailable.")?;
    Ok(core
        .model
        .profile
        .as_ref()
        .and_then(|profile| core.pending_selection.for_session(&profile.id))
        .map(|selection| {
            let _captured_at = selection.captured_at;
            selection.text.clone()
        }))
}

#[cfg(test)]
mod selection_lifecycle_tests {
    use super::*;
    use crate::text_capture::types::{CaptureMethod, CapturedText};
    #[test]
    fn cancel_and_stale_destroy_preserve_selection_but_active_destroy_releases_it() {
        let mut core = Core {
            model: Model::default(),
            settings: Settings::default(),
            registration: Default::default(),
            error: None,
            pending_selection: Default::default(),
            capture_status: String::new(),
            handoff: None,
        };
        core.model.begin(Profile {
            id: "active".into(),
            path: "unused".into(),
        });
        core.model.opened("active");
        core.pending_selection.replace(
            "active",
            Some(CapturedText {
                text: "non-sensitive fixture".into(),
                captured_at: std::time::SystemTime::now(),
                method: CaptureMethod::UiAutomation,
            }),
        );
        core.model.close(false, false);
        core.model.cancel();
        assert!(core.pending_selection.for_session("active").is_some());
        core.destroyed("stale");
        assert!(core.pending_selection.for_session("active").is_some());
        core.model.close(false, false);
        core.model.confirm();
        core.destroyed("active");
        assert_eq!(core.model.phase, Phase::Closed);
        assert!(core.pending_selection.for_session("active").is_none());
    }
}
