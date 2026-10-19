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

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn dlsym(handle: *mut std::ffi::c_void, symbol: *const std::ffi::c_char) -> *mut std::ffi::c_void;
}

#[cfg(target_os = "ios")]
fn play_system_sound(sound_id: u32) {
    type FnType = unsafe extern "C" fn(u32);
    unsafe {
        let sym = dlsym(
            (-2isize) as *mut std::ffi::c_void,
            b"AudioServicesPlaySystemSound\0".as_ptr() as *const _,
        );
        if !sym.is_null() {
            let func: FnType = std::mem::transmute(sym);
            func(sound_id);
        }
    }
}

/// A week page commits under the finger — a light thump.
pub fn bump() {
    impact(true);
    #[cfg(target_os = "ios")]
    {
        if is_enabled() {
            play_system_sound(1519);
        }
    }
}

/// The «+» peek panel flips — a firmer tap.
pub fn pop() {
    impact(false);
    #[cfg(target_os = "ios")]
    {
        if is_enabled() {
            play_system_sound(1520);
        }
    }
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

#[cfg(target_os = "ios")]
thread_local! {
    static GENERATOR_HEAVY: std::cell::RefCell<Option<objc2::rc::Retained<objc2_ui_kit::UIImpactFeedbackGenerator>>> = std::cell::RefCell::new(None);
    static GENERATOR_LIGHT: std::cell::RefCell<Option<objc2::rc::Retained<objc2_ui_kit::UIImpactFeedbackGenerator>>> = std::cell::RefCell::new(None);
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
        if light {
            GENERATOR_LIGHT.with(|cell| {
                let mut opt = cell.borrow_mut();
                if opt.is_none() {
                    #[allow(deprecated)]
                    let g = UIImpactFeedbackGenerator::initWithStyle(
                        UIImpactFeedbackGenerator::alloc(mtm),
                        UIImpactFeedbackStyle::Light,
                    );
                    *opt = Some(g);
                }
                if let Some(g) = opt.as_ref() {
                    g.prepare();
                    g.impactOccurred();
                }
            });
        } else {
            GENERATOR_HEAVY.with(|cell| {
                let mut opt = cell.borrow_mut();
                if opt.is_none() {
                    #[allow(deprecated)]
                    let g = UIImpactFeedbackGenerator::initWithStyle(
                        UIImpactFeedbackGenerator::alloc(mtm),
                        UIImpactFeedbackStyle::Heavy,
                    );
                    *opt = Some(g);
                }
                if let Some(g) = opt.as_ref() {
                    g.prepare();
                    g.impactOccurred();
                }
            });
        }
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = light;
    }
}
