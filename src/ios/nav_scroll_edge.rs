//! Connect the Diary's nested UIScrollView to UIKit's native navigation edge effect.

use dispatch2::{DispatchQueue, MainThreadBound};
use objc2::{MainThreadMarker, MainThreadOnly, rc::Weak, runtime::Sel};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_ui_kit::{
    NSDirectionalEdgeInsets, NSDirectionalRectEdge, UIAction, UIBlurEffect, UIBlurEffectStyle,
    UIButton, UIButtonConfiguration, UIButtonConfigurationCornerStyle, UIGlassEffect,
    UIGlassEffectStyle, UILabel, UINavigationBar, UINavigationBarAppearance,
    UINavigationController, UIScrollView, UIView, UIViewController, UIVisualEffectView,
};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};

const DIARY_BAR_STYLE_TAG: isize = 8_417_265;
const DIARY_TITLE_BUTTON_TAG: isize = 8_417_268;
static ORIGINAL_VIEW_DID_LAYOUT: AtomicPtr<()> = AtomicPtr::new(std::ptr::null_mut());
static ORIGINAL_NAV_BAR_LAYOUT: AtomicPtr<()> = AtomicPtr::new(std::ptr::null_mut());

/// Reapply the system appearance after Day configures its page controller's bar.
pub fn install_diary_navigation_hook() {
    static INSTALL: std::sync::Once = std::sync::Once::new();
    INSTALL.call_once(|| {
        let class = objc2::class!(UIViewController);
        let Some(method) = class.instance_method(objc2::sel!(viewDidLayoutSubviews)) else {
            return;
        };
        ORIGINAL_VIEW_DID_LAYOUT.store(method.implementation() as *mut (), Ordering::Release);
        unsafe {
            method.set_implementation(std::mem::transmute(diary_view_did_layout as *const ()));
        }
        let nav_class = objc2::class!(UINavigationBar);
        let Some(nav_method) = nav_class.instance_method(objc2::sel!(layoutSubviews)) else {
            return;
        };
        ORIGINAL_NAV_BAR_LAYOUT.store(nav_method.implementation() as *mut (), Ordering::Release);
        unsafe {
            nav_method.set_implementation(std::mem::transmute(nav_bar_layout as *const ()));
        }
    });
}

unsafe extern "C-unwind" fn nav_bar_layout(bar: &UINavigationBar, selector: Sel) {
    let original: unsafe extern "C-unwind" fn(&UINavigationBar, Sel) =
        unsafe { std::mem::transmute(ORIGINAL_NAV_BAR_LAYOUT.load(Ordering::Acquire)) };
    unsafe { original(bar, selector) };

    let mut responder = bar.nextResponder();
    while let Some(current) = responder {
        if let Some(nav_controller) = current.downcast_ref::<UINavigationController>() {
            let Some(controller) = nav_controller.topViewController() else {
                return;
            };
            if is_supported_item(bar.topItem().as_deref()) {
                apply_system_bar_appearance(bar);
                install_title_button(&controller, bar);
                schedule_title_button_retry(&controller);
            }
            return;
        }
        responder = current.nextResponder();
    }
}

unsafe extern "C-unwind" fn diary_view_did_layout(controller: &UIViewController, selector: Sel) {
    let original: unsafe extern "C-unwind" fn(&UIViewController, Sel) =
        unsafe { std::mem::transmute(ORIGINAL_VIEW_DID_LAYOUT.load(Ordering::Acquire)) };
    unsafe { original(controller, selector) };

    let nav_controller: Option<objc2::rc::Retained<UINavigationController>> =
        unsafe { objc2::msg_send![controller, navigationController] };
    let Some(nav_controller) = nav_controller else {
        return;
    };
    let bar = nav_controller.navigationBar();
    if !is_supported_controller(controller) {
        return;
    }
    install_title_button(controller, &bar);
    if bar.tag() == DIARY_BAR_STYLE_TAG {
        schedule_title_button_retry(controller);
        return;
    }
    apply_system_bar_appearance(&bar);
    schedule_title_button_retry(controller);
}

