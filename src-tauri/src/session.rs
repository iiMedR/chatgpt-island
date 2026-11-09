use std::path::{Path, PathBuf};
pub const CHATGPT_URL: &str = "https://chatgpt.com/";

#[derive(Default)]
pub struct Session {
    active: Option<String>,
    closing: bool,
    ready: bool,
}
impl Session {
    pub fn begin(&mut self, id: String) -> Result<(), String> {
        if self.active.is_some() { return Err("A popup is active or being destroyed.".into()); }
        self.active = Some(id);
        self.ready = false;
        self.closing = false;
        Ok(())
    }
    pub fn active(&self) -> Option<&str> { self.active.as_deref() }
    pub fn opened(&mut self, id: &str) {
        if self.active() == Some(id) { self.ready = true; }
    }
    pub fn request_close(&mut self) -> Result<String, String> {
        if self.closing { return Err("Destruction already pending.".into()); }
        let id = self.active.clone().ok_or("No popup is open.")?;
        if !self.ready { return Err("Popup creation is still pending.".into()); }
        self.closing = true;
        Ok(id)
    }
    pub fn close_failed(&mut self) { self.closing = false; }
    pub fn release(&mut self, id: &str) {
        if self.active() == Some(id) { self.active = None; self.closing = false; self.ready = false; }
    }
}

pub fn create_profile(root: &Path) -> Result<(String, PathBuf), String> {
    std::fs::create_dir_all(root).map_err(|e| format!("Cannot create session root: {e}"))?;
    let id = format!("island-{}", uuid::Uuid::new_v4());
    let path = root.join(&id);
    // Atomic directory creation: never reuse an existing directory.
    std::fs::create_dir(&path).map_err(|e| format!("Cannot create private profile: {e}"))?;
    Ok((id, path))
}

pub fn allowed_navigation(url: &tauri::Url) -> bool {
    url.scheme() == "https" && url.host_str() == Some("chatgpt.com")
        && url.port_or_known_default() == Some(443)
        && url.username().is_empty() && url.password().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn destruction_must_finish_before_reopening() {
        let mut s = Session::default();
        assert!(s.request_close().is_err());
        s.begin("first".into()).unwrap();
        assert!(s.request_close().is_err());
        s.opened("first");
        assert!(s.begin("duplicate".into()).is_err());
        assert_eq!(s.request_close().unwrap(), "first");
        assert!(s.request_close().is_err());
        assert!(s.begin("early".into()).is_err());
        s.release("stale");
        assert_eq!(s.active(), Some("first"));
        s.release("first");
        assert!(s.active().is_none());
        s.begin("second".into()).unwrap();
    }
    #[test]
    fn failed_close_can_be_retried() {
        let mut s = Session::default();
        s.begin("first".into()).unwrap();
        assert!(s.request_close().is_err());
        s.opened("first");
        s.request_close().unwrap();
        s.close_failed();
        assert!(s.request_close().is_ok());
        s.release("first");
        assert!(s.begin("retry".into()).is_ok());
    }
    #[test]
    fn only_chatgpt_https_navigation_is_allowed() {
        assert!(allowed_navigation(&CHATGPT_URL.parse().unwrap()));
        assert!(allowed_navigation(&"https://chatgpt.com/c/test".parse().unwrap()));
        for value in ["http://chatgpt.com", "https://chatgpt.com.evil.test", "https://evil.test", "tauri://localhost", "https://chatgpt.com:444", "https://user@chatgpt.com"] {
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
}
