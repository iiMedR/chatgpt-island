use crate::{
    lifecycle::{Controller, Profile, Request},
    session,
};
use tauri::{
    AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, webview::NewWindowResponse,
};

pub fn create(app: &AppHandle, profile: &Profile) -> Result<(), String> {
    let url = session::CHATGPT_URL
        .parse()
        .map_err(|e| format!("Invalid website URL: {e}"))?;
    let popup = WebviewWindowBuilder::new(app, &profile.id, WebviewUrl::External(url))
        .title("ChatGPT Island")
        .inner_size(900.0, 650.0)
        .center()
        .resizable(true)
        .incognito(true)
        .data_directory(profile.path.clone())
        .devtools(false)
        .on_navigation(session::allowed_navigation)
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_download(|_, _| false)
        .build()
        .map_err(|e| {
            format!("WebView creation failed. Check WebView2 Runtime and profile permissions: {e}")
        })?;
    verify_private_mode(&popup)?;
    install_escape(&popup, app.clone())?;
    eprintln!("Private popup created; WebView2 private-mode flag verified.");
    Ok(())
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

#[cfg(windows)]
fn install_escape(popup: &WebviewWindow, app: AppHandle) -> Result<(), String> {
    use webview2_com::{
        AcceleratorKeyPressedEventHandler,
        Microsoft::Web::WebView2::Win32::{
            COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN, COREWEBVIEW2_PHYSICAL_KEY_STATUS,
        },
    };
    let (sender, receiver) = std::sync::mpsc::channel();
    popup
        .with_webview(move |webview| {
            let handler = AcceleratorKeyPressedEventHandler::create(Box::new(move |_, args| {
                if let Some(args) = args {
                    // SAFETY: WebView2 delivers these arguments on its controller thread.
                    unsafe {
                        let mut key = 0;
                        let mut kind = Default::default();
                        args.VirtualKey(&mut key)?;
                        args.KeyEventKind(&mut kind)?;
                        if key == 0x1b && kind == COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN {
                            let mut status = COREWEBVIEW2_PHYSICAL_KEY_STATUS::default();
                            args.PhysicalKeyStatus(&mut status)?;
                            args.SetHandled(true)?;
                            if !status.WasKeyDown.as_bool() {
                                let _ = app.state::<Controller>().send(Request::Close);
                            }
                        }
                    }
                }
                Ok(())
            }));
            let mut token = 0;
            // SAFETY: controller is valid on this owning thread. WebView2 retains the
            // COM handler until controller destruction; it captures no WebView handle.
            let result = unsafe {
                webview
                    .controller()
                    .add_AcceleratorKeyPressed(&handler, &mut token)
            }
            .map_err(|e| e.to_string());
            let _ = sender.send(result);
        })
        .map_err(|e| e.to_string())?;
    receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .map_err(|e| e.to_string())?
}
#[cfg(not(windows))]
fn install_escape(_popup: &WebviewWindow, _app: AppHandle) -> Result<(), String> {
    Err("Escape handling requires Windows WebView2.".into())
}
