//! System locale detection — maps the device language to our locale codes.

/// Detect the system language and return a locale code supported by the app.
///
/// On iOS this reads `NSLocale.preferredLanguages`; on other platforms returns
/// `"en"`.
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

/// Read the first preferred language from `NSLocale.preferredLanguages` and
/// map it to one of our supported locales (`en`, `ru`, `be`).
#[cfg(target_os = "ios")]
fn ios_preferred_locale() -> &'static str {
    use objc2::runtime::AnyClass;
    use objc2::msg_send;
    use objc2_foundation::NSString;

    unsafe {
        let cls = AnyClass::get(c"NSLocale").expect("NSLocale class");
        let langs: objc2_foundation::NSArray<NSString> =
            msg_send![cls, preferredLanguages];
        if langs.count() == 0 {
            return "en";
        }
        let first = langs.objectAtIndex(0);
        let rust_str = first.to_string();
        let code = rust_str.split('-').next().unwrap_or("en");
        match code {
            "ru" => "ru",
            "be" => "be",
            _ => "en",
        }
    }
}