fn schedule_title_button_retry(controller: &UIViewController) {
    schedule_title_button_retry_with_remaining(controller, 3);
}

fn schedule_title_button_retry_with_remaining(controller: &UIViewController, remaining: u8) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let weak = MainThreadBound::new(Weak::new(controller), mtm);
    DispatchQueue::main().exec_async(move || {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let Some(controller) = weak.get(mtm).load() else {
            return;
        };
        let Some(nav_controller): Option<objc2::rc::Retained<UINavigationController>> =
            (unsafe { objc2::msg_send![&*controller, navigationController] })
        else {
            return;
        };
        let bar = nav_controller.navigationBar();
        install_title_button(&controller, &bar);
        if remaining > 0 {
            schedule_title_button_retry_with_remaining(&controller, remaining - 1);
        }
    });
}

/// Register the Diary scroll view after Day has mounted it in its page controller.
pub fn bind_diary_scroll(view: &objc2::rc::Retained<UIView>, _class: &str, mtm: MainThreadMarker) {
    let Some(scroll) = view.downcast_ref::<UIScrollView>() else {
        return;
    };
    configure(view, scroll);

    // Day may invoke the tweak before the page view is attached to its controller.
    // Recheck once on the next main-queue turn, scoped to this mounted scroll view.
    let weak = MainThreadBound::new(Weak::new(&**view), mtm);
    DispatchQueue::main().exec_async(move || {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let Some(view) = weak.get(mtm).load() else {
            return;
        };
        let Some(scroll) = view.downcast_ref::<UIScrollView>() else {
            return;
        };
        configure(&view, scroll);
    });
}

fn configure(view: &UIView, scroll: &UIScrollView) {
    let mut responder = view.nextResponder();
    while let Some(current) = responder {
        if let Some(controller) = current.downcast_ref::<UIViewController>() {
            let edge = NSDirectionalRectEdge::Top;
            controller.setContentScrollView_forEdge(Some(scroll), edge);

            let nav_controller: Option<objc2::rc::Retained<UINavigationController>> =
                unsafe { objc2::msg_send![controller, navigationController] };
            if let Some(nav_controller) = nav_controller {
                let bar: objc2::rc::Retained<UINavigationBar> = nav_controller.navigationBar();
                apply_system_bar_appearance(&bar);
                install_title_button(controller, &bar);
                return;
            }
        }
        responder = current.nextResponder();
    }
}

fn apply_system_bar_appearance(bar: &UINavigationBar) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let appearance = UINavigationBarAppearance::new(mtm);
    appearance.configureWithTransparentBackground();
    let clear = objc2_ui_kit::UIColor::clearColor();
    appearance.setBackgroundColor(Some(&clear));
    appearance.setShadowColor(Some(&clear));
    bar.setBackgroundColor(Some(&clear));
    bar.setStandardAppearance(&appearance);
    bar.setCompactAppearance(Some(&appearance));
    bar.setScrollEdgeAppearance(Some(&appearance));
    bar.setCompactScrollEdgeAppearance(Some(&appearance));
    bar.setTranslucent(true);
    bar.setTag(DIARY_BAR_STYLE_TAG);
}

