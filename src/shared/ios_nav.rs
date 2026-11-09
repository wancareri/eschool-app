#[cfg(target_os = "ios")]
pub mod ios_impl {
    use crate::app::{AppState, ConnStatus};
    use crate::shared::nslog::nslog;
    use day::prelude::Ambient;
    use objc2::{MainThreadMarker, MainThreadOnly};
    use objc2::rc::Retained;
    use objc2_foundation::{NSString, NSArray};
    use objc2_ui_kit::{
        UIViewController, UINavigationController, UITabBarController,
        UIBarButtonItem, UIBarButtonItemGroup, UIButton, UIColor, UIImage,
        UIControlState, UIAction, UIApplication, UIView,
    };
    use block2::RcBlock;
    use core::ptr::NonNull;
    use std::collections::VecDeque;

    /// Find the topmost visible `UINavigationItem` in the current UI hierarchy.
    pub fn find_active_nav_item(view: Option<&UIView>) -> Option<Retained<objc2_ui_kit::UINavigationItem>> {
        let mtm = MainThreadMarker::new()?;

        // 1. If view is given, walk its responder chain upwards
        if let Some(v) = view {
            let mut next = v.nextResponder();
            while let Some(responder) = next {
                if let Some(vc) = responder.downcast_ref::<UIViewController>() {
                    if let Some(nav) = vc.navigationController() {
                        nav.setNavigationBarHidden_animated(false, false);
                        if let Some(top) = nav.topViewController() {
                            nslog("[ios_nav] Found topViewController via VC.navigationController");
                            return Some(top.navigationItem());
                        }
                    }
                    nslog("[ios_nav] Found VC via responder chain");
                    return Some(vc.navigationItem());
                }
                next = responder.nextResponder();
            }
        }

        // 2. Try window from view or UIApplication.windows()
        let window = view.and_then(|v| v.window()).or_else(|| {
            let app = UIApplication::sharedApplication(mtm);
            #[allow(deprecated)]
            let wins = app.windows();
            wins.firstObject()
        })?;

        // Ensure window has system background color (prevents black screen in light mode)
        window.setBackgroundColor(Some(&UIColor::systemBackgroundColor()));
        if let Some(rvc) = window.rootViewController() {
            if let Some(v) = rvc.view() {
                v.setBackgroundColor(Some(&UIColor::systemBackgroundColor()));
            }
        }

        // 3. Search view controller tree with BFS for instant match
        let root = window.rootViewController()?;
        let mut queue = VecDeque::new();
        queue.push_back(root);

        while let Some(c) = queue.pop_front() {
            if let Some(nav) = c.downcast_ref::<UINavigationController>() {
                nav.setNavigationBarHidden_animated(false, false);
                if let Some(top) = nav.topViewController() {
                    nslog("[ios_nav] Found topViewController via UINavigationController");
                    return Some(top.navigationItem());
                }
                if let Some(vis) = nav.visibleViewController() {
                    nslog("[ios_nav] Found visibleViewController via UINavigationController");
                    return Some(vis.navigationItem());
                }
                if let Some(first) = nav.viewControllers().firstObject() {
                    nslog("[ios_nav] Found firstViewController via UINavigationController");
                    return Some(first.navigationItem());
                }
            }
            if let Some(tab) = c.downcast_ref::<UITabBarController>() {
                if let Some(sel) = unsafe { tab.selectedViewController() } {
                    if let Some(nav) = sel.downcast_ref::<UINavigationController>() {
                        nav.setNavigationBarHidden_animated(false, false);
                        if let Some(top) = nav.topViewController() {
                            nslog("[ios_nav] Found topViewController via UITabBarController");
                            return Some(top.navigationItem());
                        }
                    }
                    nslog("[ios_nav] Found selectedViewController via UITabBarController");
                    return Some(sel.navigationItem());
                }
            }
            if let Some(pres) = c.presentedViewController() {
                queue.push_back(pres);
            }
            for ch in c.childViewControllers().iter() {
                queue.push_back(ch);
            }
        }

        nslog("[ios_nav] No UINavigationItem found in hierarchy");
        None
    }

    /// Attach bar button items reliably to rightBarButtonItems.
    pub fn attach_nav_items(
        nav_item: &objc2_ui_kit::UINavigationItem,
        items: &[Retained<UIBarButtonItem>],
    ) {
        if items.is_empty() { return; }

        let arr = NSArray::from_retained_slice(items);
        nav_item.setRightBarButtonItems(Some(&arr));
        nslog(&format!("[ios_nav] Successfully attached {} bar items", items.len()));
    }

