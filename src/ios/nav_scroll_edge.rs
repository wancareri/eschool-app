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
    });
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
    let diary_title = crate::res::str::diary_title().format();
    let item = controller.navigationItem();
    let is_diary = item
        .title()
        .is_some_and(|title| title.to_string() == diary_title)
        || item
            .titleView()
            .and_then(|view| view.downcast::<UILabel>().ok())
            .and_then(|label| label.text())
            .is_some_and(|title| title.to_string() == diary_title);
    if !is_diary {
        return;
    }
    install_title_button(controller, &bar);
    if bar.tag() == DIARY_BAR_STYLE_TAG {
        return;
    }
    apply_system_bar_appearance(&bar);
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
    appearance.setBackgroundColor(Some(&objc2_ui_kit::UIColor::clearColor()));
    appearance.setShadowColor(None);
    bar.setBackgroundColor(Some(&objc2_ui_kit::UIColor::clearColor()));
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
    let Some(title_view) = item.titleView() else {
        return;
    };
    if title_view.tag() == DIARY_TITLE_BUTTON_TAG {
        return;
    }
    let Some(label) = title_view.downcast_ref::<UILabel>() else {
        return;
    };
    let Some(title) = label.text() else { return };
    if title.to_string() != crate::res::str::diary_title().format() {
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
    configuration.setBaseForegroundColor(label.textColor().as_deref());
    configuration.setCornerStyle(UIButtonConfigurationCornerStyle::Capsule);
    configuration.setContentInsets(NSDirectionalEdgeInsets {
        top: 9.0,
        leading: 22.0,
        bottom: 9.0,
        trailing: 22.0,
    });
    let button = UIButton::buttonWithConfiguration_primaryAction(&configuration, Some(&action));
    button.setTag(DIARY_TITLE_BUTTON_TAG);
    let label_size = label.frame().size;
    let height = label_size.height + 18.0;
    button.setFrame(CGRect::new(
        CGPoint::ZERO,
        CGSize::new(label_size.width + 44.0, height),
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

fn native_glass_effect(mtm: MainThreadMarker) -> objc2::rc::Retained<objc2_ui_kit::UIVisualEffect> {
    if objc2::runtime::AnyClass::get(c"UIGlassEffect").is_some() {
        let effect = UIGlassEffect::effectWithStyle(UIGlassEffectStyle::Regular, mtm);
        effect.setInteractive(true);
        return effect.into_super();
    }
    UIBlurEffect::effectWithStyle(UIBlurEffectStyle::SystemThinMaterial, mtm).into_super()
}
