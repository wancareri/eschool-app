use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sha2::{Sha256, Digest};
use rand::Rng;
use regex::Regex;

const CLIENT_ID: &str = "oauth_diary_echools";
const REDIRECT_URI: &str = "https://diary.e-schools.by/api/v1/admin/auth/callback";
const SCOPE: &str = "openid profile offline_access organization.write person.write person.write.all person.read persons.read dictionaries.read organization.read";
const OAUTH_URL: &str = "https://oauth.rios.unibel.by";
pub(crate) const BASE_URL: &str = "https://diary.e-schools.by";
const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AuthResponse {
    pub auth_token: String,
    pub refresh_token: String,
}

pub struct Auth {
    code_verifier: String,
    code_challenge: String,
    state: String,
}

impl Auth {
    pub fn new() -> Self {
        let code_verifier = generate_code_verifier();
        let code_challenge = generate_code_challenge(&code_verifier);
        let state = "schools".to_string();

        Self {
            code_verifier,
            code_challenge,
            state,
        }
    }

    pub fn login_url(&self) -> String {
        let return_url = self.build_return_url();
        format!(
            "{}/Account/Login?ReturnUrl={}",
            OAUTH_URL,
            urlencoding::encode(&return_url),
        )
    }

    fn build_return_url(&self) -> String {
        format!(
            "/connect/authorize/callback?client_id={}&response_type=code&state={}&authentication=client_secret_post&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256",
            CLIENT_ID,
            &self.state,
            urlencoding::encode(REDIRECT_URI),
            urlencoding::encode(SCOPE),
            &self.code_challenge,
        )
    }

    pub fn code_verifier(&self) -> &str {
        &self.code_verifier
    }

    /// Login with username and password.
    pub fn login_blocking(&self, username: &str, password: &str) -> Result<Token, String> {
        // Single client for entire flow — cookies must be preserved
        let client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(15))
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Failed to create client: {e}"))?;

        // 0. Hit diary login/student to establish session context
        let init_resp = client
            .get(format!("{}/api/v1/admin/auth/login/student", BASE_URL))
            .send()
            .map_err(|e| format!("Failed to init login: {e}"))?;
        let _init_status = init_resp.status(); // 302 expected, ignore errors

        // 1. GET OAuth login page
        let login_url = self.login_url();
        let login_page = client
            .get(&login_url)
            .send()
            .map_err(|e| format!("Failed to load login page: {e}"))?;

        let status = login_page.status();
        if !status.is_success() {
            return Err(format!("Login page returned HTTP {status}"));
        }

        let page_html = login_page
            .text()
            .map_err(|e| format!("Failed to read login page: {e}"))?;

        // 2. Extract __RequestVerificationToken
        let csrf_token = extract_csrf_token(&page_html)
            .ok_or_else(|| {
                let snippet: String = page_html.chars().take(300).collect();
                format!("CSRF token not found. Page: {snippet}")
            })?;

        // 3. POST credentials
        let return_url = self.build_return_url();

        let params = [
            ("Input.ReturnUrl", return_url.as_str()),
            ("Input.Username", username),
            ("Input.Password", password),
            ("Input.Button", "login"),
            ("__RequestVerificationToken", csrf_token.as_str()),
            ("Input.RememberLogin", "false"),
        ];

        let response = client
            .post(format!("{}/Account/Login", OAUTH_URL))
            .form(&params)
            .send()
            .map_err(|e| format!("Login POST failed: {e}"))?;

        // 4. Check redirect
        let status = response.status();
        if !status.is_redirection() {
            let body = response.text().unwrap_or_default();
            let snippet: String = body.chars().take(300).collect();
            return Err(format!("Expected redirect, got HTTP {status}. Body: {snippet}"));
        }

        let callback_url = response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or("No Location header in redirect")?;

        let full_callback_url = if callback_url.starts_with("http") {
            callback_url.to_string()
        } else {
            format!("{}{}", OAUTH_URL, callback_url)
        };

        // 5. Follow redirect chain from OAuth to diary callback
        let mut current_url = full_callback_url;
        let mut preauth_url = None;

        for _step in 0..15 {
            let resp = client
                .get(&current_url)
                .header("Referer", format!("{}/", OAUTH_URL))
                .send()
                .map_err(|e| format!("Redirect request failed: {e}"))?;

            if resp.status().is_redirection() {
                let location = resp
                    .headers()
                    .get("location")
                    .and_then(|v| v.to_str().ok())
                    .ok_or("Redirect without Location header")?;

                current_url = resolve_url(&current_url, location);

                if current_url.contains("login/error") {
                    // Extract error message from base64 data param
                    let error_msg = extract_uuid_from_url(&current_url)
                        .and_then(|data| base64::engine::general_purpose::STANDARD.decode(format!("{}==", data.replace('/', "/").replace('-', "+")).as_bytes()).ok())
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                        .unwrap_or_else(|| "Unknown error".to_string());
                    return Err(format!("Login failed: {error_msg}"));
                }

                if current_url.contains("data_for_login") || current_url.contains("preauthorized") {
                    preauth_url = Some(current_url.clone());
                    break;
                }
            } else {
                let body = resp.text().unwrap_or_default();
                if let Some(url) = find_data_for_login_url(&body) {
                    preauth_url = Some(url);
                    break;
                }
                break;
            }
        }