/// Replace Day's non-interactive title label with a native glass button.
fn install_title_button(controller: &UIViewController, bar: &UINavigationBar) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let Some(item) = bar.topItem() else { return };
    let title_view = item.titleView();
    if title_view.as_ref().is_some_and(|view| view.tag() == DIARY_TITLE_BUTTON_TAG) {
        return;
    }
    let label = title_view
        .as_ref()
        .and_then(|view| view.downcast_ref::<UILabel>());
    let title = item
        .title()
        .or_else(|| label.and_then(|label| label.text()));
    let Some(title) = title else { return };
    if !is_app_page_title(&title.to_string()) {
        return;
    }

    let scroll = controller.contentScrollViewForEdge(NSDirectionalRectEdge::Top);
    let weak_scroll = scroll.as_ref().map(|scroll| Weak::new(&**scroll));
    let handler = block2::RcBlock::new(move |_action: NonNull<UIAction>| {
        if let Some(scroll) = weak_scroll.as_ref().and_then(Weak::load) {
            let offset = scroll.contentOffset();
            scroll.setContentOffset_animated(
                CGPoint::new(offset.x, -scroll.adjustedContentInset().top),
                true,
            );
        }
    });
    let action = unsafe { UIAction::actionWithHandler(block2::RcBlock::as_ptr(&handler), mtm) };

    let supports_clear_glass = objc2::class!(UIButtonConfiguration)
        .class_method(objc2::sel!(clearGlassButtonConfiguration))
        .is_some();
    let configuration = if supports_clear_glass {
        UIButtonConfiguration::clearGlassButtonConfiguration(mtm)
    } else {
        UIButtonConfiguration::plainButtonConfiguration(mtm)
    };
    configuration.setTitle(Some(&title));
    if let Some(label) = label {
        configuration.setBaseForegroundColor(label.textColor().as_deref());
    }
    configuration.setCornerStyle(UIButtonConfigurationCornerStyle::Capsule);
    configuration.setContentInsets(NSDirectionalEdgeInsets {
        top: 9.0,
        leading: 32.0,
        bottom: 9.0,
        trailing: 32.0,
    });
    let button = UIButton::buttonWithConfiguration_primaryAction(&configuration, Some(&action));
    button.setTag(DIARY_TITLE_BUTTON_TAG);
    let label_size = label
        .map(|label| label.frame().size)
        .unwrap_or_else(|| CGSize::new(title.to_string().chars().count() as f64 * 9.0, 20.0));
    let height = label_size.height + 18.0;
    button.setFrame(CGRect::new(
        CGPoint::ZERO,
        CGSize::new(label_size.width + 64.0, height),
    ));

    if !supports_clear_glass {
        let effect = native_glass_effect(mtm);
        let background =
            UIVisualEffectView::initWithEffect(UIVisualEffectView::alloc(mtm), Some(&effect));
        background.setFrame(button.bounds());
        background.setUserInteractionEnabled(false);
        background.layer().setCornerRadius((height / 2.0) as _);
        background.layer().setMasksToBounds(true);
        button.insertSubview_atIndex(&background, 0);
    }
    item.setTitleView(Some(&button));
}

fn is_app_page_title(title: &str) -> bool {
    title == crate::res::str::diary_title().format()
        || title == crate::res::str::schedule_title().format()
        || title == crate::res::str::teachers_title().format()
        || title == crate::res::str::settings_title().format()
}

fn is_supported_controller(controller: &UIViewController) -> bool {
    is_supported_item(Some(&controller.navigationItem()))
}

fn is_supported_item(item: Option<&objc2_ui_kit::UINavigationItem>) -> bool {
    let Some(item) = item else { return false };
    item.title()
        .is_some_and(|title| is_app_page_title(&title.to_string()))
        || item.titleView().is_some_and(|view| {
            view.tag() == DIARY_TITLE_BUTTON_TAG
                || view
                    .downcast_ref::<UILabel>()
                    .and_then(|label| label.text())
                    .is_some_and(|title| is_app_page_title(&title.to_string()))
        })
}

fn native_glass_effect(mtm: MainThreadMarker) -> objc2::rc::Retained<objc2_ui_kit::UIVisualEffect> {
    if objc2::runtime::AnyClass::get(c"UIGlassEffect").is_some() {
        let effect = UIGlassEffect::effectWithStyle(UIGlassEffectStyle::Regular, mtm);
        effect.setInteractive(true);
        return effect.into_super();
    }
    UIBlurEffect::effectWithStyle(UIBlurEffectStyle::SystemThinMaterial, mtm).into_super()
}
