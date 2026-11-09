use crate::{
    lifecycle::{Controller, Phase},
    text_capture::validation,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tauri::{AppHandle, Manager, WebviewWindow};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Status {
    Pending,
    WaitingForChatGPT,
    Inserting,
    Ready,
    Failed,
    Cancelled,
}
pub struct Handoff {
    pub session: String,
    pub status: Status,
    pub cancelled: Arc<AtomicBool>,
    pub attempted: bool,
}
impl Handoff {
    pub fn new(session: &str) -> Self {
        Self {
            session: session.into(),
            status: Status::Pending,
            cancelled: Arc::new(AtomicBool::new(false)),
            attempted: false,
        }
    }
    pub fn claim(&mut self) -> bool {
        if self.attempted
            || self.cancelled.load(Ordering::SeqCst)
            || self.status != Status::WaitingForChatGPT
        {
            return false;
        }
        self.attempted = true;
        self.status = Status::Inserting;
        true
    }
    pub fn cancel(&mut self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.status = Status::Cancelled;
    }
}
pub fn draft(text: &str) -> Result<String, String> {
    let text = validation::validate(text).map_err(|_| "Selected text is invalid.".to_string())?;
    Ok(format!(
        "Regarding this selected text:\n\n{text}\n\nMy question: "
    ))
}
pub fn allowed(url: &tauri::Url) -> bool {
    crate::session::allowed_navigation(url)
        && url.path() == "/"
        && url.query().is_none()
        && url.fragment().is_none()
}
fn script(mode: &str, draft: &str) -> String {
    let payload = serde_json::json!({"mode": mode, "draft": draft});
    include_str!("../../../src/composer-script.js")
        .replace("__ISLAND_PAYLOAD__", &payload.to_string())
}
fn active(app: &AppHandle, id: &str, cancel: &AtomicBool) -> bool {
    !cancel.load(Ordering::SeqCst)
        && app.state::<Controller>().shared.lock().is_ok_and(|c| {
            c.model.matches(id)
                && matches!(
                    c.model.phase,
                    Phase::Open | Phase::AwaitingCloseConfirmation
                )
        })
}
fn can_edit(app: &AppHandle, id: &str) -> bool {
    app.state::<Controller>()
        .shared
        .lock()
        .is_ok_and(|c| c.model.matches(id) && c.model.phase == Phase::Open)
}
fn evaluate(window: &WebviewWindow, source: String, cancel: Arc<AtomicBool>) -> Result<String, ()> {
    if cancel.load(Ordering::SeqCst) {
        return Err(());
    }
    let url = window.url().map_err(|_| ())?;
    if url.as_str() == "about:blank" {
        return Ok("waiting".into());
    }
    if !allowed(&url) {
        return Err(());
    }
    let (tx, rx) = std::sync::mpsc::channel();
    window
        .eval_with_callback(source, move |json| {
            let _ = tx.send(json);
        })
        .map_err(|_| ())?;
    // Results contain a small status only, never page/composer/conversation text.
    let json = rx
        .recv_timeout(Duration::from_millis(750))
        .map_err(|_| ())?;
    if json.len() > 64 {
        return Err(());
    }
    serde_json::from_str::<String>(&json).map_err(|_| ())
}
fn set_status(app: &AppHandle, id: &str, status: Status) {
    if let Ok(mut c) = app.state::<Controller>().shared.lock()
        && let Some(h) = c
            .handoff
            .as_mut()
            .filter(|h| h.session == id && !h.cancelled.load(Ordering::SeqCst))
    {
        h.status = status;
    }
}
pub fn start(app: &AppHandle, id: &str) {
    let cancel = {
        let state = app.state::<Controller>();
        let Ok(mut c) = state.shared.lock() else {
            return;
        };
        if c.pending_selection.for_session(id).is_none() {
            return;
        }
        let h = Handoff::new(id);
        let cancel = h.cancelled.clone();
        c.handoff = Some(h);
        cancel
    };
    let app = app.clone();
    let id = id.to_string();
    std::thread::spawn(move || {
        set_status(&app, &id, Status::WaitingForChatGPT);
        let deadline = std::time::Instant::now() + Duration::from_secs(15);
        let mut ready = false;
        while std::time::Instant::now() < deadline && active(&app, &id, &cancel) {
            if !can_edit(&app, &id) {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            let Some(window) = app.get_webview_window(&id) else {
                return;
            };
            match evaluate(&window, script("probe", ""), cancel.clone()).as_deref() {
                Ok("ready") => {
                    ready = true;
                    break;
                }
                Ok("waiting") => std::thread::sleep(Duration::from_millis(300)),
                _ => break,
            }
        }
        if !active(&app, &id, &cancel) {
            return;
        }
        if ready && can_edit(&app, &id) {
            let prepared = {
                let state = app.state::<Controller>();
                let Ok(mut c) = state.shared.lock() else {
                    return;
                };
                let prepared = c
                    .pending_selection
                    .for_session(&id)
                    .and_then(|s| draft(&s.text).ok());
                if !c
                    .handoff
                    .as_mut()
                    .is_some_and(|h| h.session == id && h.claim())
                {
                    return;
                }
                prepared
            };
            if let (Some(prepared), Some(window)) = (prepared, app.get_webview_window(&id))
                && active(&app, &id, &cancel)
                && can_edit(&app, &id)
                && evaluate(&window, script("insert", &prepared), cancel.clone()).as_deref()
                    == Ok("inserted")
            {
                std::thread::sleep(Duration::from_millis(300));
                if active(&app, &id, &cancel)
                    && evaluate(&window, script("verify", &prepared), cancel.clone()).as_deref()
                        == Ok("accepted")
                {
                    set_status(&app, &id, Status::Ready);
                    return;
                }
            }
        }
        if active(&app, &id, &cancel) {
            set_status(&app, &id, Status::Failed);
            // Never retry an uncertain insertion. Preserve selection for deliberate manual copy.
            // Keep failures silent; the user may open manual handoff from Settings.
        }
    });
}
#[derive(serde::Serialize)]
pub struct Notice {
    status: Status,
    draft: String,
}
#[tauri::command]
pub fn handoff_snapshot(
    app: AppHandle,
    window: WebviewWindow,
    session_id: String,
) -> Result<Notice, String> {
    if window.label() != "handoff" {
        return Err("Trusted handoff notice only.".into());
    }
    let state = app.state::<Controller>();
    let c = state.shared.lock().map_err(|_| "State unavailable.")?;
    if !c.model.matches(&session_id) {
        return Err("This session has closed.".into());
    }
    let h = c
        .handoff
        .as_ref()
        .filter(|h| h.session == session_id)
        .ok_or("No handoff.")?;
    let selection = c
        .pending_selection
        .for_session(&session_id)
        .ok_or("No selected text.")?;
    Ok(Notice {
        status: h.status,
        draft: draft(&selection.text)?,
    })
}
#[tauri::command]
pub fn handoff_focus(
    app: AppHandle,
    window: WebviewWindow,
    session_id: String,
) -> Result<(), String> {
    if window.label() != "handoff"
        || app.state::<Controller>().session_id().as_deref() != Some(&session_id)
    {
        return Err("This session is unavailable.".into());
    }
    app.get_webview_window(&session_id)
        .ok_or("Popup closed.")?
        .set_focus()
        .map_err(|_| "Cannot focus popup.".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_draft_limits_and_script_data_are_safe() {
        let text = "Français\nمرحبا\nEnglish \" ` ${alert(1)} </script>";
        let prepared = draft(text).unwrap();
        assert_eq!(prepared.matches(text).count(), 1);
        assert!(draft("").is_err());
        assert!(draft(&"🙂".repeat(10000)).is_ok());
        assert!(draft(&"a".repeat(10001)).is_err());
        let source = script("insert", &prepared);
        let payload = source
            .rsplit_once(")(")
            .unwrap()
            .1
            .trim()
            .trim_end_matches(')');
        let parsed: serde_json::Value = serde_json::from_str(payload).unwrap();
        assert_eq!(parsed["draft"], prepared);
    }
    #[test]
    fn insertion_is_claimed_once_and_cancellation_is_terminal() {
        let mut h = Handoff::new("one");
        assert!(!h.claim());
        h.status = Status::WaitingForChatGPT;
        assert!(h.claim());
        assert!(!h.claim());
        h.cancel();
        assert!(!h.claim());
        assert!(h.cancelled.load(Ordering::SeqCst));
        let other = Handoff::new("two");
        assert_ne!(h.session, other.session);
    }
    #[test]
    fn unexpected_and_login_documents_are_rejected() {
        assert!(allowed(&"https://chatgpt.com/".parse().unwrap()));
        for url in [
            "https://chatgpt.com/auth/login",
            "https://chatgpt.com/c/old",
            "https://chatgpt.com/?login=1",
            "https://chatgpt.com.evil/",
            "https://user@chatgpt.com/",
            "http://chatgpt.com/",
        ] {
            assert!(!allowed(&url.parse().unwrap()));
        }
    }
}
