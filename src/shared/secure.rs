//! Secure credential storage via platform keychain/keystore.
//! iOS: Keychain (via `keyring` crate)
//! Other platforms: `day::prefs` (plain text, fallback)

#[allow(unused_imports)]
use crate::shared::nslog;

#[allow(dead_code)]
const SERVICE: &str = "by.eschool.app";

/// Save a string to secure storage.
pub fn save(key: &str, value: &str) {
    #[cfg(target_os = "ios")]
    {
        match keyring::Entry::new(SERVICE, key)
            .and_then(|e| e.set_password(value))
        {
            Ok(()) => nslog::nslog(&format!("[Keychain] Saved: {key}")),
            Err(e) => nslog::nslog(&format!("[Keychain] Save failed: {e}")),
        }
    }
    #[cfg(not(target_os = "ios"))]
    {
        day::prefs::set(key, value);
    }
}

/// Load a string from secure storage.
pub fn load(key: &str) -> Option<String> {
    #[cfg(target_os = "ios")]
    {
        match keyring::Entry::new(SERVICE, key)
            .and_then(|e| e.get_password())
        {
            Ok(v) if !v.is_empty() => Some(v),
            _ => None,
        }
    }
    #[cfg(not(target_os = "ios"))]
    {
        day::prefs::get(key).filter(|v| !v.is_empty())
    }
}

/// Delete a key from secure storage.
pub fn delete(key: &str) {
    #[cfg(target_os = "ios")]
    {
        match keyring::Entry::new(SERVICE, key)
            .and_then(|e| e.delete_credential())
        {
            Ok(()) => nslog::nslog(&format!("[Keychain] Deleted: {key}")),
            Err(e) => nslog::nslog(&format!("[Keychain] Delete failed: {e}")),
        }
    }
    #[cfg(not(target_os = "ios"))]
    {
        day::prefs::set(key, "");
    }
}
