//! Balance UIKit containment when Day replaces a top-level screen.
use day::reactive::Scope;
use dispatch2::{DispatchQueue, MainThreadBound};
use objc2::{
    MainThreadMarker,
    rc::{Retained, Weak},
};
use objc2_ui_kit::{
    UINavigationController, UISplitViewController, UITabBarController, UIView, UIViewController,
};
use std::{cell::Cell, rc::Rc};

fn root_controllers(view: &UIView, out: &mut Vec<Retained<UIViewController>>) {
    if let Some(responder) = view.nextResponder()
        && let Some(vc) = responder.downcast_ref::<UIViewController>()
        && (vc.downcast_ref::<UINavigationController>().is_some()
            || vc.downcast_ref::<UITabBarController>().is_some()
            || vc.downcast_ref::<UISplitViewController>().is_some())
        && vc.viewIfLoaded().is_some_and(|v| std::ptr::eq(&*v, view))
    {
        out.push(Retained::from(vc));
        return; // The controller owns its nested page controllers.
    }
    for child in view.subviews() {
        root_controllers(&child, out);
    }
}

pub(super) fn mount_screen(host: &UIView, mtm: MainThreadMarker) {
    let mut controllers = Vec::new();
    root_controllers(host, &mut controllers);
    let alive = Rc::new(Cell::new(true));
    let cleanup_alive = alive.clone();
    let mut cleanup = controllers;
    let cleanup_host = Weak::new(host);
    Scope::current().on_cleanup(move || {
        cleanup_alive.set(false);
        // Login can replace its inner stack for required password setup without
        // replacing this screen. Its current controller also needs teardown.
        if let Some(host) = cleanup_host.load() {
            let mut current = Vec::new();
            root_controllers(&host, &mut current);
            for vc in current {
                if !cleanup.iter().any(|old| std::ptr::eq(&**old, &*vc)) {
                    cleanup.push(vc);
                }
            }
        }
        // Day removes views during its tree edit but does not unparent nav VCs.
        // Defer UIKit callbacks until that edit has released the tree borrow.
        let bound = MainThreadBound::new(cleanup, mtm);
        DispatchQueue::main().exec_async(move || {
            let mtm = MainThreadMarker::new().expect("UIKit main thread");
            for vc in bound.into_inner(mtm) {
                vc.willMoveToParentViewController(None);
                if let Some(view) = vc.viewIfLoaded() {
                    view.removeFromSuperview();
                }
                vc.removeFromParentViewController();
            }
        });
    });

    let bound = MainThreadBound::new((Weak::new(host), alive), mtm);
    DispatchQueue::main().exec_async(move || {
        let mtm = MainThreadMarker::new().expect("UIKit main thread");
        let (host, alive) = bound.into_inner(mtm);
        if !alive.get() {
            return;
        }
        let Some(host) = host.load() else { return };
        let Some(window) = host.window() else { return };
        let Some(root) = window.rootViewController() else {
            return;
        };
        let mut controllers = Vec::new();
        root_controllers(&host, &mut controllers);
        for vc in controllers {
            let Some(view) = vc.viewIfLoaded() else {
                continue;
            };
            let Some(parent) = view.superview() else {
                continue;
            };
            // Force the incoming host through a complete containment transaction
            // after the outgoing screen is gone, including its appearance callbacks.
            vc.willMoveToParentViewController(None);
            view.removeFromSuperview();
            vc.removeFromParentViewController();
            root.addChildViewController(&vc);
            parent.addSubview(&view);
            vc.didMoveToParentViewController(Some(&root));
            view.setNeedsLayout();
        }
        // Keep DayRootVC/DayHolderView: they own Day's window and safe-area rail.
        // UIKit now lays out the new tab hierarchy on this same main-queue turn.
        if let Some(view) = root.viewIfLoaded() {
            view.setNeedsLayout();
            view.layoutIfNeeded();
        }
        host.setNeedsLayout();
        host.layoutIfNeeded();
    });
}
