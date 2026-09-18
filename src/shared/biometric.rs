//! Biometric auth support — Face ID / Touch ID on iOS.

use crate::shared::nslog;

const BIOMETRIC_KEY: &str = "auth.biometric_enabled";

/// Check if biometric auth is available on this device.
pub fn is_available() -> bool {
    #[cfg(target_os = "ios")]
    {
        unsafe {
            use objc2_local_authentication::{LAContext, LAPolicy};
            let ctx = LAContext::new();
            let result = ctx.canEvaluatePolicy_error(LAPolicy::DeviceOwnerAuthenticationWithBiometrics);
            let available = result.is_ok();
            nslog::nslog(&format!("[Biometric] canEvaluatePolicy: {available}"));
            available
        }
    }
    #[cfg(not(target_os = "ios"))]
    {
        false
    }
}

/// Check if user has enabled biometric login.
pub fn is_enabled() -> bool {
    crate::shared::secure::load(BIOMETRIC_KEY)
        .map(|v| v == "true")
        .unwrap_or(false)
}

/// Enable or disable biometric login.
pub fn set_enabled(enabled: bool) {
    crate::shared::secure::save(BIOMETRIC_KEY, if enabled { "true" } else { "false" });
    nslog::nslog(&format!("[Biometric] Set enabled: {enabled}"));
}

/// Trigger Face ID prompt. Non-blocking — posts evaluatePolicy to main thread.
/// Reply comes async on GCD queue, delivers result via setter.
pub fn authenticate_async(_state: crate::app::AppState) {
    #[cfg(target_os = "ios")]
    {
        let set_biometric_ok = _state.biometric_ok.setter();
        nslog::nslog("[Biometric] authenticate_async: posting to main thread");
        day::reactive::on_main(move || {
            nslog::nslog("[Biometric] on_main: creating LAContext");
            unsafe {
                use objc2::runtime::Bool;
                use objc2_local_authentication::{LAContext, LAPolicy};

                let ctx = LAContext::new();
                let reason = objc2_foundation::NSString::from_str("Вход в приложение");

                // Box::leak keeps the block alive for the async reply.
                let block_ref: &block2::DynBlock<dyn Fn(Bool, *mut objc2_foundation::NSError)> =
                    Box::leak(Box::new(block2::StackBlock::new(
                        move |success: Bool, _error: *mut objc2_foundation::NSError| {
                            let ok = success.as_bool();
                            nslog::nslog(&format!("[Biometric] Reply: success={ok}"));
                            // Setter is Copy+Send — dispatches to main thread via on_main
                            set_biometric_ok.set(ok);
                        },
                    )));

                nslog::nslog("[Biometric] Calling evaluatePolicy...");
                ctx.evaluatePolicy_localizedReason_reply(
                    LAPolicy::DeviceOwnerAuthenticationWithBiometrics,
                    &reason,
                    block_ref,
                );
                nslog::nslog("[Biometric] evaluatePolicy returned");
            }
        });
    }
}
