//! Connect the Diary's nested UIScrollView to UIKit's native navigation edge effect.

use dispatch2::{DispatchQueue, MainThreadBound};
use objc2::{MainThreadMarker, rc::Weak};
use objc2_ui_kit::{
    NSDirectionalRectEdge, UINavigationBar, UINavigationController, UIScrollView, UIView,
    UIViewController,
};

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
                // Day 0.7.1 sets every page bar to opaque and non-translucent. Restore
                // UIKit's native edge behavior for this scroll-backed Diary page.
                bar.setTranslucent(true);
                bar.setScrollEdgeAppearance(None);
                bar.setCompactScrollEdgeAppearance(None);
                return;
            }
        }
        responder = current.nextResponder();
    }
}
