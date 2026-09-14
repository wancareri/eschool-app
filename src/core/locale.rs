//! System locale detection — maps the device language to our locale codes.

/// Detect the system language and return a locale code supported by the app.
pub(crate) fn system_locale() -> &'static str {
    #[cfg(target_os = "ios")]
    {
        return ios_preferred_locale();
    }
    #[cfg(not(target_os = "ios"))]
    {
        "en"
    }
}

#[cfg(target_os = "ios")]
fn ios_preferred_locale() -> &'static str {
    use std::ffi::{CStr, c_char};

    extern "C" {
        fn objc_msgSend() -> *const std::ffi::c_void;
        fn sel_registerName(name: *const c_char) -> *const std::ffi::c_void;
        fn objc_getClass(name: *const c_char) -> *const std::ffi::c_void;
    }

    unsafe {
        let cls_name = c"NSLocale";
        let cls = objc_getClass(cls_name.as_ptr());
        if cls.is_null() {
            return "en";
        }

        let sel_name = c"preferredLanguages";
        let sel = sel_registerName(sel_name.as_ptr());

        let msg: extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void) -> *mut ObjcArray =
            std::mem::transmute(objc_msgSend);
        let langs = msg(cls, sel);
        if langs.is_null() {
            return "en";
        }

        // NSArray count
        let count_fn: extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void) -> usize =
            std::mem::transmute(objc_msgSend);
        let count_sel = sel_registerName(c"count".as_ptr());
        let count = count_fn(langs as *const std::ffi::c_void, count_sel);
        if count == 0 {
            return "en";
        }

        // objectAtIndex:
        let idx_sel = sel_registerName(c"objectAtIndex:".as_ptr());
        let obj_fn: extern "C" fn(
            *const std::ffi::c_void,
            *const std::ffi::c_void,
            usize,
        ) -> *mut ObjcString = std::mem::transmute(objc_msgSend);
        let obj = obj_fn(langs as *const std::ffi::c_void, idx_sel, 0);
        if obj.is_null() {
            return "en";
        }

        // UTF8String
        let utf8_sel = sel_registerName(c"UTF8String".as_ptr());
        let utf8_fn: extern "C" fn(
            *const std::ffi::c_void,
            *const std::ffi::c_void,
        ) -> *const c_char = std::mem::transmute(objc_msgSend);
        let ptr = utf8_fn(obj as *const std::ffi::c_void, utf8_sel);
        if ptr.is_null() {
            return "en";
        }

        let cstr = CStr::from_ptr(ptr);
        let rust_str = cstr.to_str().unwrap_or("en");
        let code = rust_str.split('-').next().unwrap_or("en");
        match code {
            "ru" => "ru",
            "be" => "be",
            _ => "en",
        }
    }
}

#[cfg(target_os = "ios")]
#[repr(C)]
struct ObjcArray {
    _private: [u8; 0],
}

#[cfg(target_os = "ios")]
#[repr(C)]
struct ObjcString {
    _private: [u8; 0],
}
