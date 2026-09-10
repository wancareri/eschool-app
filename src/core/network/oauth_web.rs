//! Web-based OAuth flow using WKWebView on iOS.
//!
//! Uses Safari's TLS stack (no JA3/JA4 fingerprint issues). The web view
//! navigates through the full OAuth flow. When it lands on the preauthorized
//! page, we extract the UUID and make native API calls for token exchange.

#[cfg(target_os = "ios")]
mod ios_impl {
    use std::sync::{Arc, Condvar, Mutex};
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, ProtocolObject};
    use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly, DefinedClass};
    use objc2_foundation::{NSObject, NSString, NSURL, NSURLRequest};
    use objc2_core_foundation::CGRect;
    use objc2_ui_kit::{UIView, UIViewController, UIApplication, UIWindow};
    use day::prelude::*;
    use super::super::auth::{Auth, Token, AuthResponse, BASE_URL};

    pub(super) enum OAuthResult {
        Ok(String),
        Err(String),
    }

    type SharedState = Arc<(Mutex<Option<OAuthResult>>, Condvar)>;

    struct DelegateIvars {
        shared: SharedState,
        presented_vc: Mutex<Option<Retained<UIViewController>>>,
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "DayOAuthNavDelegate"]
        #[ivars = DelegateIvars]
        struct OAuthNavDelegate;

        impl OAuthNavDelegate {
            #[unsafe(method(webView:didFinishNavigation:))]
            fn did_finish_navigation(
                &self,
                web_view: &AnyObject,
                _navigation: Option<&AnyObject>,
            ) {
                let url: Option<Retained<NSURL>> = unsafe { msg_send![web_view, URL] };
                let url_str = url
                    .and_then(|u| u.absoluteString())
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                log::info!("[OAuth] didFinish: {}", &url_str[..url_str.len().min(200)]);

                if let Some(uuid) = extract_uuid_from_url(&url_str) {
                    log::info!("[OAuth] Found UUID: {}", uuid);
                    self.signal_result(OAuthResult::Ok(uuid));
                    self.dismiss(web_view);
                    return;
                }

                if url_str.contains("login/error") {
                    log::error!("[OAuth] Login error detected");
                    self.signal_result(OAuthResult::Err("Login failed".into()));
                    self.dismiss(web_view);
                    return;
                }

                if url_str.contains("diary.e-schools.by") {
                    log::info!("[OAuth] On diary page, reading localStorage...");
                    self.read_local_storage(web_view);
                }
            }
        }
    );

    impl OAuthNavDelegate {
        fn new(mtm: MainThreadMarker, shared: SharedState) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(DelegateIvars {
                shared,
                presented_vc: Mutex::new(None),
            });
            unsafe { msg_send![super(this), init] }
        }

        fn signal_result(&self, result: OAuthResult) {
            let ivars = self.ivars();
            let mut guard = ivars.shared.0.lock().unwrap();
            *guard = Some(result);
            ivars.shared.1.notify_one();
        }

        fn dismiss(&self, web_view: &AnyObject) {
            unsafe { let _: () = msg_send![web_view, stopLoading]; };
            if let Some(vc) = self.ivars().presented_vc.lock().unwrap().take() {
                unsafe { let _: () = msg_send![&*vc, dismissViewControllerAnimated:true completion:null_block()]; };
            }
        }

        fn read_local_storage(&self, web_view: &AnyObject) {
            let js = NSString::from_str(
                "try { localStorage.getItem('e-school-auth-diary'); } catch(e) { null; }"
            );

            let shared = self.ivars().shared.clone();
            let wv = web_view as *const AnyObject;

            let block = block2::RcBlock::new(move |result: *mut AnyObject, _error: *mut AnyObject| {
                if result.is_null() {
                    return;
                }
                unsafe {
                    let is_string: bool = msg_send![&*result, isKindOfClass: objc2::runtime::Class::get(c"NSString").unwrap()];
                    if !is_string {
                        return;
                    }
                    let s: String = {
                        let ns_str: &NSString = &*(result as *mut NSString);
                        ns_str.to_string()
                    };
                    if s.is_empty() || s == "null" {
                        return;
                    }
                    log::info!("[OAuth] localStorage: {}...", &s[..s.len().min(100)]);

                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) {
                        let auth_token = val["authToken"].as_str().unwrap_or("");
                        let refresh_token = val["refreshToken"].as_str().unwrap_or("");
                        if !auth_token.is_empty() {
                            let mut guard = shared.0.lock().unwrap();
                            *guard = Some(OAuthResult::Ok(format!(
                                "token:{}:{}", auth_token, refresh_token
                            )));
                            shared.1.notify_one();
                            let _: () = msg_send![wv, stopLoading];
                        }
                    }
                }
            });

            unsafe {
                let ptr = block2::RcBlock::as_ptr(&block);
                let _: () = msg_send![web_view, evaluateJavaScript:&*js completionHandler:ptr];
                std::mem::forget(block);
            }
        }
    }

    fn null_block() -> *mut block2::Block<dyn Fn()> {
        std::ptr::null_mut()
    }

    pub fn login_with_web_view(auth: &Auth, _username: &str, _password: &str) -> Result<Token, String> {
        use std::time::Duration;

        let shared: SharedState = Arc::new((Mutex::new(None), Condvar::new()));
        let shared_clone = shared.clone();
        let login_url = auth.login_url();

        dispatch2::DispatchQueue::main().exec_async(move || {
            let mtm = MainThreadMarker::new().expect("must be on main thread");

            let config: Retained<AnyObject> = unsafe {
                let cls = objc2::runtime::Class::get(c"WKWebViewConfiguration").unwrap();
                let obj: *mut AnyObject = msg_send![cls, new];
                Retained::retain(obj).unwrap()
            };

            let data_store: Retained<AnyObject> = unsafe {
                let cls = objc2::runtime::Class::get(c"WKWebsiteDataStore").unwrap();
                let obj: *mut AnyObject = msg_send![cls, defaultDataStore];
                Retained::retain(obj).unwrap()
            };

            unsafe { let _: () = msg_send![&*config, setWebsiteDataStore:&*data_store]; };

            let bounds: CGRect = unsafe {
                let app = UIApplication::sharedApplication(mtm);
                let windows = app.windows();
                let window: &UIWindow = &windows.objectAtIndex(0);
                msg_send![window, bounds]
            };

            let web_view: Retained<AnyObject> = unsafe {
                let cls = objc2::runtime::Class::get(c"WKWebView").unwrap();
                let alloc_obj: *mut AnyObject = msg_send![cls, alloc];
                let obj: *mut AnyObject = msg_send![alloc_obj, initWithFrame:bounds configuration:&*config];
                Retained::retain(obj).unwrap()
            };

            let delegate = OAuthNavDelegate::new(mtm, shared_clone);

            unsafe {
                let delegate_ref: &AnyObject = &*(delegate.as_ref() as *const NSObject as *const AnyObject);
                let _: () = msg_send![&*web_view, setNavigationDelegate:Some(delegate_ref)];
            }

            let app = UIApplication::sharedApplication(mtm);
            let windows = app.windows();
            if windows.count() > 0 {
                let window: &UIWindow = &windows.objectAtIndex(0);
                if let Some(root_vc) = window.rootViewController() {
                    let web_vc: Retained<UIViewController> = unsafe {
                        UIViewController::new(mtm)
                    };
                    if let Some(vc_view) = unsafe { web_vc.view() } {
                        unsafe {
                            let web_view_any: &AnyObject = &*web_view;
                            let _: () = msg_send![&*vc_view, addSubview:web_view_any];
                            let _: () = msg_send![&*vc_view, setFrame:bounds];
                        }
                        web_vc.setModalPresentationStyle(objc2_ui_kit::UIModalPresentationStyle::FullScreen);
                    }

                    *delegate.ivars().presented_vc.lock().unwrap() = Some(web_vc.clone());

                    unsafe {
                        let _: () = msg_send![&*root_vc, presentViewController:&*web_vc animated:true completion:null_block()];
                    }
                }
            }

            if let Some(ns_url) = NSURL::URLWithString(&NSString::from_str(&login_url)) {
                let request = NSURLRequest::requestWithURL(&ns_url);
                unsafe { let _: () = msg_send![&*web_view, loadRequest:&*request]; };
            }

            LIVE_OBJECTS.with(|s| {
                *s.borrow_mut() = Some((web_view, delegate));
            });
        });

        let (lock, cvar) = &*shared;
        let mut result = lock.lock().unwrap();

        let deadline = std::time::Instant::now() + Duration::from_secs(120);
        while result.is_none() {
            let now = std::time::Instant::now();
            if now >= deadline {
                return Err("OAuth timeout".into());
            }
            let timeout = deadline.duration_since(now).min(Duration::from_millis(500));
            result = cvar.wait_timeout(result, timeout).unwrap().0;
        }

        let oauth_result = result.take().unwrap_or(OAuthResult::Err("No result".into()));
        match oauth_result {
            OAuthResult::Ok(data) if data.starts_with("token:") => {
                let rest = &data[6..];
                let parts: Vec<&str> = rest.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Ok(Token {
                        access_token: parts[0].to_string(),
                        refresh_token: parts[1].to_string(),
                        expires_at: None,
                    })
                } else {
                    Err("Invalid token format".into())
                }
            }
            OAuthResult::Ok(uuid) => complete_login(&uuid),
            OAuthResult::Err(e) => Err(e),
        }
    }

    thread_local! {
        static LIVE_OBJECTS: std::cell::RefCell<Option<(
            Retained<AnyObject>,
            Retained<OAuthNavDelegate>,
        )>> = std::cell::RefCell::new(None);
    }

    fn complete_login(uuid: &str) -> Result<Token, String> {
        let client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1")
            .timeout(std::time::Duration::from_secs(15))
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Failed to create client: {e}"))?;

        let data_url = format!("{}/api/v1/admin/auth/data_for_login/{}", BASE_URL, uuid);
        let data_resp = client.get(&data_url).send()
            .map_err(|e| format!("data_for_login failed: {e}"))?;
        let data_body = data_resp.text()
            .map_err(|e| format!("Failed to read response: {e}"))?;

        let profile_data: serde_json::Value = serde_json::from_str(&data_body)
            .map_err(|e| format!("Failed to parse JSON: {e}"))?;

        let profile_id = profile_data["profile_id"].as_str()
            .ok_or("No profile_id")?;
        let schools = profile_data["schools"].as_array()
            .ok_or("No schools")?;
        let school_id = schools.first()
            .and_then(|s| s.get("id"))
            .and_then(|id| id.as_str())
            .ok_or("No school_id")?;
        let kinds = profile_data["kinds"].as_array()
            .ok_or("No kinds")?;
        let kind = kinds.first()
            .and_then(|k| k.as_str())
            .unwrap_or("student")
            .to_lowercase();

        let token_raw = format!("{}:{}:{}", profile_id, school_id, kind);
        let token_b64 = BASE64.encode(token_raw.as_bytes());
        let login_url = format!("{}/api/v1/auth/login?token={}", BASE_URL, token_b64);

        let auth_resp = client.get(&login_url).send()
            .map_err(|e| format!("Token request failed: {e}"))?;
        let auth_data: AuthResponse = auth_resp.json()
            .map_err(|e| format!("Failed to parse token response: {e}"))?;

        Ok(Token {
            access_token: auth_data.auth_token,
            refresh_token: auth_data.refresh_token,
            expires_at: None,
        })
    }

    fn extract_uuid_from_url(url: &str) -> Option<String> {
        let re = regex::Regex::new(r"[?&]data=([a-f0-9-]+)").ok()?;
        let caps = re.captures(url)?;
        Some(caps.get(1)?.as_str().to_string())
    }
}

#[cfg(not(target_os = "ios"))]
pub(crate) fn login_with_web_view(
    _auth: &super::auth::Auth,
    _username: &str,
    _password: &str,
) -> Result<super::auth::Token, String> {
    Err("Web-based OAuth is only available on iOS".into())
}

#[cfg(target_os = "ios")]
pub(crate) use ios_impl::login_with_web_view;
