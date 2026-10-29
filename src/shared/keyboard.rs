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

pub fn set_keyboard_height(h: f64) {
    let sig = keyboard_height_signal();
    if (sig.get() - h).abs() > 0.5 {
        crate::shared::nslog::nslog(&format!("[Keyboard] Height updated to: {h:.1}"));
        with_animation(AnimSpec::ease_out(380), move || {
            sig.set(h);
        });
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
    ios::install();
}

#[cfg(target_os = "ios")]
mod ios {
    use objc2::runtime::{AnyClass, AnyObject, Imp, Sel};
    use objc2_foundation::NSNotification;
    use objc2_ui_kit::{NSValueUIGeometryExtensions, UIKeyboardFrameEndUserInfoKey};
    use std::sync::atomic::{AtomicBool, Ordering};

    static INSTALLED: AtomicBool = AtomicBool::new(false);

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
        super::set_keyboard_height(visible_h);
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
