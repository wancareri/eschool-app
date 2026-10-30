//! Biometric auth support — powered by day::biometrics (Face ID, Touch ID, Android Biometrics, Windows Hello).

use crate::shared::nslog;

const BIOMETRIC_KEY: &str = "auth.biometric_enabled";

/// Check if biometric auth is available on this device.
pub fn is_available() -> bool {
    let available = day::biometrics::is_available();
    nslog::nslog(&format!("[Biometric] is_available: {available}"));
    available
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

/// Trigger Face ID / Touch ID / biometric prompt.
pub fn authenticate_async(_state: crate::app::AppState) {
    let set_biometric_ok = _state.biometric_ok.setter();
    nslog::nslog("[Biometric] authenticate_async: calling day::biometrics::authenticate");
    day::biometrics::authenticate("Вход в приложение", move |ok, _err| {
        nslog::nslog(&format!("[Biometric] Reply: success={ok}"));
        set_biometric_ok.set(ok);
    });
}
