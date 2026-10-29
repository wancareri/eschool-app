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

/// Apply tint color to ALL iOS native UIKit elements via UIAppearance proxies.
/// This is the standard way to set global tint in UIKit — affects UINavigationBar,
/// UITabBar, UIButton, UISwitch, etc. without needing to find windows.
#[cfg(target_os = "ios")]
pub fn apply_ios_tint(hex: u32) {
    let r = ((hex >> 16) & 0xFF) as f64 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f64 / 255.0;
    let b = (hex & 0xFF) as f64 / 255.0;
    unsafe {
        let color = objc2_ui_kit::UIColor::colorWithRed_green_blue_alpha(
            r as objc2_core_foundation::CGFloat,
            g as objc2_core_foundation::CGFloat,
            b as objc2_core_foundation::CGFloat,
            1.0 as objc2_core_foundation::CGFloat,
        );
        let clear = objc2_ui_kit::UIColor::clearColor();
        let blur_class = objc2::class!(UIBlurEffect);
        let blur: objc2::rc::Retained<objc2::runtime::AnyObject> =
            objc2::msg_send![blur_class, effectWithStyle: 6isize]; // SystemUltraThinMaterial

        // UITabBarAppearance — liquid glass with transparent ground (no solid color underneath)
        let tab_appearance_class = objc2::class!(UITabBarAppearance);
        let tab_app: objc2::rc::Retained<objc2::runtime::AnyObject> =
            objc2::msg_send![tab_appearance_class, new];
        let _: () = objc2::msg_send![&*tab_app, configureWithTransparentBackground];
        let _: () = objc2::msg_send![&*tab_app, setBackgroundEffect: Some(&*blur)];
        let _: () = objc2::msg_send![&*tab_app, setBackgroundColor: &*clear];
        let _: () = objc2::msg_send![&*tab_app, setShadowColor: &*clear];

        // UITabBar.appearance()
        let tab_appearance: objc2::rc::Retained<objc2::runtime::AnyObject> =
            objc2::msg_send![objc2::class!(UITabBar), appearance];
        let _: () = objc2::msg_send![&*tab_appearance, setTintColor: &*color];
        let _: () = objc2::msg_send![&*tab_appearance, setStandardAppearance: &*tab_app];
        let _: () = objc2::msg_send![&*tab_appearance, setScrollEdgeAppearance: &*tab_app];

        // UINavigationBarAppearance — liquid glass with transparent ground
        let nav_appearance_class = objc2::class!(UINavigationBarAppearance);
        let nav_app: objc2::rc::Retained<objc2::runtime::AnyObject> =
            objc2::msg_send![nav_appearance_class, new];
        let _: () = objc2::msg_send![&*nav_app, configureWithTransparentBackground];
        let _: () = objc2::msg_send![&*nav_app, setBackgroundEffect: Some(&*blur)];
        let _: () = objc2::msg_send![&*nav_app, setBackgroundColor: &*clear];
        let _: () = objc2::msg_send![&*nav_app, setShadowColor: &*clear];

        // UINavigationBar.appearance()
        let nav_appearance: objc2::rc::Retained<objc2::runtime::AnyObject> =
            objc2::msg_send![objc2::class!(UINavigationBar), appearance];
        let _: () = objc2::msg_send![&*nav_appearance, setTintColor: &*color];
        let _: () = objc2::msg_send![&*nav_appearance, setStandardAppearance: &*nav_app];
        let _: () = objc2::msg_send![&*nav_appearance, setScrollEdgeAppearance: &*nav_app];
        let _: () = objc2::msg_send![&*nav_appearance, setCompactAppearance: &*nav_app];
        let _: () = objc2::msg_send![&*nav_appearance, setCompactScrollEdgeAppearance: &*nav_app];

        // UISwitch.appearance().onTintColor — switch accent color
        let switch_appearance: objc2::rc::Retained<objc2::runtime::AnyObject> =
            objc2::msg_send![objc2::class!(UISwitch), appearance];
        let _: () = objc2::msg_send![&*switch_appearance, setOnTintColor: &*color];
        // UIBarButtonItem.appearance().tintColor — bar button items
        let bar_btn_appearance: objc2::rc::Retained<objc2::runtime::AnyObject> =
            objc2::msg_send![objc2::class!(UIBarButtonItem), appearance];
        let _: () = objc2::msg_send![&*bar_btn_appearance, setTintColor: &*color];

        // Apply tintColor and transparent bar styling to all active windows (scenes + legacy windows)
        let app: objc2::rc::Retained<objc2::runtime::AnyObject> =
            objc2::msg_send![objc2::class!(UIApplication), sharedApplication];
        let scenes: Option<objc2::rc::Retained<objc2_foundation::NSArray<objc2::runtime::AnyObject>>> =
            objc2::msg_send![&*app, connectedScenes];
        if let Some(scene_arr) = scenes {
            for sc in scene_arr.iter() {
                let windows: Option<objc2::rc::Retained<objc2_foundation::NSArray<objc2::runtime::AnyObject>>> =
                    objc2::msg_send![&*sc, windows];
                if let Some(win_arr) = windows {
                    for win in win_arr.iter() {
                        let _: () = objc2::msg_send![&*win, setTintColor: &*color];
                        if let Some(view) = win.downcast_ref::<objc2_ui_kit::UIView>() {
                            retint_existing(view, &color);
                        }
                    }
                }
            }
        }
        #[allow(deprecated)]
        let windows: Option<objc2::rc::Retained<objc2_foundation::NSArray<objc2::runtime::AnyObject>>> =
            objc2::msg_send![&*app, windows];
        if let Some(win_arr) = windows {
            for win in win_arr.iter() {
                let _: () = objc2::msg_send![&*win, setTintColor: &*color];
                if let Some(view) = win.downcast_ref::<objc2_ui_kit::UIView>() {
                    retint_existing(view, &color);
                }
            }
        }
        if let Some(mtm) = objc2::MainThreadMarker::new() {
            #[allow(deprecated)]
            if let Some(win) = objc2_ui_kit::UIApplication::sharedApplication(mtm).keyWindow() {
                retint_existing(&win, &color);
            }
        }
    }
}

