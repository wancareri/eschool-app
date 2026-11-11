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
        let tabbar: Option<Retained<UITabBarController>> = root
            .childViewControllers()
            .iter()
            .find_map(|vc| vc.downcast_ref::<UITabBarController>().map(Retained::from));
        let tab_vcs = tabbar
            .as_ref()
            .and_then(|t| t.viewControllers())
            .unwrap_or_default();
        let tab_view = tabbar.as_ref().and_then(|t| t.viewIfLoaded());

        let children = root.childViewControllers();
        for vc in children.iter() {
            if let Some(nav) = vc.downcast_ref::<UINavigationController>() {
                let is_tab_nav = tab_vcs.iter().any(|tvc| {
                    std::ptr::eq(&*tvc, &**nav)
                        || tvc.childViewControllers().iter().any(|cvc| std::ptr::eq(&*cvc, &**nav))
                });
                let is_descendant = tab_view.as_ref().is_some_and(|tv| {
                    nav.viewIfLoaded().is_some_and(|nv| nv.isDescendantOfView(tv))
                });
                let is_login_nav = nav.viewIfLoaded().is_some_and(|v| v.tag() == crate::pages::login::AUTH_NAV_TAG);
                if !is_tab_nav && !is_descendant && (is_login_nav || tabbar.is_some()) {
                    vc.willMoveToParentViewController(None);
                    if let Some(view) = vc.viewIfLoaded() {
                        view.removeFromSuperview();
                    }
                    vc.removeFromParentViewController();
                }
            }
        }
        if let Some(view) = tab_view {
            if let Some(parent) = view.superview() {
                parent.bringSubviewToFront(&view);
            }
            view.setNeedsLayout();
            view.layoutIfNeeded();
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
                    if let Some(parent) = view.superview() {
                        parent.bringSubviewToFront(&view);
                    }
                    view.setNeedsLayout();
                    view.layoutIfNeeded();
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
