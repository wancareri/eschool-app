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
pub fn sync_follower_for_responder(field: &objc2_ui_kit::UIView) {
    ios::sync_follower_for_responder(field);
}

/// The page is laid out inside the bottom safe area; subtract it once so a
/// dock with an 8pt inset ends 8pt above the keyboard, including tall layouts.
#[cfg(any(target_os = "ios", test))]
fn keyboard_lift(visible_height: f64, safe_bottom: f64) -> f64 {
    (visible_height - safe_bottom).max(0.0)
}

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
    struct KeyboardFrame {
        visible_height: f64,
        duration: f64,
        curve: usize,
    }

    thread_local! {
        static OBSERVER: Cell<Option<Retained<KeyboardObserver>>> = const { Cell::new(None) };
        // Both pages stay mounted during an interactive pop. Keep each page's
        // followers synchronized, without retaining popped pages indefinitely.
        static FOLLOWERS: RefCell<Vec<Weak<UIView>>> = const { RefCell::new(Vec::new()) };
        static CURRENT_LIFT: Cell<CGFloat> = const { Cell::new(0.0) };
        static PENDING_FRAME: Cell<Option<KeyboardFrame>> = const { Cell::new(None) };
        static WAITING_FOR_TRANSITION: Cell<bool> = const { Cell::new(false) };
    }

    fn transform(lift: CGFloat) -> CGAffineTransform {
        CGAffineTransform { a: 1.0, b: 0.0, c: 0.0, d: 1.0, tx: 0.0, ty: -lift }
    }

    pub fn register_follower_view(view: &Retained<UIView>) {
        // The cached height belongs to the outgoing page until the new page
        // actually owns the responder. A mounted destination is not active yet.
        let lift = page_for_view(view).filter(|page| contains_first_responder(page))
            .map_or(0.0, |_| CURRENT_LIFT.with(Cell::get));
        unsafe {
            let _: () = objc2::msg_send![&**view, setTransform: transform(lift)];
        }
        FOLLOWERS.with(|all| {
            let mut all = all.borrow_mut();
            all.retain(|weak| weak.load().is_some_and(|v| !std::ptr::eq(&*v, &**view)));
            all.push(Weak::new(&**view));
        });
    }

    fn page_for_view(view: &UIView) -> Option<Retained<UIView>> {
        let mut responder = view.nextResponder();
        while let Some(current) = responder {
            if let Some(vc) = current.downcast_ref::<UIViewController>() {
                return vc.view();
            }
            responder = current.nextResponder();
        }
        None
    }

    fn contains_first_responder(view: &UIView) -> bool {
        view.isFirstResponder() || view.subviews().iter().any(|v| contains_first_responder(&v))
    }

    /// A visible-keyboard handoff need not emit another keyboard frame event.
    /// Activate just the destination's dock once it owns the responder.
    pub(super) fn sync_follower_for_responder(field: &UIView) {
        if !field.isFirstResponder() { return; }
        let Some(page) = page_for_view(field) else { return; };
        let lift = CURRENT_LIFT.with(Cell::get);
        let followers: Vec<_> = visible_followers().into_iter()
            .filter(|view| view.isDescendantOfView(&page)).collect();
        let mtm = field.mtm();
        let animations = block2::RcBlock::new(move || {
            for view in &followers { view.setTransform(transform(lift)); }
        });
        UIView::animateWithDuration_delay_options_animations_completion(
            0.2, 0.0, UIViewAnimationOptions::CurveEaseOut
                | UIViewAnimationOptions::BeginFromCurrentState,
            &animations, None, mtm,
        );
    }

    fn visible_followers() -> Vec<Retained<UIView>> {
        FOLLOWERS.with(|all| {
            let mut all = all.borrow_mut();
            all.retain(|weak| weak.load().is_some());
            all.iter().filter_map(|weak| {
                let view = weak.load()?;
                view.window().is_some().then_some(view)
            }).collect()
        })
    }

    fn interactive_coordinator()
    -> Option<Retained<ProtocolObject<dyn UIViewControllerTransitionCoordinator>>> {
        for view in visible_followers() {
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
            .find_map(|view| view.window())?;
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
            super::keyboard_lift(frame.visible_height, super::get_safe_area_bottom()) as CGFloat;
        CURRENT_LIFT.with(|current| current.set(lift));
        let views = visible_followers();
        // Create the layout guide on the first notification, so it is already
        // tracking UIKit's keyboard geometry when a later navigation completes.
        if let Some(window) = views.iter().find_map(|view| view.window()) {
            let _ = window.keyboardLayoutGuide();
        }
        if frame.duration > 0.0 {
            // MainThreadBound owns the native objects for the animation block.
            let views = dispatch2::MainThreadBound::new(views, mtm);
            let animations = block2::RcBlock::new(move || {
                let mtm = MainThreadMarker::new().expect("UIKit animations run on main");
                for view in views.get(mtm) {
                    unsafe {
                        let _: () = objc2::msg_send![&**view, setTransform: transform(lift)];
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
            for view in views {
                unsafe {
                    let _: () = objc2::msg_send![&*view, setTransform: transform(lift)];
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

        // Ignore overlay sheets / password manager windows that cover most of the screen (> 420pt).
        // The button dock should ONLY follow the actual on-screen keyboard.
        if visible_h > 420.0 {
            return;
        }

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

#[cfg(test)]
mod tests {
    use super::keyboard_lift;

    #[test]
    fn dock_stays_eight_points_above_keyboard_including_tall_layouts() {
        for safe_bottom in [0.0, 21.0, 34.0] {
            let page_bottom = 844.0 - safe_bottom;
            for height in [216.0, 336.0, 380.0, 430.0] {
                let button_bottom = page_bottom - 8.0 - keyboard_lift(height, safe_bottom);
                assert_eq!(button_bottom, 844.0 - height - 8.0);
            }
            assert_eq!(keyboard_lift(0.0, safe_bottom), 0.0);
            assert_eq!(keyboard_lift(safe_bottom, safe_bottom), 0.0);
        }
    }
}
