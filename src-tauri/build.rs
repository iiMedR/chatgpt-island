fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "open_popup",
            "close_popup",
            "popup_status",
            "ui_snapshot",
            "resolve_confirmation",
            "reset_warning",
        ]),
    ))
    .expect("Tauri build configuration failed")
}
