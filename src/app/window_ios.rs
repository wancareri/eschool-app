//! Balance UIKit containment when Day replaces a top-level screen.
use dispatch2::DispatchQueue;
use objc2::{
    MainThreadMarker,
    rc::Retained,
};
use objc2_ui_kit::{
    UIApplication, UINavigationController, UITabBarController, UIView,
};

pub(super) fn sync_root_controllers(mtm: MainThreadMarker, authenticated: bool) {
    let app = UIApplication::sharedApplication(mtm);
    #[allow(deprecated)]
    let Some(window) = app.keyWindow().or_else(|| {
        app.windows().iter().next().map(|w| Retained::from(&*w))
    }) else {
        return;
    };
    let Some(root) = window.rootViewController() else {
        return;
    };

    if authenticated {
        let children = root.childViewControllers();
        for vc in children.iter() {
            if vc.downcast_ref::<UINavigationController>().is_some() {
                vc.willMoveToParentViewController(None);
                if let Some(view) = vc.viewIfLoaded() {
                    view.removeFromSuperview();
                }
                vc.removeFromParentViewController();
            } else if let Some(tab) = vc.downcast_ref::<UITabBarController>() {
                if let Some(view) = tab.viewIfLoaded() {
                    if let Some(root_view) = root.viewIfLoaded() {
                        root_view.bringSubviewToFront(&view);
                    }
                    view.setNeedsLayout();
                }
            }
        }
    } else {
        let children = root.childViewControllers();
        for vc in children.iter() {
            if vc.downcast_ref::<UITabBarController>().is_some() {
                vc.willMoveToParentViewController(None);
                if let Some(view) = vc.viewIfLoaded() {
                    view.removeFromSuperview();
                }
                vc.removeFromParentViewController();
            } else if let Some(nav) = vc.downcast_ref::<UINavigationController>() {
                if let Some(view) = nav.viewIfLoaded() {
                    if let Some(root_view) = root.viewIfLoaded() {
                        root_view.bringSubviewToFront(&view);
                    }
                    view.setNeedsLayout();
                }
            }
        }
    }

    if let Some(view) = root.viewIfLoaded() {
        view.setNeedsLayout();
        view.layoutIfNeeded();
    }
}

pub(super) fn mount_screen(_host: &UIView, mtm: MainThreadMarker, authenticated: bool) {
    sync_root_controllers(mtm, authenticated);
    DispatchQueue::main().exec_async(move || {
        let mtm = MainThreadMarker::new().expect("UIKit main thread");
        sync_root_controllers(mtm, authenticated);
    });
}
