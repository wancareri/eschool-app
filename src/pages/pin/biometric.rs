
use crate::ios::nslog;

const BIOMETRIC_KEY: &str = "auth.biometric_enabled";

pub fn is_available() -> bool {
    let available = day_part_biometrics::availability() == day_part_biometrics::Availability::Available;
    nslog::nslog(&format!("[Biometric] is_available: {available}"));
    available
}

pub fn is_enabled() -> bool {
    crate::app::data::credentials::load(BIOMETRIC_KEY)
        .map(|v| v == "true")
        .unwrap_or(false)
}

pub fn set_enabled(enabled: bool) {
    crate::app::data::credentials::save(BIOMETRIC_KEY, if enabled { "true" } else { "false" });
    nslog::nslog(&format!("[Biometric] Set enabled: {enabled}"));
}

pub fn authenticate_async(_state: crate::app::AppState) {
    let ticket = crate::app::data::session::ticket();
    nslog::nslog("[Biometric] authenticate_async: calling day::biometrics::authenticate");
    day_part_biometrics::authenticate(day_part_biometrics::Prompt::new("Вход в приложение"), move |result| {
        let ok = result.is_ok();
        nslog::nslog(&format!("[Biometric] Reply: success={ok}"));
        ticket.publish(move |state| state.biometric_ok.set(ok));
    });
}
