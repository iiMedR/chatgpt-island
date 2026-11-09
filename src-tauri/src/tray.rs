use crate::lifecycle::{Controller, Request};
use tauri::{
    AppHandle, Manager,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
pub fn initialize(app: &AppHandle) -> Result<(), String> {
    let title = MenuItem::with_id(app, "title", "ChatGPT Island", false, None::<&str>)
        .map_err(|e| e.to_string())?;
    let separator = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let open = MenuItem::with_id(app, "open", "Open ChatGPT Island", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit =
        MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).map_err(|e| e.to_string())?;
    let menu = Menu::with_items(app, &[&title, &separator, &open, &settings, &quit])
        .map_err(|e| e.to_string())?;
    let icon = app
        .default_window_icon()
        .ok_or("Tray icon is missing; refusing invisible startup.")?
        .clone();
    TrayIconBuilder::with_id("island-tray")
        .icon(icon)
        .tooltip("ChatGPT Island — Alt + Shift + E")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let command = event.id.as_ref();
            eprintln!("Tray menu event received: {command}");
            let request = match command {
                "open" => Request::Open,
                "settings" => Request::Settings,
                "quit" => Request::Quit,
                _ => return,
            };
            if let Err(error) = app.state::<Controller>().send(request) {
                eprintln!("Tray command could not reach lifecycle service: {error}");
            }
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                let _ = tray.app_handle().state::<Controller>().send(Request::Open);
            }
        })
        .build(app)
        .map_err(|e| format!("Tray initialization failed: {e}"))?;
    Ok(())
}
