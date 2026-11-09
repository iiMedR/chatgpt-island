use std::path::{Path, PathBuf};
pub const CHATGPT_URL: &str = "https://chatgpt.com/";

pub fn create_profile(root: &Path) -> Result<(String, PathBuf), String> {
    std::fs::create_dir_all(root).map_err(|e| format!("Cannot create session root: {e}"))?;
    let id = format!("island-{}", uuid::Uuid::new_v4());
    let path = root.join(&id);
    // Atomic directory creation: never reuse an existing directory.
    std::fs::create_dir(&path).map_err(|e| format!("Cannot create private profile: {e}"))?;
    Ok((id, path))
}

pub fn allowed_navigation(url: &tauri::Url) -> bool {
    url.scheme() == "https"
        && url.host_str() == Some("chatgpt.com")
        && url.port_or_known_default() == Some(443)
        && url.username().is_empty()
        && url.password().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_chatgpt_https_navigation_is_allowed() {
        assert!(allowed_navigation(&CHATGPT_URL.parse().unwrap()));
        assert!(allowed_navigation(
            &"https://chatgpt.com/c/test".parse().unwrap()
        ));
        for value in [
            "http://chatgpt.com",
            "https://chatgpt.com.evil.test",
            "https://evil.test",
            "tauri://localhost",
            "https://chatgpt.com:444",
            "https://user@chatgpt.com",
        ] {
            assert!(!allowed_navigation(&value.parse().unwrap()), "{value}");
        }
    }
    #[test]
    fn profiles_are_unique() {
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let (a, pa) = create_profile(&root).unwrap();
        let (b, pb) = create_profile(&root).unwrap();
        assert_ne!(a, b);
        assert_ne!(pa, pb);
        assert!(pa.is_dir() && pb.is_dir());
        // These empty test directories were never opened by WebView2.
        std::fs::remove_dir(pa).unwrap();
        std::fs::remove_dir(pb).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn profile_creation_does_not_overwrite_an_existing_file() {
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        std::fs::write(&root, "unrelated sentinel").unwrap();
        assert!(create_profile(&root).is_err());
        assert_eq!(
            std::fs::read_to_string(&root).unwrap(),
            "unrelated sentinel"
        );
        std::fs::remove_file(root).unwrap();
    }
}
