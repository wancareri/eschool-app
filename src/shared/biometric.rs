//! Biometric auth support — Face ID / Touch ID on iOS.
//! Uses LAContext for actual biometric authentication.

use crate::shared::nslog;

const BIOMETRIC_KEY: &str = "auth.biometric_enabled";

/// Check if biometric auth is available on this device.
pub fn is_available() -> bool {
    #[cfg(target_os = "ios")]
    {
        unsafe {
            let cls = objc2::runtime::Class::get("LAContext")
                .expect("LAContext class not found");
            let ctx: *mut objc2::runtime::Object = objc2::msg_send![cls, new];
            let mut error: *mut objc2::runtime::Object = std::ptr::null_mut();
            let available: bool = objc2::msg_send![ctx, canEvaluatePolicy: 1i64 error: &mut error];
            let _ = objc2::msg_send![ctx, autorelease];
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

/// Attempt biometric authentication.
/// Returns true if biometric check passed.
pub fn authenticate() -> bool {
    if !is_available() {
        nslog::nslog("[Biometric] Not available on this device");
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
            let cls = objc2::runtime::Class::get("LAContext")
                .expect("LAContext class not found");
            let ctx: *mut objc2::runtime::Object = objc2::msg_send![cls, new];

            let reason = objc2_foundation::NSString::from_str("Вход в приложение");
            let mut error: *mut objc2::runtime::Object = std::ptr::null_mut();

            let semaphore = std::sync::Arc::new(std::sync::Mutex::new(false));
            let sem_clone = semaphore.clone();

            let block = block2::StackBlock::new(move |success: bool, _error: *mut objc2::runtime::Object| {
                if let Ok(mut val) = sem_clone.lock() {
                    *val = success;
                }
            });

            objc2::msg_send![ctx, evaluatePolicy: 1i64 localizedReason: &*reason replyBlock: &*block];

            // Wait for result (max 10 seconds)
            let start = std::time::Instant::now();
            while start.elapsed() < std::time::Duration::from_secs(10) {
                if let Ok(val) = semaphore.lock() {
                    if *val {
                        nslog::nslog("[Biometric] Auth success");
                        let _ = objc2::msg_send![ctx, autorelease];
                        return true;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }

            nslog::nslog("[Biometric] Auth timeout/failure");
            let _ = objc2::msg_send![ctx, autorelease];
            false
        }
    }
    #[cfg(not(target_os = "ios"))]
    {
        nslog::nslog("[Biometric] Non-iOS, returning true for dev");
        true
    }
}
