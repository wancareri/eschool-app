use day::prelude::Color;
use std::sync::atomic::{AtomicU32, Ordering};

/// Global accent color — updated atomically when user picks a new color.
static ACCENT_HEX: AtomicU32 = AtomicU32::new(0x3B82F6);

/// Initialize the global accent color from prefs (call once at startup).
pub fn init_accent() {
    let hex = day::prefs::get("app.accent_color")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0x3B82F6);
    ACCENT_HEX.store(hex, Ordering::Relaxed);
}

/// Set the global accent color (call from settings when user picks a new color).
pub fn set_accent(hex: u32) {
    ACCENT_HEX.store(hex, Ordering::Relaxed);
}

/// Apply tint color to ALL iOS windows — makes native UIKit elements use accent color.
/// Call this AFTER the window is created (from the nav builder or root).
#[cfg(target_os = "ios")]
pub fn apply_ios_tint(hex: u32) {
    let r = ((hex >> 16) & 0xFF) as f64 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f64 / 255.0;
    let b = (hex & 0xFF) as f64 / 255.0;
    unsafe {
        let app: objc2::rc::Retained<objc2_ui_kit::UIApplication> =
            objc2::msg_send![objc2::class!(UIApplication), sharedApplication];
        let color = objc2_ui_kit::UIColor::colorWithRed_green_blue_alpha(
            r as objc2_core_foundation::CGFloat,
            g as objc2_core_foundation::CGFloat,
            b as objc2_core_foundation::CGFloat,
            1.0 as objc2_core_foundation::CGFloat,
        );
        // Iterate ALL connected scenes and ALL windows in each scene
        let scenes = app.connectedScenes();
        let enumerator: objc2::rc::Retained<objc2_foundation::NSEnumerator> =
            objc2::msg_send![&*scenes, objectEnumerator];
        while let Some(scene_obj) = enumerator.nextObject() {
            // Try downcast to UIWindowScene
            let is_window_scene: bool =
                objc2::msg_send![&*scene_obj, isKindOfClass: objc2::class!(UIWindowScene)];
            if is_window_scene {
                let windows: objc2::rc::Retained<objc2_foundation::NSArray<objc2_ui_kit::UIWindow>> =
                    objc2::msg_send![&*scene_obj, windows];
                let win_enumerator: objc2::rc::Retained<objc2_foundation::NSEnumerator> =
                    objc2::msg_send![&*windows, objectEnumerator];
                while let Some(win_obj) = win_enumerator.nextObject() {
                    let obj: &objc2::runtime::AnyObject = &*win_obj;
                    let window = &*(obj as *const objc2::runtime::AnyObject as *const objc2_ui_kit::UIWindow);
                    window.setTintColor(Some(&color));
                }
            }
        }
    }
}

#[cfg(not(target_os = "ios"))]
pub fn apply_ios_tint(_hex: u32) {}

/// Get current accent hex.
pub fn accent_hex() -> u32 {
    ACCENT_HEX.load(Ordering::Relaxed)
}

/// Get current accent color as Day Color.
pub fn primary() -> Color {
    Color::hex(ACCENT_HEX.load(Ordering::Relaxed))
}

/// Default accent hex value.
pub const DEFAULT_ACCENT: u32 = 0x3B82F6;

// ── Named accent colors ────────────────────────────────────────────────
pub const BLUE: u32 = 0x3B82F6;
pub const GREEN: u32 = 0x10B981;
pub const PURPLE: u32 = 0x8B5CF6;
pub const ORANGE: u32 = 0xF97316;
pub const RED: u32 = 0xEF4444;

// ── Static colors ──────────────────────────────────────────────────────
pub const SECONDARY: Color = Color::hex(0x6B7280);
pub const ACCENT: Color = Color::hex(0xF59E0B);
pub const BG: Color = Color::hex(0xF8FAFC);
pub const CARD: Color = Color::hex(0xFFFFFF);
pub const BORDER: Color = Color::hex(0xE2E8F0);

// ── Status ─────────────────────────────────────────────────────────────
pub const SUCCESS: Color = Color::hex(0x10B981);
pub const ERROR: Color = Color::hex(0xEF4444);
pub const WARNING: Color = Color::hex(0xF59E0B);
pub const INFO: Color = Color::hex(0x3B82F6);

// ── Gray ───────────────────────────────────────────────────────────────
pub const GRAY_100: Color = Color::hex(0xF1F5F9);
pub const GRAY_200: Color = Color::hex(0xE2E8F0);
pub const GRAY_300: Color = Color::hex(0xCBd5E1);
pub const GRAY_400: Color = Color::hex(0x94A3B8);
pub const GRAY_500: Color = Color::hex(0x64748B);
pub const GRAY_600: Color = Color::hex(0x475569);
pub const GRAY_700: Color = Color::hex(0x334155);
pub const GRAY_800: Color = Color::hex(0x1E293B);
pub const GRAY_900: Color = Color::hex(0x0F172A);
pub const WHITE: Color = Color::hex(0xFFFFFF);

// ── Grades ─────────────────────────────────────────────────────────────
pub const GRADE_EXCELLENT: Color = Color::hex(0x10B981);
pub const GRADE_GOOD: Color = Color::hex(0x3B82F6);
pub const GRADE_SATISFACTORY: Color = Color::hex(0xF59E0B);
pub const GRADE_POOR: Color = Color::hex(0xF97316);
pub const GRADE_FAILING: Color = Color::hex(0xEF4444);

// ── Navigation ─────────────────────────────────────────────────────────
pub const NAV_DIARY: Color = Color::hex(0x3B82F6);
pub const NAV_SCHEDULE: Color = Color::hex(0x10B981);
pub const NAV_TEACHERS: Color = Color::hex(0xF59E0B);
pub const NAV_SETTINGS: Color = Color::hex(0x6B7280);

// Backward compat
pub const PRIMARY: Color = Color::hex(0x3B82F6);
