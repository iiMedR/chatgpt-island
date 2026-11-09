use std::sync::Mutex;
use tauri::{webview::NewWindowResponse, AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use crate::session::{self, Session};
pub type Sessions = Mutex<Session>;

fn authorize(window: &WebviewWindow) -> Result<(), String> {
    if window.label() == "main" { Ok(()) }
    else { Err("Controls are restricted to the local launcher.".into()) }
}

#[tauri::command]
pub async fn open_popup(app: AppHandle, window: WebviewWindow) -> Result<String, String> {
    authorize(&window)?;
    let url = session::CHATGPT_URL.parse().map_err(|e| format!("Invalid website URL: {e}"))?;
    let state = app.state::<Sessions>();
    let root = app.path().app_local_data_dir().map_err(|e| e.to_string())?.join("private-sessions");
    let (id, profile) = {
        let mut s = state.lock().map_err(|_| "Session state unavailable.")?;
        if s.active().is_some() { return Err("A popup is already active or being destroyed.".into()); }
        let (id, profile) = session::create_profile(&root)?;
        s.begin(id.clone())?;
        (id, profile)
    };
    let result = WebviewWindowBuilder::new(&app, &id, WebviewUrl::External(url))
        .title("ChatGPT Island - private feasibility test")
        .inner_size(900.0, 650.0).center().resizable(true)
        .incognito(true).data_directory(profile).devtools(false)
        .on_navigation(session::allowed_navigation)
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_download(|_, _| false)
        .build();
    match result {
        Ok(_) => {
            state.lock().map_err(|_| "Session state unavailable.")?.opened(&id);
            Ok("Popup created. Verify loading and guest access manually; network success is not implied.".into())
        }
        Err(error) => {
            state.lock().map_err(|_| "Session state unavailable.")?.release(&id);
            Err(format!("WebView creation failed. Check WebView2 Runtime and profile permissions: {error}"))
        }
    }
}

#[tauri::command]
pub async fn close_popup(app: AppHandle, window: WebviewWindow) -> Result<String, String> {
    authorize(&window)?;
    let state = app.state::<Sessions>();
    let id = state.lock().map_err(|_| "Session state unavailable.")?.request_close()?;
    if let Some(popup) = app.get_webview_window(&id) {
        if let Err(error) = popup.destroy() {
            state.lock().map_err(|_| "Session state unavailable.")?.close_failed();
            return Err(format!("Could not destroy popup: {error}"));
        }
        Ok("Destruction requested. Wait for idle status before reopening.".into())
    } else {
        state.lock().map_err(|_| "Session state unavailable.")?.release(&id);
        Ok("Popup no longer exists; session reference released.".into())
    }
}

#[tauri::command]
pub fn popup_status(app: AppHandle, window: WebviewWindow) -> Result<String, String> {
    authorize(&window)?;
    let state = app.state::<Sessions>();
    let s = state.lock().map_err(|_| "Session state unavailable.")?;
    Ok(if s.active().is_some() { "active / closing" } else { "idle" }.into())
}
