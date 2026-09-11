//! Raw FFI wrapper for NSLog — the only way to get output into `idevicesyslog`.

#[cfg(target_os = "ios")]
mod ffi {
    use objc2::runtime::AnyObject;
    use objc2::msg_send;

    unsafe extern "C" {
        // NSLog is in Foundation; on Apple platforms the linker resolves via shared cache.
        // We pass "%@" format + one NSString arg to avoid format-string injection.
        #[link_name = "\x01_NSLog"]
        fn _NSLog(fmt: *const AnyObject, arg: *const AnyObject);
    }

    pub fn nslog(msg: &str) {
        use objc2::sel;
        unsafe {
            let cls = objc2::runtime::Class::get(c"NSString").unwrap();
            // NSString(stringWithUTF8String:) → autoreleased NSString*
            let fmt_ns: *mut AnyObject = msg_send![cls, stringWithUTF8String: b"%@\0".as_ptr()];
            let msg_ns: *mut AnyObject = msg_send![cls, stringWithUTF8String: msg.as_ptr()];
            _NSLog(fmt_ns, msg_ns);
        }
    }
}

#[cfg(not(target_os = "ios"))]
mod ffi {
    pub fn nslog(msg: &str) {
        eprintln!("{msg}");
    }
}

pub use ffi::nslog;
