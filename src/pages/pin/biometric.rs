//! Biometric auth support — powered by day::biometrics (Face ID, Touch ID, Android Biometrics, Windows Hello).

use crate::ios::nslog;

const BIOMETRIC_KEY: &str = "auth.biometric_enabled";

/// Check if biometric auth is available on this device.
pub fn is_available() -> bool {
    let available = day_part_biometrics::availability() == day_part_biometrics::Availability::Available;
    nslog::nslog(&format!("[Biometric] is_available: {available}"));
    available
}

/// Check if user has enabled biometric login.
pub fn is_enabled() -> bool {
    crate::app::data::credentials::load(BIOMETRIC_KEY)
        .map(|v| v == "true")
        .unwrap_or(false)
}

/// Enable or disable biometric login.
pub fn set_enabled(enabled: bool) {
    crate::app::data::credentials::save(BIOMETRIC_KEY, if enabled { "true" } else { "false" });
    nslog::nslog(&format!("[Biometric] Set enabled: {enabled}"));
}

/// Trigger Face ID / Touch ID / biometric prompt.
pub fn authenticate_async(_state: crate::app::AppState) {
    let ticket = crate::app::data::session::ticket();
    nslog::nslog("[Biometric] authenticate_async: calling day::biometrics::authenticate");
    day_part_biometrics::authenticate(day_part_biometrics::Prompt::new("Вход в приложение"), move |result| {
        let ok = result.is_ok();
        nslog::nslog(&format!("[Biometric] Reply: success={ok}"));
        ticket.publish(move |state| state.biometric_ok.set(ok));
    });
}
