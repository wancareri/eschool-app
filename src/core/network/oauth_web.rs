//! iOS OAuth — pure HTTP flow matching the TypeScript reference.
//!
//! Flow (mirrors `scripts/eschool-auth.ts` exactly):
//! 1. GET  diary login/student  → 302 to OAuth login page
//! 2. GET  OAuth login page      → extract CSRF + ReturnUrl from form
//! 3. POST Account/Login         → 302 to authorize/callback
//! 4. GET  callback              → 302 to diary callback
//! 5. GET  diary callback        → 302 to preauthorized?data=UUID
//! 6. GET  data_for_login/{UUID} → profile JSON
//! 7. GET  auth/login?token=…    → JWT tokens

#[cfg(target_os = "ios")]
mod ios_impl {
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    use regex::Regex;

    use crate::core::network::auth::{Token, AuthResponse, BASE_URL};
    use crate::core::nslog;

    const OAUTH_URL: &str = "https://oauth.rios.unibel.by";
    const UA: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";

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

    fn decode_entities(s: &str) -> String {
        s.replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
    }

    fn extract_csrf(html: &str) -> Option<String> {
        let re = Regex::new(r#"name="__RequestVerificationToken"[^>]*value="([^"]+)""#).ok()?;
        let caps = re.captures(html)?;
        Some(caps.get(1)?.as_str().to_string())
    }

    fn extract_return_url(html: &str) -> Option<String> {
        let re = Regex::new(r#"name="Input\.ReturnUrl"\s+value="([^"]+)""#).ok()?;
        let caps = re.captures(html)?;
        Some(decode_entities(caps.get(1)?.as_str()))
    }

    fn extract_uuid(url: &str) -> Option<String> {
        let re = Regex::new(r"preauthorized\?data=([^&]+)").ok()?;
        let caps = re.captures(url)?;
        Some(caps.get(1)?.as_str().to_string())
    }

    pub fn login_with_web_view(
        _auth: &crate::core::network::auth::Auth,
        username: &str,
        password: &str,
    ) -> Result<Token, String> {
        nslog::nslog("[OAuth] Creating HTTP client");
        let client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(UA)
            .timeout(std::time::Duration::from_secs(15))
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Client build: {e}"))?;

        // ── Step 1: GET diary login/student → 302 to OAuth ──────────────
        nslog::nslog("[OAuth] Step 1: GET diary login/student");
        let resp1 = client
            .get(format!("{}/api/v1/admin/auth/login/student", BASE_URL))
            .send()
            .map_err(|e| format!("Step 1: {e}"))?;
        let oauth_login_url = resp1
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or("Step 1: no Location header")?
            .to_string();
        nslog::nslog(&format!("[OAuth] Step 1: redirect → {}", &oauth_login_url[..oauth_login_url.len().min(120)]));

        // ── Step 2: GET OAuth login page → extract CSRF + ReturnUrl ─────
        nslog::nslog("[OAuth] Step 2: GET OAuth login page");
        let login_page_url = if oauth_login_url.starts_with("http") {
            oauth_login_url
        } else {
            format!("{}{}", OAUTH_URL, oauth_login_url)
        };
        let resp2 = client
            .get(&login_page_url)
            .send()
            .map_err(|e| format!("Step 2: {e}"))?;
        let login_html = resp2.text().map_err(|e| format!("Step 2 text: {e}"))?;

        let csrf = extract_csrf(&login_html)
            .ok_or_else(|| {
                let snippet: String = login_html.chars().take(500).collect();
                nslog::nslog(&format!("[OAuth] Step 2: CSRF not found. HTML: {}", snippet));
                "CSRF token not found".to_string()
            })?;
        let return_url = extract_return_url(&login_html)
            .ok_or_else(|| {
                let snippet: String = login_html.chars().take(500).collect();
                nslog::nslog(&format!("[OAuth] Step 2: ReturnUrl not found. HTML: {}", snippet));
                "ReturnUrl not found".to_string()
            })?;
        nslog::nslog(&format!("[OAuth] Step 2: CSRF={}… ReturnUrl={}…", &csrf[..csrf.len().min(30)], &return_url[..return_url.len().min(80)]));

        // ── Step 3: POST credentials → 302 to callback ──────────────────
        nslog::nslog("[OAuth] Step 3: POST Account/Login");
        let oauth_origin = {
            let scheme_end = login_page_url.find("://").unwrap_or(0) + 3;
            let host_end = login_page_url[scheme_end..]
                .find('/')
                .map(|i| scheme_end + i)
                .unwrap_or(login_page_url.len());
            login_page_url[..host_end].to_string()
        };

        let params = [
            ("Input.ReturnUrl", return_url.as_str()),
            ("Input.Username", username),
            ("Input.Password", password),
            ("Input.Button", "login"),
            ("Input.RememberLogin", "false"),
            ("__RequestVerificationToken", csrf.as_str()),
        ];

        let resp3 = client
            .post(format!("{}/Account/Login", oauth_origin))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .map_err(|e| format!("Step 3: {e}"))?;

        nslog::nslog(&format!("[OAuth] Step 3: status={}", resp3.status()));

        let callback_raw = resp3
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or("Step 3: no Location header")?
            .to_string();
        let callback_url = decode_entities(&callback_raw);

        // ── Step 4: GET callback → 302 to diary callback ─────────────────
        nslog::nslog(&format!("[OAuth] Step 4: GET callback {}", &callback_url[..callback_url.len().min(120)]));
        let resp4_url = if callback_url.starts_with("http") {
            callback_url
        } else {
            format!("{}{}", oauth_origin, callback_url)
        };
        let resp4 = client
            .get(&resp4_url)
            .header("Referer", format!("{}/", oauth_origin))
            .send()
            .map_err(|e| format!("Step 4: {e}"))?;

        nslog::nslog(&format!("[OAuth] Step 4: status={}", resp4.status()));

        let diary_callback_raw = resp4
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or("Step 4: no Location header (expected diary callback)")?
            .to_string();
        let diary_callback_url = decode_entities(&diary_callback_raw);

        // ── Step 5: GET diary callback → 302 to preauthorized?data=UUID ─
        nslog::nslog(&format!("[OAuth] Step 5: GET diary callback {}", &diary_callback_url[..diary_callback_url.len().min(120)]));
        let resp5_url = if diary_callback_url.starts_with("http") {
            diary_callback_url
        } else {
            format!("{}{}", BASE_URL, diary_callback_url)
        };
        let resp5 = client
            .get(&resp5_url)
            .send()
            .map_err(|e| format!("Step 5: {e}"))?;

        nslog::nslog(&format!("[OAuth] Step 5: status={}", resp5.status()));

        let preauth_redirect_raw = resp5
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or("Step 5: no Location header (expected preauthorized)")?
            .to_string();
        let preauth_redirect = decode_entities(&preauth_redirect_raw);

        nslog::nslog(&format!("[OAuth] Step 5: preauth redirect → {}", &preauth_redirect[..preauth_redirect.len().min(120)]));

        let uuid = extract_uuid(&preauth_redirect)
            .ok_or_else(|| format!("UUID not found in preauth redirect: {}", &preauth_redirect))?;
        nslog::nslog(&format!("[OAuth] Step 5: UUID = {}", uuid));

        // ── Step 6: GET data_for_login/{UUID} ───────────────────────────
        nslog::nslog(&format!("[OAuth] Step 6: GET data_for_login/{}", uuid));
        let data_url = format!("{}/api/v1/admin/auth/data_for_login/{}", BASE_URL, uuid);
        let resp6 = client
            .get(&data_url)
            .send()
            .map_err(|e| format!("Step 6: {e}"))?;
        let login_data: serde_json::Value = resp6
            .json()
            .map_err(|e| format!("Step 6 parse: {e}"))?;

        let profile_id = login_data["profile_id"].as_str().ok_or("Step 6: no profile_id")?;
        let school_id = login_data["schools"].as_array()
            .and_then(|s| s.first())
            .and_then(|s| s.get("id"))
            .and_then(|id| id.as_str())
            .ok_or("Step 6: no school_id")?;
        let kind = login_data["kinds"].as_array()
            .and_then(|k| k.first())
            .and_then(|k| k.as_str())
            .unwrap_or("student")
            .to_lowercase();

        nslog::nslog(&format!("[OAuth] Step 6: profile={} school={} kind={}", profile_id, school_id, kind));

        // ── Step 7: GET auth/login?token={base64} ───────────────────────
        let token_payload = format!("{}:{}:{}", profile_id, school_id, kind);
        let token_b64 = BASE64.encode(token_payload.as_bytes());
        nslog::nslog("[OAuth] Step 7: GET auth/login?token=…");
        let resp7 = client
            .get(format!("{}/api/v1/auth/login?token={}", BASE_URL, token_b64))
            .send()
            .map_err(|e| format!("Step 7: {e}"))?;

        let auth_data: AuthResponse = resp7
            .json()
            .map_err(|e| format!("Step 7 parse: {e}"))?;

        nslog::nslog(&format!("[OAuth] Done! auth_token={}…", &auth_data.auth_token[..auth_data.auth_token.len().min(40)]));

        Ok(Token {
            access_token: auth_data.auth_token,
            refresh_token: auth_data.refresh_token,
            expires_at: None,
        })
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
