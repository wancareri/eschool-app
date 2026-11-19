
use crate::ios::nslog;
pub use day_part_haptics::Haptic;

const HAPTICS_KEY: &str = "app.haptics";

pub fn is_enabled() -> bool {
    day::prefs::get(HAPTICS_KEY).map(|v| v != "false").unwrap_or(true)
}

pub fn set_enabled(on: bool) {
    day::prefs::set(HAPTICS_KEY, if on { "true" } else { "false" });
    nslog::nslog(&format!("[Haptics] enabled={on}"));
}

pub fn bump() {
    if is_enabled() {
        day_part_haptics::play(Haptic::Light);
    }
}

pub fn pop() {
    if is_enabled() {
        day_part_haptics::play(Haptic::Medium);
    }
}

pub fn tick() {
    if is_enabled() {
        day_part_haptics::play(Haptic::Selection);
    }
}
