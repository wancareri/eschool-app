use day::prelude::*;

thread_local! {
    static KEYBOARD_SIGNAL: std::cell::Cell<Option<Signal<f64>>> = const { std::cell::Cell::new(None) };
}

pub fn keyboard_height_signal() -> Signal<f64> {
    KEYBOARD_SIGNAL.with(|c| {
        if let Some(s) = c.get() {
            s
        } else {
            let s = Scope::root().enter(|| Signal::new(0.0));
            c.set(Some(s));
            s
        }
    })
}

pub fn get_keyboard_height() -> f64 {
    keyboard_height_signal().get()
}

#[allow(deprecated)]
pub fn get_safe_area_top() -> f64 {
    #[cfg(target_os = "ios")]
    {
        use objc2::MainThreadMarker;
        use objc2_ui_kit::UIApplication;
        if let Some(mtm) = MainThreadMarker::new() {
            let app = UIApplication::sharedApplication(mtm);
            if let Some(window) = app.keyWindow() {
                let insets = window.safeAreaInsets();
                return insets.top as f64;
            }
        }
    }
    0.0
}

#[allow(deprecated)]
pub fn get_safe_area_bottom() -> f64 {
    #[cfg(target_os = "ios")]
    {
        use objc2::MainThreadMarker;
        use objc2_ui_kit::UIApplication;
        if let Some(mtm) = MainThreadMarker::new() {
            let app = UIApplication::sharedApplication(mtm);
            if let Some(window) = app.keyWindow() {
                let insets = window.safeAreaInsets();
                return insets.bottom as f64;
            }
        }
    }
    0.0
}

pub fn set_keyboard_height(h: f64) {
    set_keyboard_height_with_duration(h, 250);
}

pub fn set_keyboard_height_with_duration(h: f64, duration_ms: u32) {
    let sig = keyboard_height_signal();
    if (sig.get() - h).abs() > 0.5 {
        crate::shared::nslog::nslog(&format!(
            "[Keyboard] Height updated to: {h:.1} in {duration_ms}ms"
        ));
        sig.set(h);
    }
}

pub fn dismiss() {
    #[cfg(target_os = "ios")]
    {
        use objc2::MainThreadMarker;
        use objc2_ui_kit::UIApplication;
        if let Some(mtm) = MainThreadMarker::new() {
            let app = UIApplication::sharedApplication(mtm);
            unsafe {
                let _ = app.sendAction_to_from_forEvent(
                    objc2::sel!(resignFirstResponder),
                    None,
                    None,
                    None,
                );
            }
        }
    }
}

pub fn init() {
    #[cfg(target_os = "ios")]
    {
        ios::install();
        unsafe extern "C" {
            fn eschool_install_tap_to_dismiss();
        }
        unsafe {
            eschool_install_tap_to_dismiss();
        }
    }
}

#[cfg(target_os = "ios")]
pub fn register_follower_view(view: &objc2::rc::Retained<objc2_ui_kit::UIView>) {
    ios::register_follower_view(view);
}

#[cfg(not(target_os = "ios"))]
pub fn register_follower_view(_view: &()) {}

#[cfg(target_os = "ios")]
pub fn register_sticker_view(view: &objc2::rc::Retained<objc2_ui_kit::UIView>) {
    ios::register_sticker_view(view);
}

#[cfg(not(target_os = "ios"))]
pub fn register_sticker_view(_view: &()) {}

#[cfg(target_os = "ios")]
pub fn register_content_view(view: &objc2::rc::Retained<objc2_ui_kit::UIView>) {
    ios::register_content_view(view);
}

#[cfg(not(target_os = "ios"))]
pub fn register_content_view(_view: &()) {}

#[cfg(target_os = "ios")]
mod ios {
    use objc2::rc::{Retained, Weak};
    use objc2::runtime::{
        AnyClass, AnyObject, Imp, NSObject, NSObjectProtocol, ProtocolObject, Sel,
    };
    use objc2::{MainThreadMarker, MainThreadOnly, define_class};
    use objc2_core_foundation::{CGAffineTransform, CGFloat};
    use objc2_foundation::NSNotification;
    use objc2_ui_kit::{
        NSValueUIGeometryExtensions, UIKeyboardFrameEndUserInfoKey, UIView, UIViewAnimationOptions,
        UIViewController, UIViewControllerTransitionCoordinator,
        UIViewControllerTransitionCoordinatorContext,
    };
    use std::cell::{Cell, RefCell};
    use std::sync::atomic::{AtomicBool, Ordering};

