//! Device logging via NSLog bridge → idevicesyslog.
//!
//! The Objective-C bridge (platform/ios/nslog_bridge.m) calls
//! NSLog(@"%{public}s", msg) — the `%{public}s` is critical, without it
//! iOS redacts the message to a space in idevicesyslog.

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn eschool_nslog(msg: *const i8);
}

#[cfg(target_os = "ios")]
pub fn nslog(msg: &str) {
    let tagged = format!("ESCHOOL: {msg}\0");
    unsafe {
        eschool_nslog(tagged.as_ptr() as *const i8);
    }
}

#[cfg(not(target_os = "ios"))]
pub fn nslog(msg: &str) {
    eprintln!("ESCHOOL: {msg}");
}
