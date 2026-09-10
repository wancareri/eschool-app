//! iOS-only: programmatic OAuth + hidden WKWebView for callback.
//!
//! Flow:
//! 1. Programmatic HTTP to get authorization code from oauth.rios.unibel.by
//! 2. Hidden WKWebView loads diary callback URL (bypasses TLS fingerprint)
//! 3. Extract preauth UUID from WKWebView redirect
//! 4. Programmatic HTTP to exchange UUID for JWT tokens
//!
//! The user sees only the native login form — no web page is ever shown.

#[cfg(target_os = "ios")]
mod ios_impl {
    use std::sync::{Arc, Condvar, Mutex};
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, ProtocolObject};
    use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly, DefinedClass};
    use objc2_foundation::{NSObject, NSString, NSURL, NSURLRequest};
    use objc2_core_foundation::CGRect;
    use objc2_ui_kit::{UIView, UIViewController, UIApplication, UIWindow};
    use day::prelude::*;
    use regex::Regex;

    use super::super::auth::{Auth, Token, AuthResponse, BASE_URL};

    const OAUTH_URL: &str = "https://oauth.rios.unibel.by";
    const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";

    // ── Cross-thread result ──────────────────────────────────────────────

    enum CallbackResult {
        Ok(String),
        Err(String),
    }

    type SharedState = Arc<(Mutex<Option<CallbackResult>>, Condvar)>;

    // ── Hidden WKWebView navigation delegate ──────────────────────────────

    struct DelegateIvars {
        shared: SharedState,
        presented_vc: Mutex<Option<Retained<UIViewController>>>,
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "DayHiddenNavDelegate"]
        #[ivars = DelegateIvars]
        struct HiddenNavDelegate;

        impl HiddenNavDelegate {
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

                log::info!("[HiddenWV] didFinish: {}", &url_str[..url_str.len().min(200)]);

                if let Some(uuid) = extract_uuid_from_url(&url_str) {
                    log::info!("[HiddenWV] Found UUID: {}", uuid);
                    self.signal(CallbackResult::Ok(uuid));
                    self.cleanup(web_view);
                    return;
                }

                if url_str.contains("login/error") {
                    log::error!("[HiddenWV] Login error");
                    self.signal(CallbackResult::Err("Login failed".into()));
                    self.cleanup(web_view);
                    return;
                }

                if url_str.contains("diary.e-schools.by") {
                    log::info!("[HiddenWV] On diary, reading localStorage...");
                    self.read_local_storage(web_view);
                }
            }

            #[unsafe(method(webView:didFailNavigation:withError:))]
            fn did_fail_navigation(
                &self,
                web_view: &AnyObject,
                _navigation: Option<&AnyObject>,
                error: *mut AnyObject,
            ) {
                let msg: String = if error.is_null() {
                    "unknown".into()
                } else {
                    unsafe {
                        let ns_err: &NSString = &*(error as *const _ as *const NSString);
                        ns_err.to_string()
                    }
                };
                log::error!("[HiddenWV] Navigation failed: {}", msg);
                self.signal(CallbackResult::Err(format!("WebView error: {msg}")));
                self.cleanup(web_view);
            }
        }
    );

    impl HiddenNavDelegate {
        fn new(mtm: MainThreadMarker, shared: SharedState) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(DelegateIvars {
                shared,
                presented_vc: Mutex::new(None),
            });
            unsafe { msg_send![super(this), init] }
        }

        fn signal(&self, result: CallbackResult) {
            let ivars = self.ivars();
            let mut guard = ivars.shared.0.lock().unwrap();
            if guard.is_none() {
                *guard = Some(result);
                ivars.shared.1.notify_one();
            }
        }

        fn cleanup(&self, web_view: &AnyObject) {
            unsafe { let _: () = msg_send![web_view, stopLoading]; };
            if let Some(vc) = self.ivars().presented_vc.lock().unwrap().take() {
                unsafe { let _: () = msg_send![&*vc, dismissViewControllerAnimated:false completion:null_block()]; };
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
                    log::info!("[HiddenWV] localStorage: {}...", &s[..s.len().min(100)]);

                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) {
                        let auth_token = val["authToken"].as_str().unwrap_or("");
                        let refresh_token = val["refreshToken"].as_str().unwrap_or("");
                        if !auth_token.is_empty() {
                            let mut guard = shared.0.lock().unwrap();
                            if guard.is_none() {
                                *guard = Some(CallbackResult::Ok(format!(
                                    "token:{}:{}", auth_token, refresh_token
                                )));
                                shared.1.notify_one();
                            }
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

    // ── Step 1: Programmatic HTTP to get auth code ────────────────────────

    fn get_auth_code(auth: &Auth, username: &str, password: &str) -> Result<String, String> {
        let client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(15))
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Failed to create client: {e}"))?;

        // Init session
        let _ = client
            .get(format!("{}/api/v1/admin/auth/login/student", BASE_URL))
            .send();

        // Get login page
        let login_url = auth.login_url();
        let login_page = client
            .get(&login_url)
            .send()
            .map_err(|e| format!("Failed to load login page: {e}"))?;

        if !login_page.status().is_success() {
            return Err(format!("Login page HTTP {}", login_page.status()));
        }

        let html = login_page.text().map_err(|e| e.to_string())?;
        let csrf = extract_csrf(&html)
            .ok_or("CSRF token not found")?;

        // POST credentials
        let return_url = auth.build_return_url();
        let params = [
            ("Input.ReturnUrl", return_url.as_str()),
            ("Input.Username", username),
            ("Input.Password", password),
            ("Input.Button", "login"),
            ("__RequestVerificationToken", csrf.as_str()),
            ("Input.RememberLogin", "false"),
        ];

        let resp = client
            .post(format!("{}/Account/Login", OAUTH_URL))
            .form(&params)
            .send()
            .map_err(|e| format!("Login POST failed: {e}"))?;

        if !resp.status().is_redirection() {
            return Err(format!("Expected redirect, got HTTP {}", resp.status()));
        }

        let location = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or("No Location header")?;

        let callback_url = if location.starts_with("http") {
            location.to_string()
        } else {
            format!("{}{}", OAUTH_URL, location)
        };

        // Follow redirects until we reach the diary callback URL
        let mut current_url = callback_url;
        for _ in 0..15 {
            let resp = client
                .get(&current_url)
                .header("Referer", format!("{}/", OAUTH_URL))
                .send()
                .map_err(|e| format!("Redirect failed: {e}"))?;

            if resp.status().is_redirection() {
                let loc = resp
                    .headers()
                    .get("location")
                    .and_then(|v| v.to_str().ok())
                    .ok_or("Redirect without Location")?;

                current_url = resolve_url(&current_url, loc);

                if current_url.contains("login/error") {
                    return Err("Login failed (wrong credentials?)".into());
                }

                if current_url.contains("diary.e-schools.by") && current_url.contains("code=") {
                    log::info!("[HTTP] Reached diary callback: {}", &current_url[..current_url.len().min(200)]);
                    return Ok(current_url);
                }
            } else {
                let status = resp.status();
                let body = resp.text().unwrap_or_default();
                if let Some(url) = find_callback_url(&body) {
                    return Ok(url);
                }
                return Err(format!("Unexpected response HTTP {}", status));
            }
        }

        Err("Too many redirects".into())
    }

    // ── Step 3: Token exchange (after UUID extracted) ─────────────────────

    fn complete_login(uuid: &str) -> Result<Token, String> {
        let client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(15))
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Failed to create client: {e}"))?;

        let data_url = format!("{}/api/v1/admin/auth/data_for_login/{}", BASE_URL, uuid);
        let data_resp = client.get(&data_url).send()
            .map_err(|e| format!("data_for_login failed: {e}"))?;
        let data_body = data_resp.text().map_err(|e| e.to_string())?;

        let profile_data: serde_json::Value = serde_json::from_str(&data_body)
            .map_err(|e| format!("Parse error: {e}"))?;

        let profile_id = profile_data["profile_id"].as_str()
            .ok_or("No profile_id")?;
        let school_id = profile_data["schools"].as_array()
            .and_then(|s| s.first())
            .and_then(|s| s.get("id"))
            .and_then(|id| id.as_str())
            .ok_or("No school_id")?;
        let kind = profile_data["kinds"].as_array()
            .and_then(|k| k.first())
            .and_then(|k| k.as_str())
            .unwrap_or("student")
            .to_lowercase();

        let token_raw = format!("{}:{}:{}", profile_id, school_id, kind);
        let token_b64 = BASE64.encode(token_raw.as_bytes());

        let auth_resp = client
            .get(format!("{}/api/v1/auth/login?token={}", BASE_URL, token_b64))
            .send()
            .map_err(|e| format!("Token request failed: {e}"))?;
        let auth_data: AuthResponse = auth_resp.json()
            .map_err(|e| format!("Token parse error: {e}"))?;

        Ok(Token {
            access_token: auth_data.auth_token,
            refresh_token: auth_data.refresh_token,
            expires_at: None,
        })
    }

    // ── Public API ───────────────────────────────────────────────────────

    pub fn login_with_web_view(auth: &Auth, username: &str, password: &str) -> Result<Token, String> {
        use std::time::Duration;

        // Step 1: Get auth code via programmatic HTTP
        log::info!("[OAuth] Step 1: Getting auth code via HTTP...");
        let callback_url = get_auth_code(auth, username, password)?;
        log::info!("[OAuth] Got callback URL, opening hidden WKWebView...");

        // Step 2: Hidden WKWebView for the callback (bypasses TLS fingerprint)
        let shared: SharedState = Arc::new((Mutex::new(None), Condvar::new()));
        let shared_clone = shared.clone();

        dispatch2::DispatchQueue::main().exec_async(move || {
            let mtm = MainThreadMarker::new().expect("must be on main thread");

            // Create config
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

            // Get screen bounds
            let bounds: CGRect = unsafe {
                let app = UIApplication::sharedApplication(mtm);
                let windows = app.windows();
                let window: &UIWindow = &windows.objectAtIndex(0);
                msg_send![window, bounds]
            };

            // Create WKWebView (hidden)
            let web_view: Retained<AnyObject> = unsafe {
                let cls = objc2::runtime::Class::get(c"WKWebView").unwrap();
                let alloc_obj: *mut AnyObject = msg_send![cls, alloc];
                let obj: *mut AnyObject = msg_send![alloc_obj, initWithFrame:bounds configuration:&*config];
                Retained::retain(obj).unwrap()
            };

            // Make it invisible
            unsafe {
                let _: () = msg_send![&*web_view, setAlpha:0.0f64];
            }

            // Create delegate
            let delegate = HiddenNavDelegate::new(mtm, shared_clone);

            // Set navigation delegate
            unsafe {
                let delegate_ref: &AnyObject = &*(delegate.as_ref() as *const NSObject as *const AnyObject);
                let _: () = msg_send![&*web_view, setNavigationDelegate:Some(delegate_ref)];
            }

            // Add to root view controller (needed for cookie handling)
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
                    }

                    *delegate.ivars().presented_vc.lock().unwrap() = Some(web_vc.clone());

                    // Present silently (no animation)
                    unsafe {
                        let _: () = msg_send![&*root_vc, presentViewController:&*web_vc animated:false completion:null_block()];
                    }
                }
            }

            // Navigate to callback URL
            if let Some(ns_url) = NSURL::URLWithString(&NSString::from_str(&callback_url)) {
                let request = NSURLRequest::requestWithURL(&ns_url);
                unsafe { let _: () = msg_send![&*web_view, loadRequest:&*request]; };
            }

            // Keep alive
            HIDDEN_WV_OBJECTS.with(|s| {
                *s.borrow_mut() = Some((web_view, delegate));
            });
        });

        // Wait for result
        let (lock, cvar) = &*shared;
        let mut result = lock.lock().unwrap();

        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        while result.is_none() {
            let now = std::time::Instant::now();
            if now >= deadline {
                return Err("Timeout waiting for callback".into());
            }
            let timeout = deadline.duration_since(now).min(Duration::from_millis(200));
            result = cvar.wait_timeout(result, timeout).unwrap().0;
        }

        let cb_result = result.take().unwrap_or(CallbackResult::Err("No result".into()));
        match cb_result {
            CallbackResult::Ok(data) if data.starts_with("token:") => {
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
            CallbackResult::Ok(uuid) => {
                log::info!("[OAuth] Step 3: Exchanging UUID for tokens...");
                complete_login(&uuid)
            }
            CallbackResult::Err(e) => Err(e),
        }
    }

    thread_local! {
        static HIDDEN_WV_OBJECTS: std::cell::RefCell<Option<(
            Retained<AnyObject>,
            Retained<HiddenNavDelegate>,
        )>> = std::cell::RefCell::new(None);
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    fn resolve_url(base: &str, relative: &str) -> String {
        if relative.starts_with("http") {
            relative.to_string()
        } else if relative.starts_with('/') {
            let scheme_end = base.find("://").unwrap_or(0) + 3;
            let host_end = base[scheme_end..]
                .find('/')
                .map(|i| scheme_end + i)
                .unwrap_or(base.len());
            format!("{}{}", &base[..host_end], relative)
        } else {
            let base_path = base.rfind('/').map(|i| &base[..=i]).unwrap_or(base);
            format!("{}{}", base_path, relative)
        }
    }

    fn extract_csrf(html: &str) -> Option<String> {
        let re = Regex::new(r#"name="__RequestVerificationToken"[^>]*value="([^"]+)""#).ok()?;
        let caps = re.captures(html)?;
        Some(caps.get(1)?.as_str().to_string())
    }

    fn find_callback_url(body: &str) -> Option<String> {
        let re = Regex::new(r#"href="([^"]*diary\.e-schools\.by/api/v1/admin/auth/callback[^"]*)""#).ok()?;
        let caps = re.captures(body)?;
        Some(caps.get(1)?.as_str().to_string())
    }

    fn extract_uuid_from_url(url: &str) -> Option<String> {
        let re = Regex::new(r"[?&]data=([a-f0-9-]+)").ok()?;
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
