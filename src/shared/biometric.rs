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

/// Attempt biometric authentication (shows Face ID / Touch ID prompt).
/// Blocks until the user responds.
pub fn authenticate() -> bool {
    if !is_available() {
        nslog::nslog("[Biometric] Not available");
        return false;
    }
    let has_token = crate::shared::secure::load("auth.token").is_some();
    if !has_token {
        nslog::nslog("[Biometric] No stored token");
        return false;
    }

    #[cfg(target_os = "ios")]
    {
        unsafe {
            use objc2::runtime::Bool;
            use objc2_local_authentication::{LAContext, LAPolicy};

            let ctx = LAContext::new();
            let reason = objc2_foundation::NSString::from_str("Вход в приложение");

            let (tx, rx) = std::sync::mpsc::sync_channel::<bool>(1);
            let tx_box: Box<std::sync::mpsc::SyncSender<bool>> = Box::new(tx);
            let tx_ptr: *mut std::sync::mpsc::SyncSender<bool> = Box::into_raw(tx_box);

            let block = block2::StackBlock::new(move |success: Bool, _error: *mut objc2_foundation::NSError| {
                let success_bool = success.as_bool();
                nslog::nslog(&format!("[Biometric] Reply: success={success_bool}"));
                let tx = Box::from_raw(tx_ptr);
                let _ = tx.send(success_bool);
            });

            ctx.evaluatePolicy_localizedReason_reply(
                LAPolicy::DeviceOwnerAuthenticationWithBiometrics,
                &reason,
                &block,
            );

            match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(success) => {
                    nslog::nslog(&format!("[Biometric] Result: {success}"));
                    success
                }
                Err(_) => {
                    nslog::nslog("[Biometric] Timeout waiting for reply");
                    false
                }
            }
        }
    }
    #[cfg(not(target_os = "ios"))]
    {
        true
    }
}