        // If we hit a preauthorized page, extract the uuid and call data_for_login
        let data_url = if let Some(url) = preauth_url {
            if url.contains("data_for_login") {
                url
            } else if let Some(uuid) = extract_uuid_from_url(&url) {
                format!("{}/api/v1/admin/auth/data_for_login/{}", BASE_URL, uuid)
            } else {
                return Err(format!("Preauthorized URL has no UUID: {}", url).into());
            }
        } else {
            return Err("No preauthorized redirect received from callback".into());
        };

        // 6. Get profile data from data_for_login
        let data_resp = client
            .get(&data_url)
            .send()
            .map_err(|e| format!("data_for_login request failed: {e}"))?;

        let data_body = data_resp
            .text()
            .map_err(|e| format!("Failed to read data_for_login response: {e}"))?;

        let profile_data: serde_json::Value = serde_json::from_str(&data_body)
            .map_err(|e| format!("Failed to parse data_for_login JSON: {e}"))?;

        let profile_id = profile_data["profile_id"]
            .as_str()
            .ok_or("No profile_id in data_for_login response")?;
        let schools = profile_data["schools"]
            .as_array()
            .ok_or("No schools in data_for_login response")?;
        let school_id = schools.first()
            .and_then(|s| s.get("id"))
            .and_then(|id| id.as_str())
            .ok_or("No school_id in data_for_login response")?;
        let kinds = profile_data["kinds"]
            .as_array()
            .ok_or("No kinds in data_for_login response")?;
        let kind = kinds.first()
            .and_then(|k| k.as_str())
            .unwrap_or("student")
            .to_lowercase();

        // 7. Build token string: profile_id:school_id:kind (base64 encoded)
        //    Matches the web SPA: btoa(profile_id + ":" + school_id + ":" + kind)
        let token_raw = format!("{}:{}:{}", profile_id, school_id, kind);
        let token_b64 = BASE64.encode(token_raw.as_bytes());

        let login_url = format!("{}/api/v1/auth/login?token={}", BASE_URL, token_b64);

        // 8. Get JWT tokens
        let auth_resp = client
            .get(&login_url)
            .send()
            .map_err(|e| format!("Token request failed: {e}"))?;

        let auth_data: AuthResponse = auth_resp
            .json()
            .map_err(|e| format!("Failed to parse token response: {e}"))?;

        Ok(Token {
            access_token: auth_data.auth_token,
            refresh_token: auth_data.refresh_token,
            expires_at: None,
        })
    }
}

fn resolve_url(base: &str, relative: &str) -> String {
    if relative.starts_with("http") {
        relative.to_string()
    } else if relative.starts_with('/') {
        let scheme_end = base.find("://").unwrap_or(0) + 3;
        let base_host_end = base[scheme_end..]
            .find('/')
            .map(|i| scheme_end + i)
            .unwrap_or(base.len());
        format!("{}{}", &base[..base_host_end], relative)
    } else {
        let base_path = if let Some(pos) = base.rfind('/') {
            &base[..=pos]
        } else {
            base
        };
        format!("{}{}", base_path, relative)
    }
}

/// Extract __RequestVerificationToken from ASP.NET Core HTML form.
/// Actual HTML: <input name="__RequestVerificationToken" type="hidden" value="CfDJ8..." />
fn extract_csrf_token(html: &str) -> Option<String> {
    // The exact pattern from the OAuth server:
    // <input name="__RequestVerificationToken" type="hidden" value="CfDJ8..." />
    let re = Regex::new(r#"name="__RequestVerificationToken"[^>]*value="([^"]+)""#).ok()?;
    let caps = re.captures(html)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn extract_code_from_url(url: &str) -> Option<String> {
    let re = Regex::new(r"[?&]code=([^&]+)").ok()?;
    let caps = re.captures(url)?;
    Some(urlencoding::decode(caps.get(1)?.as_str()).ok()?.into_owned())
}

fn find_data_for_login_url(body: &str) -> Option<String> {
    let re = Regex::new(r#"/api/v1/admin/auth/data_for_login/([a-f0-9-]+)"#).ok()?;
    let caps = re.captures(body)?;
    let uuid = caps.get(1)?.as_str();
    Some(format!("{}/api/v1/admin/auth/data_for_login/{}", BASE_URL, uuid))
}

fn extract_uuid_from_url(url: &str) -> Option<String> {
    let re = Regex::new(r"[?&]data=([a-f0-9-]+)").ok()?;
    let caps = re.captures(url)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn generate_code_verifier() -> String {
    let mut rng = rand::thread_rng();
    (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            match idx {
                0..10 => (b'0' + idx) as char,
                10..36 => (b'a' + idx - 10) as char,
                36..62 => (b'A' + idx - 36) as char,
                _ => unreachable!(),
            }
        })
        .collect()
}

fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    BASE64.encode(result)
        .replace('+', "-")
        .replace('/', "_")
        .replace('=', "")
}
