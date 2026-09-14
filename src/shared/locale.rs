//! System locale detection — maps the device language to our locale codes.

pub fn system_locale() -> &'static str {
    #[cfg(target_os = "ios")]
    {
        ios_preferred_locale()
    }
    #[cfg(not(target_os = "ios"))]
    {
        "en"
    }
}

#[cfg(target_os = "ios")]
fn ios_preferred_locale() -> &'static str {
    use std::ffi::{CStr, c_void};

    #[link(name = "objc", kind = "dylib")]
    unsafe extern "C" {
        fn objc_getClass(name: *const std::ffi::c_char) -> *const c_void;
        fn sel_registerName(name: *const std::ffi::c_char) -> *const c_void;
        fn objc_msgSend() -> *const c_void;
    }

    unsafe {
        let cls = objc_getClass(c"NSLocale".as_ptr());
        if cls.is_null() {
            return "en";
        }
        let sel = sel_registerName(c"preferredLanguages".as_ptr());

        type MsgSendFn = unsafe extern "C" fn(*const c_void, *const c_void) -> *const c_void;
        let msg_send: MsgSendFn = std::mem::transmute(objc_msgSend as *const c_void);
        let langs = msg_send(cls, sel);
        if langs.is_null() {
            return "en";
        }

        let count_sel = sel_registerName(c"count".as_ptr());
        let count: usize = {
            type Fn = unsafe extern "C" fn(*const c_void, *const c_void) -> usize;
            let f: Fn = std::mem::transmute(objc_msgSend as *const c_void);
            f(langs, count_sel)
        };
        if count == 0 {
            return "en";
        }

        let obj = {
            type Fn = unsafe extern "C" fn(*const c_void, *const c_void, usize) -> *const c_void;
            let f: Fn = std::mem::transmute(objc_msgSend as *const c_void);
            let idx_sel = sel_registerName(c"objectAtIndex:".as_ptr());
            f(langs, idx_sel, 0)
        };
        if obj.is_null() {
            return "en";
        }

        let utf8_sel = sel_registerName(c"UTF8String".as_ptr());
        let ptr = {
            type Fn = unsafe extern "C" fn(*const c_void, *const c_void) -> *const std::ffi::c_char;
            let f: Fn = std::mem::transmute(objc_msgSend as *const c_void);
            f(obj, utf8_sel)
        };
        if ptr.is_null() {
            return "en";
        }

        let rust_str = CStr::from_ptr(ptr).to_str().unwrap_or("en");
        let code = rust_str.split('-').next().unwrap_or("en");
        match code {
            "ru" => "ru",
            "be" => "be",
            _ => "en",
        }
    }
}
