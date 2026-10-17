use crate::app::AppState;
use day::prelude::*;

pub fn keyboard_height_signal() -> Signal<f64> {
    AppState::get_main()
        .map(|s| s.keyboard_height)
        .unwrap_or_else(|| Signal::new(0.0))
}

pub fn get_keyboard_height() -> f64 {
    keyboard_height_signal().get()
}

pub fn set_keyboard_height(h: f64) {
    if let Some(state) = AppState::get_main() {
        if (state.keyboard_height.get() - h).abs() > 0.5 {
            crate::shared::nslog::nslog(&format!("[Keyboard] Height changed to: {h:.1}"));
            state.keyboard_height.set(h);
        }
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
    use objc2::rc::Retained;
    use objc2::runtime::NSObjectProtocol;
    use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly};
    use objc2_foundation::{NSNotification, NSNotificationCenter, NSObject};
    use objc2_ui_kit::{
        UIApplication, UIKeyboardFrameEndUserInfoKey, UIKeyboardWillChangeFrameNotification,
        UIKeyboardWillHideNotification, NSValueUIGeometryExtensions,
    };
    use std::sync::atomic::{AtomicBool, Ordering};

    static OBSERVER_INSTALLED: AtomicBool = AtomicBool::new(false);

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "EschoolKeyboardObserver"]
        #[ivars = ()]
        pub struct EschoolKeyboardObserver;

        unsafe impl NSObjectProtocol for EschoolKeyboardObserver {}

        impl EschoolKeyboardObserver {
            #[unsafe(method(keyboardFrameChanged:))]
            fn keyboard_frame_changed(&self, notification: &NSNotification) {
                let Some(info) = notification.userInfo() else {
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

            #[unsafe(method(keyboardWillHide:))]
            fn keyboard_will_hide(&self, _notification: &NSNotification) {
                super::set_keyboard_height(0.0);
            }
        }
    );

    impl EschoolKeyboardObserver {
        fn new(mtm: MainThreadMarker) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(());
            unsafe { msg_send![super(this), init] }
        }
    }

    pub fn install() {
        if OBSERVER_INSTALLED.swap(true, Ordering::SeqCst) {
            return;
        }
        if let Some(mtm) = MainThreadMarker::new() {
            let app = UIApplication::sharedApplication(mtm);
            unsafe {
                let center = NSNotificationCenter::defaultCenter();

                // 1. Unregister Day's AppDelegate from UIKeyboardWillChangeFrameNotification.
                // In Day (day-uikit), AppDelegate::keyboard_will_change shrinks the root_view,
                // which causes the UITabBar (bottom navigation bar) to move up above the keyboard.
                // By unregistering it, the root window remains full screen and the bottom bar
                // stays pinned to the bottom of the screen instead of jumping up.
                if let Some(delegate) = app.delegate() {
                    let delegate_obj: &objc2::runtime::AnyObject = delegate.as_ref();
                    center.removeObserver_name_object(
                        delegate_obj,
                        Some(UIKeyboardWillChangeFrameNotification),
                        None,
                    );
                    crate::shared::nslog::nslog("[Keyboard] Unregistered Day AppDelegate keyboard observer");
                }

                // 2. Register our custom observer to dynamically track the keyboard height
                // for modal bottom sheets.
                let observer = EschoolKeyboardObserver::new(mtm);
                center.addObserver_selector_name_object(
                    &observer,
                    sel!(keyboardFrameChanged:),
                    Some(UIKeyboardWillChangeFrameNotification),
                    None,
                );
                center.addObserver_selector_name_object(
                    &observer,
                    sel!(keyboardWillHide:),
                    Some(UIKeyboardWillHideNotification),
                    None,
                );

                // Keep observer alive permanently
                let _ = Box::leak(Box::new(observer));
                crate::shared::nslog::nslog("[Keyboard] Custom keyboard observer installed");
            }
        }
    }
}
