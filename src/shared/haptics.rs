//! Haptic feedback — a settings-gated tactile layer over UIKit.
//!
//! Default ON; «Ощущения → Вибрация» in Settings flips `app.haptics`. Every
//! call site is a UIKit event handler, so the generators are touched straight
//! on the main thread; other platforms (and a lost main thread) stay silent
//! no-ops.

use crate::shared::nslog;

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
    impact(true);
}

/// The «+» peek panel flips — a firmer tap.
pub fn pop() {
    impact(false);
}

/// A light tick: mode tabs, the quarter picker, an accent pick.
pub fn tick() {
    #[cfg(target_os = "ios")]
    {
        if !is_enabled() {
            return;
        }
        use objc2::MainThreadMarker;
        use objc2_ui_kit::UISelectionFeedbackGenerator;
        if let Some(mtm) = MainThreadMarker::new() {
            let g = UISelectionFeedbackGenerator::new(mtm);
            g.prepare();
            g.selectionChanged();
        }
    }
}

fn impact(light: bool) {
    #[cfg(target_os = "ios")]
    {
        if !is_enabled() {
            return;
        }
        use objc2::{MainThreadMarker, MainThreadOnly};
        use objc2_ui_kit::{UIImpactFeedbackGenerator, UIImpactFeedbackStyle};
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let style = if light {
            UIImpactFeedbackStyle::Light
        } else {
            UIImpactFeedbackStyle::Medium
        };
        // initWithStyle is THE designated initializer — the generated
        // `#[deprecated]` mirrors a header annotation, not the real API.
        #[allow(deprecated)]
        let g = UIImpactFeedbackGenerator::initWithStyle(
            UIImpactFeedbackGenerator::alloc(mtm),
            style,
        );
        g.prepare();
        g.impactOccurred();
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = light;
    }
}