    static INSTALLED: AtomicBool = AtomicBool::new(false);

    #[derive(Clone, Copy)]
    enum FollowerKind {
        Content,
        Sticker,
    }

    #[derive(Clone, Copy)]
    struct KeyboardFrame {
        visible_height: f64,
        duration: f64,
        curve: usize,
    }

    thread_local! {
        static OBSERVER: Cell<Option<Retained<KeyboardObserver>>> = const { Cell::new(None) };
        // Both pages stay mounted during an interactive pop. Keep each page's
        // followers synchronized, without retaining popped pages indefinitely.
        static FOLLOWERS: RefCell<Vec<(Weak<UIView>, FollowerKind)>> = const { RefCell::new(Vec::new()) };
        static CURRENT_LIFT: Cell<CGFloat> = const { Cell::new(0.0) };
        static PENDING_FRAME: Cell<Option<KeyboardFrame>> = const { Cell::new(None) };
        static WAITING_FOR_TRANSITION: Cell<bool> = const { Cell::new(false) };
    }

    fn transform(kind: FollowerKind, lift: CGFloat) -> CGAffineTransform {
        match kind {
            FollowerKind::Sticker if lift > 0.0 => CGAffineTransform {
                a: 0.48,
                b: 0.0,
                c: 0.0,
                d: 0.48,
                tx: 0.0,
                ty: -(lift * 0.50),
            },
            _ => CGAffineTransform {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                tx: 0.0,
                ty: -lift,
            },
        }
    }

    fn register_view(view: &Retained<UIView>, kind: FollowerKind) {
        let lift = CURRENT_LIFT.with(Cell::get);
        unsafe {
            let _: () = objc2::msg_send![&**view, setTransform: transform(kind, lift)];
        }
        FOLLOWERS.with(|all| {
            let mut all = all.borrow_mut();
            all.retain(|(weak, _)| weak.load().is_some_and(|v| !std::ptr::eq(&*v, &**view)));
            all.push((Weak::new(&**view), kind));
        });
    }

    pub fn register_follower_view(view: &Retained<UIView>) {
        register_view(view, FollowerKind::Content);
    }
    pub fn register_sticker_view(view: &Retained<UIView>) {
        register_view(view, FollowerKind::Sticker);
    }
    pub fn register_content_view(view: &Retained<UIView>) {
        register_view(view, FollowerKind::Content);
    }

    fn visible_followers() -> Vec<(Retained<UIView>, FollowerKind)> {
        FOLLOWERS.with(|all| {
            let mut all = all.borrow_mut();
            all.retain(|(weak, _)| weak.load().is_some());
            all.iter()
                .filter_map(|(weak, kind)| {
                    let view = weak.load()?;
                    view.window().is_some().then_some((view, *kind))
                })
                .collect()
        })
    }

    fn interactive_coordinator()
    -> Option<Retained<ProtocolObject<dyn UIViewControllerTransitionCoordinator>>> {
        for (view, _) in visible_followers() {
            let mut responder = view.nextResponder();
            while let Some(current) = responder {
                if let Some(vc) = current.downcast_ref::<UIViewController>() {
                    let coordinator = vc.transitionCoordinator().or_else(|| {
                        vc.navigationController()
                            .and_then(|nav| nav.transitionCoordinator())
                    });
                    if let Some(coordinator) = coordinator.filter(|c| c.initiallyInteractive()) {
                        return Some(coordinator);
                    }
                }
                responder = current.nextResponder();
            }
        }
        None
    }

    /// UIKit can announce the destination keyboard frame at the START of an
    /// interactive pop while the keyboard itself is still stationary. Leave the
    /// forms at their current lift until UIKit commits or cancels the transition.
    fn defer_interactive_frame(frame: KeyboardFrame) -> bool {
        let Some(coordinator) = interactive_coordinator() else {
            return false;
        };
        PENDING_FRAME.with(|pending| pending.set(Some(frame)));
        if WAITING_FOR_TRANSITION.with(|waiting| waiting.replace(true)) {
            return true;
        }
        let completion = block2::RcBlock::new(
            move |context: std::ptr::NonNull<
                ProtocolObject<dyn UIViewControllerTransitionCoordinatorContext>,
            >| {
                let cancelled = unsafe { context.as_ref().isCancelled() };
                dispatch2::DispatchQueue::main().exec_async(move || {
                    WAITING_FOR_TRANSITION.with(|waiting| waiting.set(false));
                    let pending = PENDING_FRAME.with(Cell::take);
                    if !cancelled {
                        if let Some(mut frame) = pending {
                            // Read UIKit's settled geometry: another field on the
                            // revealed page may have kept the keyboard on screen.
                            frame.visible_height =
                                settled_keyboard_height().unwrap_or(frame.visible_height);
                            frame.duration = 0.0;
                            apply_keyboard_frame(frame);
                        }
                    }
                });
            },
        );
        if coordinator.animateAlongsideTransition_completion(None, Some(&completion)) {
            true
        } else {
            WAITING_FOR_TRANSITION.with(|waiting| waiting.set(false));
            PENDING_FRAME.with(Cell::take);
            false
        }
    }

