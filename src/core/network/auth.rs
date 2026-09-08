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

    /// Build the OAuth authorization URL for browser-based login.
    pub fn login_url(&self) -> String {
        let return_url = format!(
            "/connect/authorize/callback?client_id={}&response_type=code&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256&state={}",
            CLIENT_ID,
            urlencoding::encode(REDIRECT_URI),
            urlencoding::encode(SCOPE),
            &self.code_challenge,
            &self.state,
        );
        format!(
            "{}/Account/Login?ReturnUrl={}",
            OAUTH_URL,
            urlencoding::encode(&return_url),
        )
    }

    pub fn code_verifier(&self) -> &str {
        &self.code_verifier
    }

    /// Login with username and password using blocking HTTP client.
    /// Returns JWT tokens on success.
    pub fn login_blocking(&self, username: &str, password: &str) -> Result<Token, String> {
        // 1. GET login page to get CSRF token and session cookie
        let client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("Failed to create client: {e}"))?;

        let login_url = self.login_url();
        let login_page = client
            .get(&login_url)
            .send()
            .map_err(|e| format!("Failed to load login page: {e}"))?;

        let page_html = login_page
            .text()
            .map_err(|e| format!("Failed to read login page: {e}"))?;

        // 2. Extract __RequestVerificationToken from HTML
        let csrf_token = extract_csrf_token(&page_html)
            .ok_or("Failed to extract CSRF token from login page")?;

        // 3. POST credentials
        let return_url = format!(
            "/connect/authorize/callback?client_id={}&response_type=code&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256&state={}",
            CLIENT_ID,
            urlencoding::encode(REDIRECT_URI),
            urlencoding::encode(SCOPE),
            &self.code_challenge,
            &self.state,
        );

        let params = [
            ("Input.Username", username),
            ("Input.Password", password),
            ("Input.ReturnUrl", &return_url),
            ("Input.Button", "login"),
            ("__RequestVerificationToken", &csrf_token),
            ("Input.RememberLogin", "true"),
        ];

        let response = client
            .post(format!("{}/Account/Login", OAUTH_URL))
            .form(&params)
            .send()
            .map_err(|e| format!("Login request failed: {e}"))?;

        // 4. Follow redirect to get auth code
        // The POST returns 302 with Location header pointing to callback URL
        let callback_url = response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or("No redirect after login — check credentials")?;

        // The callback URL might be relative or absolute
        let full_callback_url = if callback_url.starts_with("http") {
            callback_url.to_string()
        } else {
            format!("{}{}", OAUTH_URL, callback_url)
        };

        // 5. Follow the callback redirect chain to diary.e-schools.by
        // Disable automatic redirect to capture each step
        let client_no_redirect = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
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

            let status = resp.status();
            if status.is_redirection() {
                let location = resp
                    .headers()
                    .get("location")
                    .and_then(|v| v.to_str().ok())
                    .ok_or("Redirect without Location header")?;

                current_url = if location.starts_with("http") {
                    location.to_string()
                } else if location.starts_with('/') {
                    // Extract base URL from current_url
                    let base = current_url
                        .split("://")
                        .nth(1)
                        .and_then(|s| s.find('/'))
                        .map(|i| &current_url[..current_url.find("://").unwrap() + 3 + i])
                        .unwrap_or(&current_url);
                    format!("{}{}", base, location)
                } else {
                    format!("{}/{}", current_url.trim_end_matches('/'), location)
                };

                // Check if this is the callback URL with a code
                if current_url.contains("code=") {
                    if let Some(code) = extract_code_from_url(&current_url) {
                        auth_code = Some(code);
                        break;
                    }
                }
            } else {
                // Check response body for code
                let body = resp.text().unwrap_or_default();
                if let Some(code) = extract_code_from_url(&body) {
                    auth_code = Some(code);
                    break;
                }
                break;
            }
        }

        let code = auth_code.ok_or("Failed to get authorization code from redirect")?;

        // 6. Exchange authorization code for tokens via diary.e-schools.by
        // The callback redirects to: /api/v1/auth/login?token=<base64>
        // But we need to construct the base64 token ourselves
        // Actually, let's just follow the redirect chain and get the final JWT

        // First, hit the callback URL with the code
        let token_url = format!(
            "{}/api/v1/admin/auth/callback?code={}&state={}",
            BASE_URL, code, self.state
        );

        let resp = client_no_redirect
            .get(&token_url)
            .send()
            .map_err(|e| format!("Token exchange failed: {e}"))?;

        // Follow redirects to get to /api/v1/auth/login?token=...
        let mut token_url_final = String::new();
        let mut current = token_url;

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

                current = if location.starts_with("http") {
                    location.to_string()
                } else if location.starts_with('/') {
                    let base = current
                        .split("://")
                        .nth(1)
                        .and_then(|s| s.find('/'))
                        .map(|i| &current[..current.find("://").unwrap() + 3 + i])
                        .unwrap_or(&current);
                    format!("{}{}", base, location)
                } else {
                    format!("{}/{}", current.trim_end_matches('/'), location)
                };

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

        // 7. GET the token endpoint to receive JWT
        let auth_resp = client_no_redirect
            .get(&token_url_final)
            .send()
            .map_err(|e| format!("Final token request failed: {e}"))?;

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

/// Extract __RequestVerificationToken from HTML form.
fn extract_csrf_token(html: &str) -> Option<String> {
    let re = Regex::new(r#"name="__RequestVerificationToken"\s+value="([^"]+)""#).ok()?;
    let caps = re.captures(html)?;
    Some(caps.get(1)?.as_str().to_string())
}

/// Extract authorization code from URL query parameter.
fn extract_code_from_url(url: &str) -> Option<String> {
    let re = Regex::new(r"[?&]code=([^&]+)").ok()?;
    let caps = re.captures(url)?;
    Some(urlencoding::decode(&caps.get(1)?.as_str().to_string()).ok()?.into_owned())
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
