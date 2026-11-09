mod popup;
mod session;

use std::sync::Mutex;
use tauri::Manager;

pub fn run() {
    let result = tauri::Builder::default()
        .manage(Mutex::new(session::Session::default()))
        .invoke_handler(tauri::generate_handler![
            popup::open_popup,
            popup::close_popup,
            popup::popup_status
        ])
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                if window.label() == "main" {
                    window.app_handle().exit(0);
                } else {
                    let state = window.state::<popup::Sessions>();
                    if let Ok(mut session) = state.lock() {
                        session.release(window.label());
                    }
                    eprintln!("Private popup destroyed; profile retained pending safe offline cleanup.");
                }
            }
        })
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("ChatGPT Island initialization/runtime failure. Check WebView2 Runtime: {error}");
        std::process::exit(1);
    }
}