/// Recursively retint live UIKit classes and enforce liquid glass transparency.
#[cfg(target_os = "ios")]
unsafe fn retint_existing(view: &objc2_ui_kit::UIView, color: &objc2_ui_kit::UIColor) {
    let clear = objc2_ui_kit::UIColor::clearColor();
    let obj: &objc2::runtime::AnyObject = view.as_ref();

    if let Some(bar) = obj.downcast_ref::<objc2_ui_kit::UITabBar>() {
        unsafe {
            bar.setTintColor(Some(color));
            bar.setTranslucent(true);
            bar.setBackgroundColor(Some(&clear));

            let tab_appearance_class = objc2::class!(UITabBarAppearance);
            let tab_app: objc2::rc::Retained<objc2::runtime::AnyObject> =
                objc2::msg_send![tab_appearance_class, new];
            let blur_class = objc2::class!(UIBlurEffect);
            let blur: objc2::rc::Retained<objc2::runtime::AnyObject> =
                objc2::msg_send![blur_class, effectWithStyle: 6isize];
            let _: () = objc2::msg_send![&*tab_app, configureWithTransparentBackground];
            let _: () = objc2::msg_send![&*tab_app, setBackgroundEffect: Some(&*blur)];
            let _: () = objc2::msg_send![&*tab_app, setBackgroundColor: &*clear];
            let _: () = objc2::msg_send![&*tab_app, setShadowColor: &*clear];

            let _: () = objc2::msg_send![bar, setStandardAppearance: &*tab_app];
            let _: () = objc2::msg_send![bar, setScrollEdgeAppearance: &*tab_app];
        };
    } else if let Some(bar) = obj.downcast_ref::<objc2_ui_kit::UINavigationBar>() {
        unsafe {
            bar.setTintColor(Some(color));
            bar.setTranslucent(true);
            bar.setBackgroundColor(Some(&clear));

            let nav_appearance_class = objc2::class!(UINavigationBarAppearance);
            let nav_app: objc2::rc::Retained<objc2::runtime::AnyObject> =
                objc2::msg_send![nav_appearance_class, new];
            let blur_class = objc2::class!(UIBlurEffect);
            let blur: objc2::rc::Retained<objc2::runtime::AnyObject> =
                objc2::msg_send![blur_class, effectWithStyle: 6isize];
            let _: () = objc2::msg_send![&*nav_app, configureWithTransparentBackground];
            let _: () = objc2::msg_send![&*nav_app, setBackgroundEffect: Some(&*blur)];
            let _: () = objc2::msg_send![&*nav_app, setBackgroundColor: &*clear];
            let _: () = objc2::msg_send![&*nav_app, setShadowColor: &*clear];

            let _: () = objc2::msg_send![bar, setStandardAppearance: &*nav_app];
            let _: () = objc2::msg_send![bar, setScrollEdgeAppearance: &*nav_app];
            let _: () = objc2::msg_send![bar, setCompactAppearance: &*nav_app];
            let _: () = objc2::msg_send![bar, setCompactScrollEdgeAppearance: &*nav_app];
        };
    } else if let Some(sw) = obj.downcast_ref::<objc2_ui_kit::UISwitch>() {
        sw.setOnTintColor(Some(color));
    }

    // Clear background for DayNavPageView so the area beneath the tab bar
    // and status bar isn't a solid opaque rectangle.
    let class_name = view.class().name().to_str().unwrap_or("");
    if class_name == "DayNavPageView" {
        view.setBackgroundColor(Some(&clear));
    }

    for sub in view.subviews().iter() {
        unsafe { retint_existing(&sub, color) };
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
