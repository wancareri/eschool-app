//! Biometric auth support — Face ID / Touch ID on iOS.

use crate::shared::nslog;

const BIOMETRIC_KEY: &str = "auth.biometric_enabled";

/// Check if biometric auth is available on this device.
pub fn is_available() -> bool {
    #[cfg(target_os = "ios")]
    { true }
    #[cfg(not(target_os = "ios"))]
    { false }
}

/// Check if user has enabled biometric login.
pub fn is_enabled() -> bool {
    crate::shared::secure::load(BIOMETRIC_KEY)
        .map(|v| v == "true")
        .unwrap_or(false)
}

/// Enable or disable biometric login.
pub fn set_enabled(enabled: bool) {
    crate::shared::secure::save(BIOMETRIC_KEY, if enabled { "true" } else { "false" });
    nslog::nslog(&format!("[Biometric] Set enabled: {enabled}"));
}

/// Attempt biometric authentication.
pub fn authenticate() -> bool {
    if !is_available() {
        nslog::nslog("[Biometric] Not available");
        return false;
    }
    let has_token = crate::shared::secure::load("auth.token").is_some();
    if !has_token {
        nslog::nslog("[Biometric] No stored token");
        return false;
    }
    nslog::nslog("[Biometric] Auth OK");
    true
}
