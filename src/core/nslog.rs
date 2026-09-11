//! Device logging via os_log → system log (idevicesyslog).

#[cfg(target_os = "ios")]
mod inner {
    use std::sync::Once;

    type LogFn = unsafe extern "C" fn(*const std::ffi::c_void, u8, *const std::ffi::c_void, *const i8, ...);
    type CreateFn = unsafe extern "C" fn(*const i8, *const i8) -> *const std::ffi::c_void;

    struct SyncPtr(std::cell::UnsafeCell<*const std::ffi::c_void>);
    unsafe impl Send for SyncPtr {}
    unsafe impl Sync for SyncPtr {}

    struct SyncOptFn(std::cell::UnsafeCell<Option<LogFn>>);
    unsafe impl Send for SyncOptFn {}
    unsafe impl Sync for SyncOptFn {}

    static HANDLE: SyncPtr = SyncPtr(std::cell::UnsafeCell::new(std::ptr::null()));
    static LOG_FN: SyncOptFn = SyncOptFn(std::cell::UnsafeCell::new(None));
    static INIT: Once = Once::new();

    fn ensure() {
        INIT.call_once(|| {
            unsafe {
                let lib = libc::dlopen(b"libSystem.B.dylib\0".as_ptr().cast(), libc::RTLD_LAZY);
                if lib.is_null() { return; }
                let cs = libc::dlsym(lib, b"os_log_create\0".as_ptr().cast());
                let lf = libc::dlsym(lib, b"os_log_impl\0".as_ptr().cast());
                if cs.is_null() || lf.is_null() { return; }
                let create_fn: CreateFn = std::mem::transmute(cs);
                let log_fn: LogFn = std::mem::transmute(lf);
                *HANDLE.0.get() = create_fn(b"by.eschool.app\0".as_ptr().cast(), b"ESCHOOL\0".as_ptr().cast());
                *LOG_FN.0.get() = Some(log_fn);
            }
        });
    }

    pub fn log(msg: &str) {
        ensure();
        let tagged = format!("{msg}\0");
        unsafe {
            if let Some(f) = *LOG_FN.0.get() {
                f(*HANDLE.0.get(), 0, std::ptr::null(), tagged.as_ptr() as *const i8);
            }
        }
    }
}

#[cfg(not(target_os = "ios"))]
mod inner {
    pub fn log(msg: &str) {
        eprintln!("{msg}");
    }
}

pub fn nslog(msg: &str) {
    inner::log(&format!("ESCHOOL: {msg}"));
}
