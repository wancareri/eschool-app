//! Logging that actually shows up on iOS device.
//!
//! - `os_log` → system log (`idevicesyslog` / Console.app)
//! - File fallback → /tmp/eschool.log

#[cfg(target_os = "ios")]
mod ios_log {
    use std::ffi::CString;

    type OsLogT = *const std::ffi::c_void;

    unsafe extern "C" {
        #[link_name = "_os_log_create"]
        fn os_log_create(subsystem: *const i8, category: *const i8) -> OsLogT;

        #[link_name = "_os_log_impl"]
        fn os_log_impl(
            log: OsLogT,
            r#type: u8,
            dso: *const std::ffi::c_void,
            format: *const i8,
            ...
        );
    }

    static mut LOG_HANDLE: OsLogT = std::ptr::null();

    fn ensure_log() -> OsLogT {
        unsafe {
            if LOG_HANDLE.is_null() {
                let sub = CString::new("by.eschool.app").unwrap();
                let cat = CString::new("ESCHOOL").unwrap();
                LOG_HANDLE = os_log_create(sub.as_ptr(), cat.as_ptr());
            }
            LOG_HANDLE
        }
    }

    pub fn log(msg: &str) {
        let handle = ensure_log();
        let Ok(c_msg) = CString::new(msg) else { return };
        unsafe {
            os_log_impl(handle, 0, std::ptr::null(), c_msg.as_ptr());
        }
        // File fallback
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/eschool.log")
            .and_then(|mut f| {
                use std::io::Write;
                writeln!(f, "{}", msg)
            });
    }
}

#[cfg(not(target_os = "ios"))]
mod ios_log {
    pub fn log(msg: &str) {
        eprintln!("{msg}");
    }
}

pub fn nslog(msg: &str) {
    ios_log::log(&format!("ESCHOOL: {msg}"));
}
