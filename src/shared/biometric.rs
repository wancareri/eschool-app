//! Biometric auth support — Face ID / Touch ID on iOS.

use crate::shared::nslog;
use std::sync::atomic::{AtomicBool, Ordering};

const BIOMETRIC_KEY: &str = "auth.biometric_enabled";
pub static BIOMETRIC_OK: AtomicBool = AtomicBool::new(false);

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
/// Reply comes async on GCD queue, delivers result via on_main.
pub fn authenticate_async() {
    #[cfg(target_os = "ios")]
    {
        nslog::nslog("[Biometric] authenticate_async: posting to main thread");
        day::reactive::on_main(|| {
            nslog::nslog("[Biometric] on_main: creating LAContext");
            unsafe {
                use objc2::runtime::Bool;
                use objc2_local_authentication::{LAContext, LAPolicy};

                let ctx = LAContext::new();
                let reason = objc2_foundation::NSString::from_str("Вход в приложение");

                // Box::leak keeps the block alive for the async reply.
                // Without leaking, StackBlock would be dropped when this scope ends,
                // but evaluatePolicy returns immediately and the reply comes later.
                let block_ref: &block2::DynBlock<dyn Fn(Bool, *mut objc2_foundation::NSError)> =
                    Box::leak(Box::new(block2::StackBlock::new(
                        move |success: Bool, _error: *mut objc2_foundation::NSError| {
                            let ok = success.as_bool();
                            nslog::nslog(&format!("[Biometric] Reply: success={ok}"));
                            // Set atomic flag directly — no on_main needed here.
                            // The watch in window_shell detects the change on main thread.
                            BIOMETRIC_OK.store(ok, Ordering::Relaxed);
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
