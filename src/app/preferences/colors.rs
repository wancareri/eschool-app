use day::prelude::Color;
use std::sync::atomic::{AtomicU32, Ordering};

/// Global accent color — updated atomically when user picks a new color.
static ACCENT_HEX: AtomicU32 = AtomicU32::new(0x3B82F6);

/// Prefer iOS 26's clear Liquid Glass. Older UIKit versions fall back to
/// ultra-thin material because UIGlassEffect is not available there.
#[cfg(feature = "uikit")]
fn clear_glass_effect(mtm: objc2::MainThreadMarker) -> objc2::rc::Retained<objc2_ui_kit::UIVisualEffect> {
    if let Some(class) = objc2::runtime::AnyClass::get(c"UIGlassEffect") {
        let effect: objc2::rc::Retained<objc2::runtime::AnyObject> = unsafe {
            objc2::msg_send![class, effectWithStyle: 1isize] // UIGlassEffectStyleClear
        };
        if let Ok(effect) = effect.downcast::<objc2_ui_kit::UIVisualEffect>() {
            return effect;
        }
    }

    objc2_ui_kit::UIBlurEffect::effectWithStyle(
        objc2_ui_kit::UIBlurEffectStyle::SystemUltraThinMaterial,
        mtm,
    )
    .into_super()
}

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
#[cfg(feature = "uikit")]
pub fn apply_ios_tint(hex: u32) {
    let Some(mtm) = objc2::MainThreadMarker::new() else {
        return;
    };
    let nav_glass = clear_glass_effect(mtm);
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

        // Keep the standard appearance for all pages; the Diary's live bar receives its
        // dedicated gradient glass backdrop below.
        let nav_appearance_class = objc2::class!(UINavigationBarAppearance);
        let nav_app: objc2::rc::Retained<objc2::runtime::AnyObject> =
            objc2::msg_send![nav_appearance_class, new];
        let _: () = objc2::msg_send![&*nav_app, configureWithOpaqueBackground];
        let _: () = objc2::msg_send![&*nav_app, setBackgroundColor: &*objc2_ui_kit::UIColor::systemBackgroundColor()];

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
                            retint_existing(view, &color, &nav_glass, mtm);
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
                    retint_existing(view, &color, &nav_glass, mtm);
                }
            }
        }
        #[allow(deprecated)]
        if let Some(win) = objc2_ui_kit::UIApplication::sharedApplication(mtm).keyWindow() {
            retint_existing(&win, &color, &nav_glass, mtm);
        }
    }
}

/// Recursively retint live UIKit classes and enforce liquid glass transparency.
#[cfg(feature = "uikit")]
unsafe fn retint_existing(
    view: &objc2_ui_kit::UIView,
    color: &objc2_ui_kit::UIColor,
    nav_glass: &objc2_ui_kit::UIVisualEffect,
    mtm: objc2::MainThreadMarker,
) {
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
            let diary_title = crate::res::str::diary_title().format();
            let is_diary = bar
                .topItem()
                .and_then(|item| item.title())
                .is_some_and(|title| title.to_string() == diary_title);
            bar.setTintColor(Some(color));
            bar.setTranslucent(is_diary);
            let bar_background = if is_diary {
                clear.clone()
            } else {
                objc2_ui_kit::UIColor::systemBackgroundColor()
            };
            bar.setBackgroundColor(Some(&bar_background));

            let nav_appearance_class = objc2::class!(UINavigationBarAppearance);
            let nav_app: objc2::rc::Retained<objc2::runtime::AnyObject> =
                objc2::msg_send![nav_appearance_class, new];
            if is_diary {
                let _: () = objc2::msg_send![&*nav_app, configureWithTransparentBackground];
                let _: () = objc2::msg_send![&*nav_app, setBackgroundColor: &*clear];
                let _: () = objc2::msg_send![&*nav_app, setShadowColor: &*clear];
            } else {
                let _: () = objc2::msg_send![&*nav_app, configureWithOpaqueBackground];
                let _: () = objc2::msg_send![&*nav_app, setBackgroundColor: &*bar_background];
            }

            let _: () = objc2::msg_send![bar, setStandardAppearance: &*nav_app];
            let _: () = objc2::msg_send![bar, setScrollEdgeAppearance: &*nav_app];
            let _: () = objc2::msg_send![bar, setCompactAppearance: &*nav_app];
            let _: () = objc2::msg_send![bar, setCompactScrollEdgeAppearance: &*nav_app];
            if is_diary {
                let empty_shadow = objc2_ui_kit::UIImage::new();
                bar.setShadowImage(Some(&empty_shadow));
                crate::ios::nav_glass::apply(bar, nav_glass, mtm);
            }

            // Give only the Diary title a directly attached rounded blur surface. No
            // border or extra shell separates the native title from its glass material.
            if let Some(item) = bar.topItem()
                && let Some(title) = item.titleView()
                && let Some(label) = title.downcast_ref::<objc2_ui_kit::UILabel>()
                && label.text().is_some_and(|text| text.to_string() == crate::res::str::diary_title().format())
            {
                use objc2_core_foundation::{CGPoint, CGRect, CGSize};
                use objc2_ui_kit::UIVisualEffectView;
                use objc2::MainThreadOnly;
                let label_size = label.frame().size;
                let width = label_size.width + 24.0;
                let height = label_size.height + 10.0;
                let mtm = objc2::MainThreadMarker::new().expect("main thread");
                let effect = clear_glass_effect(mtm);
                let blur = UIVisualEffectView::initWithEffect(
                    UIVisualEffectView::alloc(mtm),
                    Some(&effect),
                );
                blur.setFrame(CGRect::new(CGPoint::ZERO, CGSize::new(width, height)));
                blur.setClipsToBounds(true);
                blur.layer().setCornerRadius((height / 2.0) as _);
                let content = blur.contentView();
                label.setFrame(CGRect::new(
                    CGPoint::new(12.0, 5.0),
                    CGSize::new(label_size.width, label_size.height),
                ));
                content.addSubview(label);
                item.setTitleView(Some(&blur));
            }
        };
    } else if let Some(sw) = obj.downcast_ref::<objc2_ui_kit::UISwitch>() {
        sw.setOnTintColor(Some(color));
    }

    // Preserve modal cover backgrounds during appearance changes; page
    // containers still stay transparent beneath the tab and status bars.
    let mut cur: Option<objc2::rc::Retained<objc2_ui_kit::UIResponder>> = view.nextResponder();
    let mut is_cover = false;
    while let Some(responder) = cur {
        if responder.class().name().to_bytes() == b"DayCoverVC" {
            is_cover = true;
            break;
        }
        cur = responder.nextResponder();
    }
    if !is_cover {
        let class_name = view.class().name().to_str().unwrap_or("");
        if class_name == "DayNavPageView" || class_name == "DayHolderView" || class_name == "UINavigationTransitionView" {
            view.setBackgroundColor(Some(&clear));
        }
    }

    for sub in view.subviews().iter() {
        unsafe { retint_existing(&sub, color, nav_glass, mtm) };
    }
}

#[cfg(not(feature = "uikit"))]
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