    fn settled_keyboard_height() -> Option<f64> {
        let window = visible_followers()
            .into_iter()
            .find_map(|(view, _)| view.window())?;
        let frame = window.keyboardLayoutGuide().layoutFrame();
        if frame.size.height == 0.0 && frame.origin.y == 0.0 {
            return None;
        }
        if frame.size.height <= window.safeAreaInsets().bottom + 0.5 {
            return Some(0.0);
        }
        Some((window.bounds().size.height - frame.origin.y).clamp(0.0, window.bounds().size.height))
    }

    fn apply_keyboard_frame(frame: KeyboardFrame) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let lift =
            (frame.visible_height - super::get_safe_area_bottom()).clamp(0.0, 340.0) as CGFloat;
        CURRENT_LIFT.with(|current| current.set(lift));
        let views = visible_followers();
        // Create the layout guide on the first notification, so it is already
        // tracking UIKit's keyboard geometry when a later navigation completes.
        if let Some(window) = views.iter().find_map(|(view, _)| view.window()) {
            let _ = window.keyboardLayoutGuide();
        }
        if frame.duration > 0.0 {
            // MainThreadBound owns the native objects for the animation block.
            let views = dispatch2::MainThreadBound::new(views, mtm);
            let animations = block2::RcBlock::new(move || {
                let mtm = MainThreadMarker::new().expect("UIKit animations run on main");
                for (view, kind) in views.get(mtm) {
                    unsafe {
                        let _: () = objc2::msg_send![&**view, setTransform: transform(*kind, lift)];
                    }
                }
            });
            UIView::animateWithDuration_delay_options_animations_completion(
                frame.duration,
                0.0,
                UIViewAnimationOptions((frame.curve << 16) | (1 << 1) | (1 << 2)),
                &animations,
                None,
                mtm,
            );
        } else {
            for (view, kind) in views {
                unsafe {
                    let _: () = objc2::msg_send![&*view, setTransform: transform(kind, lift)];
                }
            }
        }
        super::set_keyboard_height_with_duration(
            frame.visible_height,
            (frame.duration * 1000.0).round() as u32,
        );
    }

    // Disable Day's root-frame resizing. Only our dedicated observer processes
    // the notification; handling it here too would start every animation twice.
    unsafe extern "C-unwind" fn suppress_day_keyboard_resize(
        _this: *mut AnyObject,
        _cmd: Sel,
        _notification: *mut NSNotification,
    ) {
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "DayKeyboardHeightObserver"]
        struct KeyboardObserver;

        unsafe impl NSObjectProtocol for KeyboardObserver {}

        impl KeyboardObserver {
            #[unsafe(method(keyboardFrameChanged:))]
            fn keyboard_frame_changed(&self, notif: *mut NSNotification) {
                unsafe {
                    swizzled_keyboard_will_change(std::ptr::null_mut(), objc2::sel!(keyboardFrameChanged:), notif);
                }
            }
        }
    );

    unsafe extern "C-unwind" fn swizzled_keyboard_will_change(
        _this: *mut AnyObject,
        _cmd: Sel,
        notification: *mut NSNotification,
    ) {
        if notification.is_null() {
            return;
        }
        let notif: &NSNotification = unsafe { &*notification };
        let Some(info) = notif.userInfo() else {
            return;
        };
        let Some(val) = info
            .objectForKey(unsafe { UIKeyboardFrameEndUserInfoKey })
            .and_then(|o| o.downcast::<objc2_foundation::NSValue>().ok())
        else {
            return;
        };
        let kb = unsafe { val.CGRectValue() };
        let screen_h = crate::pages::diary::get_screen_height();
        let visible_h = if kb.origin.y < screen_h {
            (screen_h - kb.origin.y).max(0.0)
        } else {
            0.0
        };

        let dur_key =
            objc2_foundation::NSString::from_str("UIKeyboardAnimationDurationUserInfoKey");
        let duration_sec = info
            .objectForKey(&dur_key)
            .and_then(|o| o.downcast::<objc2_foundation::NSNumber>().ok())
            .map(|n| n.as_f64())
            .unwrap_or(0.25);

        let curve_key = objc2_foundation::NSString::from_str("UIKeyboardAnimationCurveUserInfoKey");
        let curve = info
            .objectForKey(&curve_key)
            .and_then(|o| o.downcast::<objc2_foundation::NSNumber>().ok())
            .map(|n| n.as_isize())
            .unwrap_or(7);

        let frame = KeyboardFrame {
            visible_height: visible_h,
            duration: duration_sec,
            curve: curve as usize,
        };
        if !defer_interactive_frame(frame) {
            apply_keyboard_frame(frame);
        }
    }

    pub fn install() {
        if INSTALLED.swap(true, Ordering::SeqCst) {
            return;
        }

        // Swizzle DayAppDelegate's keyboardWillChange: method.
        // In day-uikit, DayAppDelegate::keyboard_will_change shrinks root_view.frame
        // to clamp to the keyboard top, which forces the UITabBarController's tab bar
        // to float up above the keyboard.
        // Suppress its resize handler; the dedicated observer below owns keyboard
        // geometry and animations. Processing both callbacks would animate twice.
        let cls = AnyClass::get(c"DayAppDelegate");
        if let Some(cls) = cls {
            let sel = objc2::sel!(keyboardWillChange:);
            if let Some(method) = cls.instance_method(sel) {
                let imp: Imp =
                    unsafe { std::mem::transmute(suppress_day_keyboard_resize as *const ()) };
                unsafe {
                    method.set_implementation(imp);
                }
                crate::shared::nslog::nslog(
                    "[Keyboard] Successfully swizzled DayAppDelegate keyboardWillChange:",
                );
            } else {
                crate::shared::nslog::nslog(
                    "[Keyboard] WARNING: keyboardWillChange: method not found on DayAppDelegate",
                );
            }
        } else {
            crate::shared::nslog::nslog("[Keyboard] WARNING: DayAppDelegate class not found");
        }

        // Register single dedicated observer for UIKeyboardWillChangeFrameNotification.
        // This is the canonical master notification that covers show, hide, orientation,
        // and accessory/QuickType bar transitions with full animation duration and curve.
        if let Some(mtm) = MainThreadMarker::new() {
            let obs: Retained<KeyboardObserver> =
                unsafe { objc2::msg_send![KeyboardObserver::alloc(mtm), init] };
            let center = objc2_foundation::NSNotificationCenter::defaultCenter();
            unsafe {
                center.addObserver_selector_name_object(
                    &obs,
                    objc2::sel!(keyboardFrameChanged:),
                    Some(objc2_ui_kit::UIKeyboardWillChangeFrameNotification),
                    None,
                );
            }
            OBSERVER.with(|c| c.set(Some(obs)));
            crate::shared::nslog::nslog(
                "[Keyboard] Registered KeyboardObserver for UIKeyboardWillChangeFrameNotification",
            );
        }

        // Swizzle UITextField setBorderStyle: to force UITextBorderStyleNone (0) so text fields have no native border/outline
        let tf_cls = AnyClass::get(c"UITextField");
        if let Some(cls) = tf_cls {
            let sel = objc2::sel!(setBorderStyle:);
            if let Some(method) = cls.instance_method(sel) {
                let orig: Imp = method.implementation();
                ORIG_SET_BORDER_STYLE.store(orig as *mut (), std::sync::atomic::Ordering::Release);
                let imp: Imp =
                    unsafe { std::mem::transmute(swizzled_set_border_style as *const ()) };
                unsafe {
                    method.set_implementation(imp);
                }
                crate::shared::nslog::nslog(
                    "[TextField] Successfully swizzled UITextField setBorderStyle: to force None",
                );
            }
        }
    }

    static ORIG_SET_BORDER_STYLE: std::sync::atomic::AtomicPtr<()> =
        std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());
    unsafe extern "C-unwind" fn swizzled_set_border_style(
        this: *mut AnyObject,
        cmd: Sel,
        _style: isize,
    ) {
        // Force borderStyle to UITextBorderStyleNone (0) so text fields have no native outline/border
        let orig_ptr = ORIG_SET_BORDER_STYLE.load(std::sync::atomic::Ordering::Acquire);
        if !orig_ptr.is_null() {
            let orig: unsafe extern "C-unwind" fn(*mut AnyObject, Sel, isize) =
                unsafe { std::mem::transmute(orig_ptr) };
            unsafe {
                orig(this, cmd, 0);
            }
        }
    }
}
