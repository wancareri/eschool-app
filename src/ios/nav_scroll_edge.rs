//! Connect the Diary's nested UIScrollView to UIKit's native navigation edge effect.

use dispatch2::{DispatchQueue, MainThreadBound};
use objc2::{MainThreadMarker, rc::Weak, runtime::Sel};
use objc2_ui_kit::{
    NSDirectionalRectEdge, UINavigationBar, UINavigationBarAppearance, UINavigationController,
    UIScrollView, UIView, UIViewController,
};
use std::sync::atomic::{AtomicPtr, Ordering};

const DIARY_BAR_STYLE_TAG: isize = 8_417_265;
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
            method.set_implementation(std::mem::transmute(
                diary_view_did_layout as *const (),
            ));
        }
    });
}

unsafe extern "C-unwind" fn diary_view_did_layout(controller: &UIViewController, selector: Sel) {
    let original: unsafe extern "C-unwind" fn(&UIViewController, Sel) = unsafe {
        std::mem::transmute(ORIGINAL_VIEW_DID_LAYOUT.load(Ordering::Acquire))
    };
    unsafe { original(controller, selector) };

    let diary_title = crate::res::str::diary_title().format();
    if !controller
        .navigationItem()
        .title()
        .is_some_and(|title| title.to_string() == diary_title)
    {
        return;
    }
    let nav_controller: Option<objc2::rc::Retained<UINavigationController>> = unsafe {
        objc2::msg_send![controller, navigationController]
    };
    let Some(nav_controller) = nav_controller else {
        return;
    };
    let bar = nav_controller.navigationBar();
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

            let nav_controller: Option<objc2::rc::Retained<UINavigationController>> = unsafe {
                objc2::msg_send![controller, navigationController]
            };
            if let Some(nav_controller) = nav_controller {
                let bar: objc2::rc::Retained<UINavigationBar> = nav_controller.navigationBar();
                apply_system_bar_appearance(&bar);
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
    appearance.configureWithDefaultBackground();
    bar.setStandardAppearance(&appearance);
    bar.setCompactAppearance(Some(&appearance));
    bar.setScrollEdgeAppearance(None);
    bar.setCompactScrollEdgeAppearance(None);
    bar.setTranslucent(true);
    bar.setTag(DIARY_BAR_STYLE_TAG);
}
