//! Device logging via NSLog bridge → idevicesyslog.

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
