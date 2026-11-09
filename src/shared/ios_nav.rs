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

    /// Find the topmost visible `UINavigationItem` in the current UI hierarchy.
    pub fn find_active_nav_item(view: Option<&UIView>) -> Option<Retained<objc2_ui_kit::UINavigationItem>> {
        let mtm = MainThreadMarker::new()?;

        // 1. If view is given, walk its responder chain upwards
        if let Some(v) = view {
            let mut next = v.nextResponder();
            while let Some(responder) = next {
                if let Some(vc) = responder.downcast_ref::<UIViewController>() {
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

        // 3. Search view controller tree
        let root = window.rootViewController()?;
        let mut stack = vec![root];
        while let Some(c) = stack.pop() {
            if let Some(nav) = c.downcast_ref::<UINavigationController>() {
                unsafe { nav.setNavigationBarHidden_animated(false, false); }
                if let Some(top) = nav.topViewController() {
                    nslog("[ios_nav] Found topViewController via UINavigationController");
                    return Some(top.navigationItem());
                }
                if let Some(vis) = nav.visibleViewController() {
                    nslog("[ios_nav] Found visibleViewController via UINavigationController");
                    return Some(vis.navigationItem());
                }
            }
            if let Some(tab) = c.downcast_ref::<UITabBarController>() {
                if let Some(sel) = unsafe { tab.selectedViewController() } {
                    if let Some(nav) = sel.downcast_ref::<UINavigationController>() {
                        unsafe { nav.setNavigationBarHidden_animated(false, false); }
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
                stack.push(pres);
            }
            for ch in c.childViewControllers().iter() {
                stack.push(ch);
            }
        }

        nslog("[ios_nav] No UINavigationItem found in hierarchy");
        None
    }

    /// Attach bar button items to both rightBarButtonItems and trailingItemGroups (iOS 16+)
    pub fn attach_nav_items(
        nav_item: &objc2_ui_kit::UINavigationItem,
        items: &[Retained<UIBarButtonItem>],
    ) {
        let Some(mtm) = MainThreadMarker::new() else { return };
        if items.is_empty() { return };

        let arr = NSArray::from_retained_slice(items);
        nav_item.setRightBarButtonItems(Some(&arr));

        let group = UIBarButtonItemGroup::fixedGroupWithRepresentativeItem_items(None, &arr, mtm);
        let group_arr = NSArray::from_retained_slice(&[group]);
        nav_item.setTrailingItemGroups(&group_arr);
        nslog(&format!("[ios_nav] Successfully attached {} bar items", items.len()));
    }

    /// Build Theme button
    pub fn build_theme_button(mtm: MainThreadMarker) -> (Retained<UIButton>, Retained<UIBarButtonItem>) {
        let b = UIButton::new(mtm);
        b.setTranslatesAutoresizingMaskIntoConstraints(false);
        b.widthAnchor().constraintEqualToConstant(36.0).setActive(true);
        b.heightAnchor().constraintEqualToConstant(36.0).setActive(true);
        b.layer().setCornerRadius(18.0);
        b.layer().setMasksToBounds(true);
        b.setBackgroundColor(Some(&UIColor::tertiarySystemFillColor()));

        let cur = day::prefs::get(crate::THEME_KEY);
        let icon = match cur.as_deref() {
            Some("light") => "☀️",
            Some("dark") => "🌙",
            _ => if day::dark_mode() { "🌙" } else { "☀️" },
        };
        b.setTitle_forState(Some(&NSString::from_str(icon)), UIControlState::Normal);

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
        (b, item)
    }

    /// Build Language button
    pub fn build_lang_button(mtm: MainThreadMarker) -> (Retained<UIButton>, Retained<UIBarButtonItem>) {
        let b = UIButton::new(mtm);
        b.setTranslatesAutoresizingMaskIntoConstraints(false);
        b.widthAnchor().constraintEqualToConstant(64.0).setActive(true);
        b.heightAnchor().constraintEqualToConstant(36.0).setActive(true);
        b.layer().setCornerRadius(18.0);
        b.layer().setMasksToBounds(true);
        b.setBackgroundColor(Some(&UIColor::tertiarySystemFillColor()));

        let cur = day::locale().get_untracked();
        let lower = cur.to_lowercase();
        let title = if lower.starts_with("be") {
            "🌐 BE"
        } else if lower.starts_with("en") {
            "🌐 EN"
        } else {
            "🌐 RU"
        };
        b.setTitle_forState(Some(&NSString::from_str(title)), UIControlState::Normal);
        b.setTitleColor_forState(Some(&UIColor::labelColor()), UIControlState::Normal);

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
        (b, item)
    }

    /// Setup Login navigation bar (Theme + Language)
    pub fn setup_login_nav_bar(view: Option<&UIView>) {
        let Some(mtm) = MainThreadMarker::new() else { return };
        let Some(nav_item) = find_active_nav_item(view) else { return };

        let (t_btn, t_item) = build_theme_button(mtm);
        let (l_btn, l_item) = build_lang_button(mtm);

        // Right to left: theme on right, lang on left
        attach_nav_items(&nav_item, &[t_item, l_item]);

        // Watch theme & locale changes
        let t_ptr = Retained::as_ptr(&t_btn) as usize;
        let l_ptr = Retained::as_ptr(&l_btn) as usize;
        struct SendBtns(usize, usize);
        unsafe impl Send for SendBtns {}
        let sb = SendBtns(t_ptr, l_ptr);

        day::reactive::watch(
            move || (day::locale().get(), day::dark_mode()),
            move |(loc, dark), _| {
                let Some(_mtm) = MainThreadMarker::new() else { return };
                let t = unsafe { &*(sb.0 as *const UIButton) };
                let l = unsafe { &*(sb.1 as *const UIButton) };

                let cur = day::prefs::get(crate::THEME_KEY);
                let icon = match cur.as_deref() {
                    Some("light") => "☀️",
                    Some("dark") => "🌙",
                    _ => if *dark { "🌙" } else { "☀️" },
                };
                t.setTitle_forState(Some(&NSString::from_str(icon)), UIControlState::Normal);

                let lower = loc.to_lowercase();
                let title = if lower.starts_with("be") {
                    "🌐 BE"
                } else if lower.starts_with("en") {
                    "🌐 EN"
                } else {
                    "🌐 RU"
                };
                l.setTitle_forState(Some(&NSString::from_str(title)), UIControlState::Normal);
            },
        );
    }

    /// Schedule retries for login nav bar to guarantee attachment regardless of mount timing
    pub fn schedule_login_nav_bar(view: &UIView) {
        let ptr = view as *const UIView as usize;
        struct SendPtr(usize);
        unsafe impl Send for SendPtr {}

        for delay in [0, 80, 200, 500, 1000] {
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
        b.setTranslatesAutoresizingMaskIntoConstraints(false);
        b.widthAnchor().constraintEqualToConstant(36.0).setActive(true);
        b.heightAnchor().constraintEqualToConstant(36.0).setActive(true);
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
        (b, item)
    }

    /// Setup Status navigation bar
    pub fn setup_status_nav_bar(view: Option<&UIView>, state: AppState) {
        let Some(mtm) = MainThreadMarker::new() else { return };
        let Some(nav_item) = find_active_nav_item(view) else { return };

        let (b, item) = build_status_button(mtm, state);
        attach_nav_items(&nav_item, &[item]);

        let b_ptr = Retained::as_ptr(&b) as usize;
        struct SendBtn(usize);
        unsafe impl Send for SendBtn {}
        let sb = SendBtn(b_ptr);

        day::reactive::watch(
            move || state.conn_status.get(),
            move |&status, _| {
                let Some(_mtm) = MainThreadMarker::new() else { return };
                let btn = unsafe { &*(sb.0 as *const UIButton) };
                let icon_name = match status {
                    ConnStatus::Connecting => "arrow.triangle.2.circlepath",
                    ConnStatus::Offline => "wifi.slash",
                    ConnStatus::Error => "exclamationmark.triangle.fill",
                    _ => "checkmark.circle.fill",
                };
                let icon = UIImage::systemImageNamed(&NSString::from_str(icon_name));
                btn.setImage_forState(icon.as_deref(), UIControlState::Normal);
                let tint = match status {
                    ConnStatus::Offline => UIColor::systemGrayColor(),
                    ConnStatus::Error => UIColor::systemRedColor(),
                    _ => UIColor::labelColor(),
                };
                unsafe {
                    btn.setTintColor(Some(&tint));
                }
            },
        );
    }

    /// Schedule retries for status nav bar
    pub fn schedule_status_nav_bar(view: &UIView, state: AppState) {
        let ptr = view as *const UIView as usize;
        struct SendPtr(usize);
        unsafe impl Send for SendPtr {}

        for delay in [0, 80, 200, 500, 1000] {
            let sp = SendPtr(ptr);
            day::reactive::on_main_delayed(delay, move || {
                let v = unsafe { &*(sp.0 as *const UIView) };
                let st = AppState::ambient();
                setup_status_nav_bar(Some(v), st);
            });
        }
    }
}
