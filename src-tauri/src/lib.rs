mod confirmation;
mod handoff;
mod lifecycle;
mod popup;
mod session;
mod settings;
mod shortcuts;
mod text_capture;
mod tray;
mod window_layout;
use lifecycle::{Controller, Request};
use std::sync::atomic::Ordering;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(shortcuts::plugin())
        .on_menu_event(tray::handle_menu_event)
        .invoke_handler(tauri::generate_handler![
            lifecycle::open_popup,
            lifecycle::close_popup,
            lifecycle::popup_status,
            lifecycle::ui_snapshot,
            lifecycle::resolve_confirmation,
            lifecycle::reset_warning,
            lifecycle::show_manual_handoff,
            lifecycle::set_clipboard_fallback,
            lifecycle::selection_preview,
            handoff::handoff_snapshot,
            handoff::handoff_focus
        ])
        .setup(|app| {
            Controller::install(app.handle())?;
            // A tray failure is fatal: never leave an inaccessible invisible app.
            tray::initialize(app.handle())?;
            shortcuts::register(app.handle());
            if cfg!(debug_assertions) && std::env::args().any(|a| a == "--dev-launcher") {
                confirmation::show_launcher(app.handle())?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            let app = window.app_handle();
            let controller = app.state::<Controller>();
            match event {
                tauri::WindowEvent::CloseRequested { api, .. }
                    if !controller.exiting.load(Ordering::SeqCst) =>
                {
                    match window.label() {
                        "settings" | "handoff" => {}
                        "main" => {
                            api.prevent_close();
                            let _ = controller.send(Request::Quit);
                        }
                        "confirmation" => {
                            api.prevent_close();
                            if let Some(session_id) = controller.session_id() {
                                let _ = controller.send(Request::Decision {
                                    session_id,
                                    confirm: false,
                                    skip: false,
                                    reply: None,
                                });
                            }
                        }
                        id if id.starts_with("island-") => {
                            api.prevent_close();
                            let _ = controller.send(Request::Close);
                        }
                        _ => {}
                    }
                }
                tauri::WindowEvent::Destroyed if window.label().starts_with("island-") => {
                    let _ = controller.send(Request::Destroyed(window.label().to_string()));
                }
                tauri::WindowEvent::Moved(_)
                | tauri::WindowEvent::Resized(_)
                | tauri::WindowEvent::ScaleFactorChanged { .. }
                    if window.label().starts_with("island-") =>
                {
                    if let Some(popup) = app.get_webview_window(window.label()) {
                        window_layout::ensure_visible(&popup);
                    }
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!());
    match app {
        Ok(app) => app.run(|app, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                if !app.state::<Controller>().exiting.load(Ordering::SeqCst) {
                    api.prevent_exit();
                }
            }
            tauri::RunEvent::Exit => {
                let _ = app.global_shortcut().unregister_all();
            }
            _ => {}
        }),
        Err(error) => {
            eprintln!("ChatGPT Island startup failed: {error}");
            std::process::exit(1);
        }
    }
}

pub fn run_capture_helper_if_requested() -> bool {
    text_capture::coordinator::run_helper_if_requested()
}
