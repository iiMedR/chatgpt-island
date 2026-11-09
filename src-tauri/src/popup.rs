use crate::session::{self, Session};
use std::sync::Mutex;
use tauri::{
    AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, webview::NewWindowResponse,
};
pub type Sessions = Mutex<Session>;

fn authorize(window: &WebviewWindow) -> Result<(), String> {
    if window.label() == "main" {
        Ok(())
    } else {
        Err("Controls are restricted to the local launcher.".into())
    }
}

#[tauri::command]
pub async fn open_popup(app: AppHandle, window: WebviewWindow) -> Result<String, String> {
    authorize(&window)?;
    let url = session::CHATGPT_URL
        .parse()
        .map_err(|e| format!("Invalid website URL: {e}"))?;
    let state = app.state::<Sessions>();
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join("private-sessions");
    let (id, profile) = {
        let mut s = state.lock().map_err(|_| "Session state unavailable.")?;
        if s.active().is_some() {
            return Err("A popup is already active or being destroyed.".into());
        }
        let (id, profile) = session::create_profile(&root)?;
        s.begin(id.clone())?;
        (id, profile)
    };
    let result = WebviewWindowBuilder::new(&app, &id, WebviewUrl::External(url))
        .title("ChatGPT Island - private feasibility test")
        .inner_size(900.0, 650.0)
        .center()
        .resizable(true)
        .incognito(true)
        .data_directory(profile)
        .devtools(false)
        .on_navigation(session::allowed_navigation)
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_download(|_, _| false)
        .build();
    match result {
        Ok(popup) => {
            if let Err(error) = verify_private_mode(&popup) {
                // Keep the reservation until Destroyed. If destruction fails,
                // mark ready so the launcher can retry destruction.
                state
                    .lock()
                    .map_err(|_| "Session state unavailable.")?
                    .opened(&id);
                if let Err(destroy_error) = popup.destroy() {
                    return Err(format!("{error}; destruction failed: {destroy_error}"));
                }
                return Err(error);
            }
            state
                .lock()
                .map_err(|_| "Session state unavailable.")?
                .opened(&id);
            Ok("Popup created; WebView2 reports private mode enabled. Verify loading and guest access manually.".into())
        }
        Err(error) => {
            state
                .lock()
                .map_err(|_| "Session state unavailable.")?
                .release(&id);
            Err(format!(
                "WebView creation failed. Check WebView2 Runtime and profile permissions: {error}"
            ))
        }
    }
}

#[tauri::command]
pub async fn close_popup(app: AppHandle, window: WebviewWindow) -> Result<String, String> {
    authorize(&window)?;
    let state = app.state::<Sessions>();
    let id = state
        .lock()
        .map_err(|_| "Session state unavailable.")?
        .request_close()?;
    if let Some(popup) = app.get_webview_window(&id) {
        if let Err(error) = popup.destroy() {
            state
                .lock()
                .map_err(|_| "Session state unavailable.")?
                .close_failed();
            return Err(format!("Could not destroy popup: {error}"));
        }
        Ok("Destruction requested. Wait for idle status before reopening.".into())
    } else {
        state
            .lock()
            .map_err(|_| "Session state unavailable.")?
            .release(&id);
        Ok("Popup no longer exists; session reference released.".into())
    }
}

#[tauri::command]
pub fn popup_status(app: AppHandle, window: WebviewWindow) -> Result<String, String> {
    authorize(&window)?;
    let state = app.state::<Sessions>();
    let s = state.lock().map_err(|_| "Session state unavailable.")?;
    Ok(if s.active().is_some() {
        "active / closing"
    } else {
        "idle"
    }
    .into())
}

// Inspect only the native profile's private-mode flag, never page data or cookies.
#[cfg(windows)]
fn verify_private_mode(popup: &WebviewWindow) -> Result<(), String> {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_13;
    use windows_core::Interface;
    let (sender, receiver) = std::sync::mpsc::channel();
    popup
        .with_webview(move |webview| {
            // SAFETY: Tauri executes this callback on the WebView's owning thread.
            // The controller and profile remain owned by the live native WebView.
            let result = unsafe {
                (|| -> windows_core::Result<bool> {
                    let core = webview.controller().CoreWebView2()?;
                    let profile = core.cast::<ICoreWebView2_13>()?.Profile()?;
                    let mut enabled = windows_core::BOOL::default();
                    profile.IsInPrivateModeEnabled(&mut enabled)?;
                    Ok(enabled.as_bool())
                })()
            }
            .map_err(|error| format!("Cannot verify WebView2 private mode: {error}"));
            let _ = sender.send(result);
        })
        .map_err(|error| error.to_string())?;
    match receiver.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(Ok(true)) => Ok(()),
        Ok(Ok(false)) => Err("WebView2 did not enable private mode; session refused.".into()),
        Ok(Err(error)) => Err(error),
        Err(error) => Err(format!(
            "Private-mode verification did not complete: {error}"
        )),
    }
}

#[cfg(not(windows))]
fn verify_private_mode(_popup: &WebviewWindow) -> Result<(), String> {
    Err("This feasibility prototype requires Windows WebView2.".into())
}