    /// Build Theme button with native SF Symbol (sun.max.fill / moon.stars.fill)
    pub fn build_theme_button(mtm: MainThreadMarker) -> (Retained<UIButton>, Retained<UIBarButtonItem>) {
        let b = UIButton::new(mtm);
        b.setFrame(objc2_core_foundation::CGRect {
            origin: objc2_core_foundation::CGPoint { x: 0.0, y: 0.0 },
            size: objc2_core_foundation::CGSize { width: 36.0, height: 36.0 },
        });
        b.layer().setCornerRadius(18.0);
        b.layer().setMasksToBounds(true);
        b.setBackgroundColor(Some(&UIColor::tertiarySystemFillColor()));

        let cur = day::prefs::get(crate::THEME_KEY);
        let icon_name = match cur.as_deref() {
            Some("light") => "sun.max.fill",
            Some("dark") => "moon.stars.fill",
            _ => if day::dark_mode() { "moon.stars.fill" } else { "sun.max.fill" },
        };
        let icon = UIImage::systemImageNamed(&NSString::from_str(icon_name));
        b.setImage_forState(icon.as_deref(), UIControlState::Normal);
        b.setTitle_forState(None, UIControlState::Normal);
        unsafe {
            b.setTintColor(Some(&UIColor::labelColor()));
        }

        let action_block = RcBlock::new(move |_act: NonNull<UIAction>| {
            crate::shared::haptics::tick();
            crate::pages::login::show_theme_sheet();
        });
        let act = unsafe {
            UIAction::actionWithTitle_image_identifier_handler(
                &NSString::from_str("ThemeAction"),
                None,
                None,
                RcBlock::as_ptr(&action_block),
                mtm,
            )
        };
        b.addAction_forControlEvents(&act, objc2_ui_kit::UIControlEvents::TouchUpInside);

        let item = UIBarButtonItem::initWithCustomView(UIBarButtonItem::alloc(mtm), &b);
        item.setWidth(36.0);
        (b, item)
    }

    /// Build Language button with native SF Symbol globe + clean code (" RU", " BE", " EN")
    pub fn build_lang_button(mtm: MainThreadMarker) -> (Retained<UIButton>, Retained<UIBarButtonItem>) {
        let b = UIButton::new(mtm);
        b.setFrame(objc2_core_foundation::CGRect {
            origin: objc2_core_foundation::CGPoint { x: 0.0, y: 0.0 },
            size: objc2_core_foundation::CGSize { width: 68.0, height: 36.0 },
        });
        b.layer().setCornerRadius(18.0);
        b.layer().setMasksToBounds(true);
        b.setBackgroundColor(Some(&UIColor::tertiarySystemFillColor()));

        let cur = day::locale().get_untracked();
        let lower = cur.to_lowercase();
        let code = if lower.starts_with("be") {
            "BE"
        } else if lower.starts_with("en") {
            "EN"
        } else {
            "RU"
        };

        let globe = UIImage::systemImageNamed(&NSString::from_str("globe"));
        b.setImage_forState(globe.as_deref(), UIControlState::Normal);
        b.setTitle_forState(Some(&NSString::from_str(&format!(" {}", code))), UIControlState::Normal);
        b.setTitleColor_forState(Some(&UIColor::labelColor()), UIControlState::Normal);
        unsafe {
            b.setTintColor(Some(&UIColor::labelColor()));
        }
        if let Some(lbl) = b.titleLabel() {
            unsafe {
                lbl.setFont(Some(&objc2_ui_kit::UIFont::boldSystemFontOfSize(13.0)));
            }
        }

        let action_block = RcBlock::new(move |_act: NonNull<UIAction>| {
            crate::shared::haptics::tick();
            crate::pages::login::show_lang_sheet();
        });
        let act = unsafe {
            UIAction::actionWithTitle_image_identifier_handler(
                &NSString::from_str("LangAction"),
                None,
                None,
                RcBlock::as_ptr(&action_block),
                mtm,
            )
        };
        b.addAction_forControlEvents(&act, objc2_ui_kit::UIControlEvents::TouchUpInside);

        let item = UIBarButtonItem::initWithCustomView(UIBarButtonItem::alloc(mtm), &b);
        item.setWidth(68.0);
        (b, item)
    }

