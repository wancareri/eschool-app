//! Haptic feedback — powered by day::haptics.

use crate::shared::nslog;
pub use day::Haptic;

const HAPTICS_KEY: &str = "app.haptics";

/// On unless explicitly turned off in Settings.
pub fn is_enabled() -> bool {
    day::prefs::get(HAPTICS_KEY).map(|v| v != "false").unwrap_or(true)
}

pub fn set_enabled(on: bool) {
    day::prefs::set(HAPTICS_KEY, if on { "true" } else { "false" });
    nslog::nslog(&format!("[Haptics] enabled={on}"));
}

/// A week page commits under the finger — a light thump.
pub fn bump() {
    if is_enabled() {
        day::haptics::play(Haptic::Light);
    }
}

/// The «+» peek panel flips — a firmer tap.
pub fn pop() {
    if is_enabled() {
        day::haptics::play(Haptic::Medium);
    }
}

/// A light tick: mode tabs, the quarter picker, an accent pick.
pub fn tick() {
    if is_enabled() {
        day::haptics::play(Haptic::Selection);
    }
}
