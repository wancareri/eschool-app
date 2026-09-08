use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sha2::{Sha256, Digest};
use rand::Rng;
use regex::Regex;

const CLIENT_ID: &str = "oauth_diary_echools";
const REDIRECT_URI: &str = "https://diary.e-schools.by/api/v1/admin/auth/callback";
const SCOPE: &str = "openid profile offline_access organization.write person.write person.write.all person.read persons.read dictionaries.read organization.read";
const OAUTH_URL: &str = "https://oauth.rios.unibel.by";
const BASE_URL: &str = "https://diary.e-schools.by";
const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
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
        // 1. GET login page to get CSRF token and session cookie
        let client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("Failed to create client: {e}"))?;

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
        // HTML pattern: <input name="__RequestVerificationToken" type="hidden" value="CfDJ8..." />
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
            ("Input.RememberLogin", "true"),
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

        // 5. Follow redirect chain to get auth code
        let client_no_redirect = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("Failed to create client: {e}"))?;

        let mut current_url = full_callback_url;
        let mut auth_code = None;

        for _ in 0..10 {
            let resp = client_no_redirect
                .get(&current_url)
                .send()
                .map_err(|e| format!("Redirect request failed: {e}"))?;

            if resp.status().is_redirection() {
                let location = resp
                    .headers()
                    .get("location")
                    .and_then(|v| v.to_str().ok())
                    .ok_or("Redirect without Location header")?;

                current_url = resolve_url(&current_url, location);

                if let Some(code) = extract_code_from_url(&current_url) {
                    auth_code = Some(code);
                    break;
                }
            } else {
                let body = resp.text().unwrap_or_default();
                if let Some(code) = extract_code_from_url(&body) {
                    auth_code = Some(code);
                    break;
                }
                break;
            }
        }

        let code = auth_code.ok_or("No authorization code found in redirect chain")?;

        // 6. Exchange code for JWT via diary.e-schools.by
        let token_url = format!(
            "{}/api/v1/admin/auth/callback?code={}&state={}",
            BASE_URL, code, self.state
        );

        let mut current = token_url;
        let mut token_url_final = String::new();

        for _ in 0..10 {
            let resp = client_no_redirect
                .get(&current)
                .send()
                .map_err(|e| format!("Token redirect failed: {e}"))?;

            if resp.status().is_redirection() {
                let location = resp
                    .headers()
                    .get("location")
                    .and_then(|v| v.to_str().ok())
                    .ok_or("Redirect without Location")?;

                current = resolve_url(&current, location);

                if current.contains("/api/v1/auth/login") {
                    token_url_final = current;
                    break;
                }
            } else {
                break;
            }
        }

        if token_url_final.is_empty() {
            return Err("Failed to reach token endpoint".into());
        }

        // 7. GET JWT tokens
        let auth_resp = client_no_redirect
            .get(&token_url_final)
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