    /// Setup Login navigation bar (Theme + Language)
    pub fn setup_login_nav_bar(view: Option<&UIView>) {
        let state = AppState::ambient();
        if state.is_authenticated.get() {
            return;
        }
        let Some(mtm) = MainThreadMarker::new() else { return };
        let Some(nav_item) = find_active_nav_item(view) else { return };

        let (_t_btn, t_item) = build_theme_button(mtm);
        let (_l_btn, l_item) = build_lang_button(mtm);

        // Right to left: theme on right, lang on left
        attach_nav_items(&nav_item, &[t_item, l_item]);
    }

    /// Schedule retries for login nav bar to guarantee instantaneous attachment
    pub fn schedule_login_nav_bar(view: &UIView) {
        let ptr = view as *const UIView as usize;
        struct SendPtr(usize);
        unsafe impl Send for SendPtr {}

        for delay in [0, 20, 60, 150, 400, 800, 1500] {
            let sp = SendPtr(ptr);
            day::reactive::on_main_delayed(delay, move || {
                let v = unsafe { &*(sp.0 as *const UIView) };
                setup_login_nav_bar(Some(v));
            });
        }
    }

    /// Build Network status button
    pub fn build_status_button(mtm: MainThreadMarker, state: AppState) -> (Retained<UIButton>, Retained<UIBarButtonItem>) {
        let b = UIButton::new(mtm);
        b.setFrame(objc2_core_foundation::CGRect {
            origin: objc2_core_foundation::CGPoint { x: 0.0, y: 0.0 },
            size: objc2_core_foundation::CGSize { width: 36.0, height: 36.0 },
        });
        b.layer().setCornerRadius(18.0);
        b.layer().setMasksToBounds(true);
        b.setBackgroundColor(Some(&UIColor::tertiarySystemFillColor()));

        let icon_name = match state.conn_status.get() {
            ConnStatus::Connecting => "arrow.triangle.2.circlepath",
            ConnStatus::Offline => "wifi.slash",
            ConnStatus::Error => "exclamationmark.triangle.fill",
            _ => "checkmark.circle.fill",
        };
        let icon = UIImage::systemImageNamed(&NSString::from_str(icon_name));
        b.setImage_forState(icon.as_deref(), UIControlState::Normal);

        let tint = match state.conn_status.get() {
            ConnStatus::Offline => UIColor::systemGrayColor(),
            ConnStatus::Error => UIColor::systemRedColor(),
            _ => UIColor::labelColor(),
        };
        unsafe {
            b.setTintColor(Some(&tint));
        }

        let action_block = RcBlock::new(move |_act: NonNull<UIAction>| {
            crate::shared::haptics::tick();
            state.show_network_modal.set(Some("net".into()));
        });
        let act = unsafe {
            UIAction::actionWithTitle_image_identifier_handler(
                &NSString::from_str("NetworkStatus"),
                None,
                None,
                RcBlock::as_ptr(&action_block),
                mtm,
            )
        };
        b.addAction_forControlEvents(&act, objc2_ui_kit::UIControlEvents::TouchUpInside);

        let item = UIBarButtonItem::initWithCustomView(UIBarButtonItem::alloc(mtm), &b);
        item.setWidth(36.0);
        (b, item)
    }

    /// Setup Status navigation bar
    pub fn setup_status_nav_bar(view: Option<&UIView>, state: AppState) {
        if !state.is_authenticated.get() {
            return;
        }
        let Some(mtm) = MainThreadMarker::new() else { return };
        let Some(nav_item) = find_active_nav_item(view) else { return };

        let (_b, item) = build_status_button(mtm, state);
        attach_nav_items(&nav_item, &[item]);
    }

    /// Schedule retries for status nav bar
    pub fn schedule_status_nav_bar(view: &UIView) {
        let ptr = view as *const UIView as usize;
        struct SendPtr(usize);
        unsafe impl Send for SendPtr {}

        for delay in [0, 60, 180, 450] {
            let sp = SendPtr(ptr);
            day::reactive::on_main_delayed(delay, move || {
                let v = unsafe { &*(sp.0 as *const UIView) };
                let st = AppState::ambient();
                setup_status_nav_bar(Some(v), st);
            });
        }
    }
}
