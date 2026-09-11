//! Device logging via os_log (shows in idevicesyslog).
//! Uses dlsym to avoid static linking — no framework linking needed.

#[cfg(target_os = "ios")]
mod inner {
    use std::ffi::CString;
    use std::sync::Once;

    type LogFn = unsafe extern "C" fn(*const std::ffi::c_void, u8, *const std::ffi::c_void, *const i8, ...);
    type CreateFn = unsafe extern "C" fn(*const i8, *const i8) -> *const std::ffi::c_void;

    struct LogState {
        handle: std::cell::UnsafeCell<*const std::ffi::c_void>,
        log_fn: std::cell::UnsafeCell<Option<LogFn>>,
    }

    unsafe impl Send for LogState {}
    unsafe impl Sync for LogState {}

    static STATE: LogState = LogState {
        handle: std::cell::UnsafeCell::new(std::ptr::null()),
        log_fn: std::cell::UnsafeCell::new(None),
    };
    static INIT: Once = Once::new();

    fn ensure() {
        INIT.call_once(|| {
            unsafe {
                let lib = libc::dlopen(
                    b"libSystem.B.dylib\0".as_ptr().cast(),
                    libc::RTLD_LAZY,
                );
                if lib.is_null() { return; }

                let create_sym = libc::dlsym(lib, b"os_log_create\0".as_ptr().cast());
                let log_sym = libc::dlsym(lib, b"os_log_impl\0".as_ptr().cast());

                if create_sym.is_null() || log_sym.is_null() { return; }

                let create_fn: CreateFn = std::mem::transmute(create_sym);
                let log_fn: LogFn = std::mem::transmute(log_sym);

                let sub = CString::new("by.eschool.app").unwrap();
                let cat = CString::new("ESCHOOL").unwrap();
                *STATE.handle.get() = create_fn(sub.as_ptr(), cat.as_ptr());
                *STATE.log_fn.get() = Some(log_fn);
            }
        });
    }

    pub fn log(msg: &str) {
        ensure();
        let tagged = format!("ESCHOOL: {msg}");
        let Ok(c_msg) = CString::new(tagged.as_str()) else { return };
        unsafe {
            if let Some(f) = *STATE.log_fn.get() {
                f(*STATE.handle.get(), 0, std::ptr::null(), c_msg.as_ptr());
            }
        }
    }
}

#[cfg(not(target_os = "ios"))]
mod inner {
    pub fn log(msg: &str) {
        eprintln!("ESCHOOL: {msg}");
    }
}

pub fn nslog(msg: &str) {
    inner::log(msg);
}
