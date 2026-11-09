use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub skip_close_confirmation: bool,
    pub clipboard_fallback_enabled: bool,
}

impl Settings {
    pub fn after_decision(&self, confirmed: bool, skip: bool) -> Self {
        Self {
            skip_close_confirmation: self.skip_close_confirmation || (confirmed && skip),
            clipboard_fallback_enabled: self.clipboard_fallback_enabled,
        }
    }
}
pub fn load(path: &Path) -> (Settings, Option<String>) {
    match fs::read(path) {
        Ok(bytes) => match serde_json::from_slice(&bytes) {
            Ok(value) => (value, None),
            Err(_) => (
                Settings::default(),
                Some("Settings were invalid; close warnings are enabled.".into()),
            ),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (Settings::default(), None),
        Err(_) => (
            Settings::default(),
            Some("Settings could not be read; close warnings are enabled.".into()),
        ),
    }
}

pub fn save(path: &Path, value: &Settings) -> Result<(), String> {
    let parent = path.parent().ok_or("Invalid settings location.")?;
    fs::create_dir_all(parent).map_err(|e| format!("Cannot create settings directory: {e}"))?;
    let temporary = parent.join(format!("settings-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| -> Result<(), String> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| e.to_string())?;
        file.write_all(&serde_json::to_vec(value).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        replace(&temporary, path).map_err(|e| format!("Cannot save preference: {e}"))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(windows)]
fn replace(from: &Path, to: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };
    let from: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: both paths are NUL-terminated buffers valid for this call.
    unsafe {
        MoveFileExW(
            windows_core::PCWSTR(from.as_ptr()),
            windows_core::PCWSTR(to.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    }
    .map_err(|e| e.to_string())
}
#[cfg(not(windows))]
fn replace(from: &Path, to: &Path) -> Result<(), String> {
    fs::rename(from, to).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_corruption_and_unknown_fields_enable_warning() {
        assert!(!Settings::default().skip_close_confirmation);
        assert!(
            serde_json::from_str::<Settings>(r#"{"skip_close_confirmation":true,"messages":[]}"#)
                .is_err()
        );
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        fs::create_dir(&root).unwrap();
        let path = root.join("settings.json");
        assert_eq!(load(&path), (Settings::default(), None));
        fs::write(&path, "invalid").unwrap();
        let (settings, warning) = load(&path);
        assert!(!settings.skip_close_confirmation && warning.is_some());
        fs::remove_file(path).unwrap();
        fs::remove_dir(root).unwrap();
    }
    #[test]
    fn preference_is_replaced_and_only_boolean_is_stored() {
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let path = root.join("settings.json");
        save(
            &path,
            &Settings {
                skip_close_confirmation: true,
                ..Settings::default()
            },
        )
        .unwrap();
        assert!(load(&path).0.skip_close_confirmation);
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            r#"{"skip_close_confirmation":true,"clipboard_fallback_enabled":false}"#
        );
        save(&path, &Settings::default()).unwrap();
        assert_eq!(load(&path).0, Settings::default());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_file(path).unwrap();
        fs::remove_dir(root).unwrap();
    }
    #[test]
    fn checkbox_is_committed_only_on_confirm() {
        let current = Settings::default();
        assert_eq!(current.after_decision(false, true), current);
        assert_eq!(current.after_decision(false, false), current);
        assert_eq!(current.after_decision(true, false), current);
        assert!(current.after_decision(true, true).skip_close_confirmation);
    }
    #[test]
    fn old_settings_keep_copy_disabled_and_warning_changes_preserve_opt_in() {
        let old: Settings = serde_json::from_str(r#"{"skip_close_confirmation":true}"#).unwrap();
        assert!(!old.clipboard_fallback_enabled);
        let enabled = Settings {
            clipboard_fallback_enabled: true,
            ..Settings::default()
        };
        assert!(
            enabled
                .after_decision(true, true)
                .clipboard_fallback_enabled
        );
        assert!(
            enabled
                .after_decision(false, false)
                .clipboard_fallback_enabled
        );
    }
}
