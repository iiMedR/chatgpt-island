use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, webview::NewWindowResponse};

pub fn local_navigation(url: &tauri::Url) -> bool {
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    (url.scheme() == "tauri" && url.host_str() == Some("localhost") && url.port().is_none())
        || url.origin().ascii_serialization() == "http://tauri.localhost"
        || (cfg!(debug_assertions) && url.origin().ascii_serialization() == "http://127.0.0.1:1420")
}
fn show(
    app: &AppHandle,
    label: &str,
    title: &str,
    query: &str,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(label) {
        window.unminimize().map_err(|e| e.to_string())?;
        return window.set_focus().map_err(|e| e.to_string());
    }
    WebviewWindowBuilder::new(
        app,
        label,
        WebviewUrl::App(format!("index.html?{query}").into()),
    )
    .title(title)
    .inner_size(width, height)
    .resizable(false)
    .center()
    .minimizable(false)
    .maximizable(false)
    .devtools(false)
    .on_navigation(local_navigation)
    .on_new_window(|_, _| NewWindowResponse::Deny)
    .build()
    .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn show_confirmation(app: &AppHandle, id: &str) -> Result<(), String> {
    show(
        app,
        "confirmation",
        "Close this conversation?",
        &format!("view=confirmation&session={id}"),
        450.0,
        280.0,
    )
}
pub fn show_settings(app: &AppHandle) -> Result<(), String> {
    show(
        app,
        "settings",
        "ChatGPT Island - Settings",
        "view=settings",
        500.0,
        590.0,
    )
}
pub fn show_launcher(app: &AppHandle) -> Result<(), String> {
    show(
        app,
        "main",
        "ChatGPT Island - Development",
        "view=launcher",
        720.0,
        650.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trusted_windows_cannot_navigate_to_remote_or_other_local_servers() {
        for value in [
            "tauri://localhost/index.html",
            "http://tauri.localhost/index.html",
        ] {
            assert!(local_navigation(&value.parse().unwrap()));
        }
        for value in [
            "https://chatgpt.com",
            "http://localhost:3000",
            "http://tauri.localhost:9999",
            "http://127.0.0.1:9999",
            "http://user@tauri.localhost",
        ] {
            assert!(!local_navigation(&value.parse().unwrap()), "{value}");
        }
    }
}
