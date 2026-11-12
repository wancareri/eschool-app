#[cfg(feature = "uikit")]
pub fn install_lifecycle_observers() {
    use crate::pages::pin::{biometric, security};
    use objc2::rc::Retained;
    use objc2::{MainThreadMarker, MainThreadOnly, define_class};
    use objc2_foundation::{NSNotification, NSNotificationCenter, NSObject, NSObjectProtocol};

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "EschoolLockLifecycleObserver"]
        struct LockObserver;
        unsafe impl NSObjectProtocol for LockObserver {}
        impl LockObserver {
            #[unsafe(method(didEnterBackground:))]
            fn did_enter_background(&self, _notification: &NSNotification) {
                security::save_last_background();
            }
            #[unsafe(method(willEnterForeground:))]
            fn will_enter_foreground(&self, _notification: &NSNotification) {
                let Some(state) = crate::app::AppState::get_main() else { return };
                if !state.is_authenticated.get() || !security::should_auto_lock() { return; }
                let pin = security::is_enabled();
                let biometric = biometric::is_enabled()
                    && biometric::is_available();
                if !pin && !biometric { return; }
                crate::app::data::session::invalidate();
                crate::pages::diary::data::clear_data(state);
                state.biometric_ok.set(false);
                state.pin_input.set(String::new());
                state.pin_error.set(false);
                state.is_authenticated.set(false);
                state.pin_lock_active.set(pin);
                // The pin_lock_active watcher handles the PIN+Face ID case.
                if biometric && !pin { biometric::authenticate_async(state); }
            }
        }
    );
    thread_local! {
        static OBSERVER: std::cell::RefCell<Option<Retained<LockObserver>>> = const { std::cell::RefCell::new(None) };
    }
    OBSERVER.with(|slot| {
        if slot.borrow().is_some() {
            return;
        }
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let observer: Retained<LockObserver> =
            unsafe { objc2::msg_send![LockObserver::alloc(mtm), init] };
        let center = NSNotificationCenter::defaultCenter();
        unsafe {
            center.addObserver_selector_name_object(
                &observer,
                objc2::sel!(didEnterBackground:),
                Some(objc2_ui_kit::UIApplicationDidEnterBackgroundNotification),
                None,
            );
            center.addObserver_selector_name_object(
                &observer,
                objc2::sel!(willEnterForeground:),
                Some(objc2_ui_kit::UIApplicationWillEnterForegroundNotification),
                None,
            );
        }
        *slot.borrow_mut() = Some(observer);
    });
}

