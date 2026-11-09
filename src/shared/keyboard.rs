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
        crate::shared::nslog::nslog(&format!("[Keyboard] Height updated to: {h:.1} in {duration_ms}ms"));
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
    use objc2::rc::Retained;
    use objc2::runtime::{AnyClass, AnyObject, Imp, NSObject, NSObjectProtocol, Sel};
    use objc2::{define_class, MainThreadMarker, MainThreadOnly};
    use objc2_core_foundation::{CGFloat, CGAffineTransform};
    use objc2_foundation::NSNotification;
    use objc2_ui_kit::{NSValueUIGeometryExtensions, UIKeyboardFrameEndUserInfoKey, UIViewAnimationOptions};
    use std::cell::{Cell, RefCell};
    use std::sync::atomic::{AtomicBool, Ordering};

    static INSTALLED: AtomicBool = AtomicBool::new(false);

    thread_local! {
        static OBSERVER: Cell<Option<Retained<KeyboardObserver>>> = const { Cell::new(None) };
        static HARDWARE_FOLLOWER: RefCell<Option<Retained<objc2_ui_kit::UIView>>> = const { RefCell::new(None) };
        static HARDWARE_STICKER: RefCell<Option<Retained<objc2_ui_kit::UIView>>> = const { RefCell::new(None) };
        static HARDWARE_CONTENT: RefCell<Option<Retained<objc2_ui_kit::UIView>>> = const { RefCell::new(None) };
        static CURRENT_LIFT: Cell<CGFloat> = const { Cell::new(0.0) };
    }

    pub fn register_follower_view(view: &Retained<objc2_ui_kit::UIView>) {
        let lift = CURRENT_LIFT.with(|l| l.get());
        if lift > 0.0 {
            let transform = CGAffineTransform {
                a: 1.0, b: 0.0,
                c: 0.0, d: 1.0,
                tx: 0.0, ty: -lift,
            };
            unsafe {
                let _: () = objc2::msg_send![&**view, setTransform: transform];
            }
        }
        HARDWARE_FOLLOWER.with(|c| {
            *c.borrow_mut() = Some(view.clone());
        });
        crate::shared::nslog::nslog("[Keyboard] Registered hardware follower view for instant 120fps keyboard tracking");
    }

    pub fn register_sticker_view(view: &Retained<objc2_ui_kit::UIView>) {
        let lift = CURRENT_LIFT.with(|l| l.get());
        if lift > 0.0 {
            let transform = CGAffineTransform {
                a: 0.48, b: 0.0,
                c: 0.0, d: 0.48,
                tx: 0.0, ty: -(lift * 0.50),
            };
            unsafe {
                let _: () = objc2::msg_send![&**view, setTransform: transform];
            }
        }
        HARDWARE_STICKER.with(|c| {
            *c.borrow_mut() = Some(view.clone());
        });
        crate::shared::nslog::nslog("[Keyboard] Registered hardware sticker view for smooth scaling");
    }

    pub fn register_content_view(view: &Retained<objc2_ui_kit::UIView>) {
        let lift = CURRENT_LIFT.with(|l| l.get());
        if lift > 0.0 {
            let transform = CGAffineTransform {
                a: 1.0, b: 0.0,
                c: 0.0, d: 1.0,
                tx: 0.0, ty: -lift,
            };
            unsafe {
                let _: () = objc2::msg_send![&**view, setTransform: transform];
            }
        }
        HARDWARE_CONTENT.with(|c| {
            *c.borrow_mut() = Some(view.clone());
        });
        crate::shared::nslog::nslog("[Keyboard] Registered hardware content view for smooth follower lifting");
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

        let dur_key = objc2_foundation::NSString::from_str("UIKeyboardAnimationDurationUserInfoKey");
        let duration_sec = info
            .objectForKey(&dur_key)
            .and_then(|o| o.downcast::<objc2_foundation::NSNumber>().ok())
            .map(|n| n.as_f64())
            .unwrap_or(0.25);
        let duration_ms = (duration_sec * 1000.0).round() as u32;

        let curve_key = objc2_foundation::NSString::from_str("UIKeyboardAnimationCurveUserInfoKey");
        let curve = info
            .objectForKey(&curve_key)
            .and_then(|o| o.downcast::<objc2_foundation::NSNumber>().ok())
            .map(|n| n.as_isize())
            .unwrap_or(7);

        let safe_b = super::get_safe_area_bottom();
        // Clamp lift to at most 340.0 pt. When iOS Password Manager / AutoFill modal sheet pops up,
        // UIKeyboardFrameEndUserInfoKey reports a full-screen or massive height,
        // which previously caused the button to shoot up to the very top of the screen.
        let max_lift = 340.0;
        let lift = if visible_h > 0.0 {
            (visible_h - safe_b).max(0.0).min(max_lift)
        } else {
            0.0
        };

        CURRENT_LIFT.with(|l| l.set(lift as CGFloat));

        // Hardware instant 120fps GPU follower translation:
        // Runs in the exact same CoreAnimation transaction as the keyboard!
        HARDWARE_FOLLOWER.with(|c| {
            if let Some(follower) = c.borrow().as_ref() {
                if follower.window().is_some() {
                    if let Some(mtm) = MainThreadMarker::new() {
                        let transform = if lift > 0.0 {
                            CGAffineTransform {
                                a: 1.0, b: 0.0,
                                c: 0.0, d: 1.0,
                                tx: 0.0, ty: -(lift as CGFloat),
                            }
                        } else {
                            CGAffineTransform {
                                a: 1.0, b: 0.0,
                                c: 0.0, d: 1.0,
                                tx: 0.0, ty: 0.0,
                            }
                        };
                        if duration_sec > 0.0 {
                            let options = ((curve as usize) << 16) | (1usize << 1);
                            let follower_clone = follower.clone();
                            let anim = block2::RcBlock::new(move || {
                                unsafe {
                                    let _: () = objc2::msg_send![&*follower_clone, setTransform: transform];
                                }
                            });
                            objc2_ui_kit::UIView::animateWithDuration_delay_options_animations_completion(
                                duration_sec,
                                0.0,
                                UIViewAnimationOptions(options),
                                &anim,
                                None,
                                mtm,
                            );
                        } else {
                            unsafe {
                                let _: () = objc2::msg_send![&**follower, setTransform: transform];
                            }
                        }
                    }
                }
            }
        });

        // Hardware smooth 120fps GPU sticker scaling:
        HARDWARE_STICKER.with(|c| {
            if let Some(sticker) = c.borrow().as_ref() {
                if sticker.window().is_some() {
                    if let Some(mtm) = MainThreadMarker::new() {
                        let transform = if lift > 0.0 {
                            // Scale down smoothly to ~48% and shift up towards top bar:
                            // Centers the duck in the remaining space above the content!
                            let ty = -((lift * 0.50) as CGFloat);
                            CGAffineTransform {
                                a: 0.48, b: 0.0,
                                c: 0.0, d: 0.48,
                                tx: 0.0, ty,
                            }
                        } else {
                            CGAffineTransform {
                                a: 1.0, b: 0.0,
                                c: 0.0, d: 1.0,
                                tx: 0.0, ty: 0.0,
                            }
                        };
                        if duration_sec > 0.0 {
                            let options = ((curve as usize) << 16) | (1usize << 1);
                            let sticker_clone = sticker.clone();
                            let anim = block2::RcBlock::new(move || {
                                unsafe {
                                    let _: () = objc2::msg_send![&*sticker_clone, setTransform: transform];
                                }
                            });
                            objc2_ui_kit::UIView::animateWithDuration_delay_options_animations_completion(
                                duration_sec,
                                0.0,
                                UIViewAnimationOptions(options),
                                &anim,
                                None,
                                mtm,
                            );
                        } else {
                            unsafe {
                                let _: () = objc2::msg_send![&**sticker, setTransform: transform];
                            }
                        }
                    }
                }
            }
        });

        // Hardware smooth 120fps GPU content follower:
        // Pinned to the bottom! Moves up in exact 1:1 sync with the button and keyboard!
        HARDWARE_CONTENT.with(|c| {
            if let Some(content) = c.borrow().as_ref() {
                if content.window().is_some() {
                    if let Some(mtm) = MainThreadMarker::new() {
                        let transform = if lift > 0.0 {
                            CGAffineTransform {
                                a: 1.0, b: 0.0,
                                c: 0.0, d: 1.0,
                                tx: 0.0, ty: -(lift as CGFloat),
                            }
                        } else {
                            CGAffineTransform {
                                a: 1.0, b: 0.0,
                                c: 0.0, d: 1.0,
                                tx: 0.0, ty: 0.0,
                            }
                        };
                        if duration_sec > 0.0 {
                            let options = ((curve as usize) << 16) | (1usize << 1);
                            let content_clone = content.clone();
                            let anim = block2::RcBlock::new(move || {
                                unsafe {
                                    let _: () = objc2::msg_send![&*content_clone, setTransform: transform];
                                }
                            });
                            objc2_ui_kit::UIView::animateWithDuration_delay_options_animations_completion(
                                duration_sec,
                                0.0,
                                UIViewAnimationOptions(options),
                                &anim,
                                None,
                                mtm,
                            );
                        } else {
                            unsafe {
                                let _: () = objc2::msg_send![&**content, setTransform: transform];
                            }
                        }
                    }
                }
            }
        });

        super::set_keyboard_height_with_duration(visible_h, duration_ms);
    }

    pub fn install() {
        if INSTALLED.swap(true, Ordering::SeqCst) {
            return;
        }

        // Swizzle DayAppDelegate's keyboardWillChange: method.
        // In day-uikit, DayAppDelegate::keyboard_will_change shrinks root_view.frame
        // to clamp to the keyboard top, which forces the UITabBarController's tab bar
        // to float up above the keyboard.
        // By replacing the method with our own handler:
        // 1. Day's original root_view shrinking is bypassed completely — the tab bar
        //    stays at the bottom of the screen and NEVER jumps up.
        // 2. Our handler gets the keyboard end frame directly from UIKit and updates
        //    our reactive `keyboard_height` signal, allowing modal bottom sheets
        //    to smoothly shift up and remain fully visible above the keyboard.
        let cls = AnyClass::get(c"DayAppDelegate");
        if let Some(cls) = cls {
            let sel = objc2::sel!(keyboardWillChange:);
            if let Some(method) = cls.instance_method(sel) {
                let imp: Imp = unsafe { std::mem::transmute(swizzled_keyboard_will_change as *const ()) };
                unsafe {
                    method.set_implementation(imp);
                }
                crate::shared::nslog::nslog("[Keyboard] Successfully swizzled DayAppDelegate keyboardWillChange:");
            } else {
                crate::shared::nslog::nslog("[Keyboard] WARNING: keyboardWillChange: method not found on DayAppDelegate");
            }
        } else {
            crate::shared::nslog::nslog("[Keyboard] WARNING: DayAppDelegate class not found");
        }

        // Register single dedicated observer for UIKeyboardWillChangeFrameNotification.
        // This is the canonical master notification that covers show, hide, orientation,
        // and accessory/QuickType bar transitions with full animation duration and curve.
        if let Some(mtm) = MainThreadMarker::new() {
            let obs: Retained<KeyboardObserver> = unsafe { objc2::msg_send![KeyboardObserver::alloc(mtm), init] };
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
            crate::shared::nslog::nslog("[Keyboard] Registered KeyboardObserver for UIKeyboardWillChangeFrameNotification");
        }

        // Swizzle UITextField setBorderStyle: to force UITextBorderStyleNone (0) so text fields have no native border/outline
        let tf_cls = AnyClass::get(c"UITextField");
        if let Some(cls) = tf_cls {
            let sel = objc2::sel!(setBorderStyle:);
            if let Some(method) = cls.instance_method(sel) {
                let orig: Imp = method.implementation();
                ORIG_SET_BORDER_STYLE.store(orig as *mut (), std::sync::atomic::Ordering::Release);
                let imp: Imp = unsafe { std::mem::transmute(swizzled_set_border_style as *const ()) };
                unsafe {
                    method.set_implementation(imp);
                }
                crate::shared::nslog::nslog("[TextField] Successfully swizzled UITextField setBorderStyle: to force None");
            }

            let sel_sec = objc2::sel!(setSecureTextEntry:);
            if let Some(method) = cls.instance_method(sel_sec) {
                let orig: Imp = method.implementation();
                ORIG_SET_SECURE_TEXT_ENTRY.store(orig as *mut (), std::sync::atomic::Ordering::Release);
                let imp: Imp = unsafe { std::mem::transmute(swizzled_set_secure_text_entry as *const ()) };
                unsafe {
                    method.set_implementation(imp);
                }
                crate::shared::nslog::nslog("[TextField] Successfully swizzled UITextField setSecureTextEntry:");
            }
        }
    }

    static ORIG_SET_BORDER_STYLE: std::sync::atomic::AtomicPtr<()> =
        std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());
    static ORIG_SET_SECURE_TEXT_ENTRY: std::sync::atomic::AtomicPtr<()> =
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
        unsafe {
            let _: () = objc2::msg_send![this, setAutocapitalizationType: 0isize];
            // Leave autocorrectionType & spellCheckingType as Default (0)
            // so iOS QuickType / Passwords bar does NOT collapse when switching to Russian keyboard!
            let _: () = objc2::msg_send![this, setAutocorrectionType: 0isize];
            let _: () = objc2::msg_send![this, setSpellCheckingType: 0isize];
            let is_secure: bool = objc2::msg_send![this, isSecureTextEntry];
            let ctype = if is_secure {
                objc2_foundation::NSString::from_str("password")
            } else {
                objc2_foundation::NSString::from_str("username")
            };
            let _: () = objc2::msg_send![this, setTextContentType: &*ctype];

            // Lock the input assistant bar visible on all keyboards
            let assistant: Option<Retained<AnyObject>> = objc2::msg_send![this, inputAssistantItem];
            if let Some(item) = assistant {
                let _: () = objc2::msg_send![&*item, setAllowsHidingShortcuts: false];
            }
        }
    }

    unsafe extern "C-unwind" fn swizzled_set_secure_text_entry(
        this: *mut AnyObject,
        cmd: Sel,
        secure: bool,
    ) {
        let orig_ptr = ORIG_SET_SECURE_TEXT_ENTRY.load(std::sync::atomic::Ordering::Acquire);
        if !orig_ptr.is_null() {
            let orig: unsafe extern "C-unwind" fn(*mut AnyObject, Sel, bool) =
                unsafe { std::mem::transmute(orig_ptr) };
            unsafe {
                orig(this, cmd, secure);
            }
        }
        unsafe {
            let _: () = objc2::msg_send![this, setAutocapitalizationType: 0isize];
            let _: () = objc2::msg_send![this, setAutocorrectionType: 0isize];
            let _: () = objc2::msg_send![this, setSpellCheckingType: 0isize];
            let ctype = if secure {
                objc2_foundation::NSString::from_str("password")
            } else {
                objc2_foundation::NSString::from_str("username")
            };
            let _: () = objc2::msg_send![this, setTextContentType: &*ctype];

            let assistant: Option<Retained<AnyObject>> = objc2::msg_send![this, inputAssistantItem];
            if let Some(item) = assistant {
                let _: () = objc2::msg_send![&*item, setAllowsHidingShortcuts: false];
            }
        }
    }
}
