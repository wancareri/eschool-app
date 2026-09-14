//! Authentication — login, logout, token refresh.

use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::shared::api::blocking;
use crate::shared::nslog;

const TOKEN_KEY: &str = "auth.token";
const REFRESH_KEY: &str = "auth.refresh_token";
const SCHOOL_ID_KEY: &str = "auth.school_id";
const PROFILE_ID_KEY: &str = "auth.profile_id";
const CLASS_ID_KEY: &str = "auth.class_id";
const FULL_NAME_KEY: &str = "auth.full_name";
const SCHOOL_NAME_KEY: &str = "auth.school_name";

const CLIENT_ID: &str = "oauth_diary_echools";
const REDIRECT_URI: &str = "https://diary.e-schools.by/api/v1/admin/auth/callback";
const SCOPE: &str = "openid profile offline_access organization.write person.write person.write.all person.read persons.read dictionaries.read organization.read";
const OAUTH_URL: &str = "https://oauth.rios.unibel.by";
const UA: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
    pub auth_token: String,
    pub refresh_token: String,
}

pub struct Auth {
    state: String,
}

impl Auth {
    pub fn new() -> Self {
        Self { state: "schools".to_string() }
    }

    fn build_return_url(&self) -> String {
        format!(
            "/connect/authorize/callback?client_id={}&response_type=code&state={}&authentication=client_secret_post&redirect_uri={}&scope={}",
            CLIENT_ID,
            &self.state,
            urlencoding::encode(REDIRECT_URI),
            urlencoding::encode(SCOPE),
        )
    }

    /// Login with username and password — pure HTTP redirect flow matching iOS.
    pub fn login_blocking(&self, username: &str, password: &str) -> Result<Token, String> {
        let client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(UA)
            .timeout(std::time::Duration::from_secs(15))
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Client build: {e}"))?;

        // Step 1: GET diary login/student → 302 to OAuth
        nslog::nslog("[OAuth] Step 1: GET diary login/student");
        let resp1 = client
            .get(format!("{}/api/v1/admin/auth/login/student", blocking::BASE_URL))
            .send()
            .map_err(|e| format!("Step 1: {e}"))?;
        nslog::nslog(&format!("[OAuth] Step 1: status={}", resp1.status()));
        let oauth_login_url = resp1
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or("Step 1: no Location header")?
            .to_string();

        // Step 2: GET OAuth login page → extract CSRF + ReturnUrl
        let login_page_url = if oauth_login_url.starts_with("http") {
            oauth_login_url
        } else {
            format!("{}{}", OAUTH_URL, oauth_login_url)
        };
        nslog::nslog(&format!("[OAuth] Step 2: GET {}", &login_page_url[..login_page_url.len().min(120)]));
        let resp2 = client
            .get(&login_page_url)
            .send()
            .map_err(|e| format!("Step 2: {e}"))?;
        let login_html = resp2.text().map_err(|e| format!("Step 2 text: {e}"))?;

        let csrf = extract_csrf(&login_html)
            .ok_or_else(|| format!("CSRF not found"))?;
        let return_url = extract_return_url(&login_html)
            .ok_or_else(|| format!("ReturnUrl not found"))?;

        // Step 3: POST credentials → 302 to callback
        let oauth_origin = extract_origin(&login_page_url);
        let params = [
            ("Input.ReturnUrl", return_url.as_str()),
            ("Input.Username", username),
            ("Input.Password", password),
            ("Input.Button", "login"),
            ("Input.RememberLogin", "false"),
            ("__RequestVerificationToken", csrf.as_str()),
        ];

        nslog::nslog(&format!("[OAuth] Step 3: POST {}/Account/Login", oauth_origin));
        let resp3 = client
            .post(format!("{}/Account/Login", oauth_origin))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .map_err(|e| format!("Step 3: {e}"))?;

        let callback_raw = match resp3.headers().get("location").and_then(|v| v.to_str().ok()) {
            Some(loc) => loc.to_string(),
            None => {
                let body = resp3.text().unwrap_or_default();
                return Err(format!("Step 3: no Location. Body: {}", &body[..body.len().min(300)]));
            }
        };
        let callback_url = decode_entities(&callback_raw);

        // Step 4: GET callback → 302 to diary callback
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

        let diary_callback_raw = match resp4.headers().get("location").and_then(|v| v.to_str().ok()) {
            Some(loc) => loc.to_string(),
            None => {
                let body = resp4.text().unwrap_or_default();
                return Err(format!("Step 4: no Location. Body: {}", &body[..body.len().min(300)]));
            }
        };
        let diary_callback_url = decode_entities(&diary_callback_raw);

        // Step 5: GET diary callback → 302 to preauthorized?data=UUID
        let resp5_url = if diary_callback_url.starts_with("http") {
            diary_callback_url
        } else {
            format!("{}{}", blocking::BASE_URL, diary_callback_url)
        };
        let resp5 = client
            .get(&resp5_url)
            .send()
            .map_err(|e| format!("Step 5: {e}"))?;

        let preauth_redirect_raw = match resp5.headers().get("location").and_then(|v| v.to_str().ok()) {
            Some(loc) => loc.to_string(),
            None => {
                let body = resp5.text().unwrap_or_default();
                return Err(format!("Step 5: no Location. Body: {}", &body[..body.len().min(300)]));
            }
        };
        let preauth_redirect = decode_entities(&preauth_redirect_raw);
        let uuid = extract_uuid(&preauth_redirect)
            .ok_or_else(|| format!("UUID not found in preauth redirect"))?;

        // Step 6: GET data_for_login/{UUID}
        let data_url = format!("{}/api/v1/admin/auth/data_for_login/{}", blocking::BASE_URL, uuid);
        nslog::nslog(&format!("[OAuth] Step 6: GET data_for_login/{}", uuid));
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

        // Step 7: GET auth/login?token={base64}
        use base64::Engine;
        let token_payload = format!("{}:{}:{}", profile_id, school_id, kind);
        let token_b64 = base64::engine::general_purpose::STANDARD.encode(token_payload.as_bytes());
        nslog::nslog("[OAuth] Step 7: GET auth/login?token=…");
        let resp7 = client
            .get(format!("{}/api/v1/auth/login?token={}", blocking::BASE_URL, token_b64))
            .send()
            .map_err(|e| format!("Step 7: {e}"))?;

        let auth_data: AuthResponse = resp7
            .json()
            .map_err(|e| format!("Step 7 parse: {e}"))?;

        nslog::nslog("[OAuth] Done!");
        Ok(Token {
            access_token: auth_data.auth_token,
            refresh_token: auth_data.refresh_token,
            expires_at: None,
        })
    }
}

/// Refresh an expired access token using the stored refresh token.
pub fn refresh_access_token(refresh_token: &str) -> Result<(String, String), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("Failed to create client: {e}"))?;

    let resp = client
        .post(format!("{}/connect/token", OAUTH_URL))
        .form(&[
            ("client_id", CLIENT_ID),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ])
        .send()
        .map_err(|e| format!("Refresh request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().unwrap_or_default();
        return Err(format!("Refresh failed (HTTP {status}): {body}"));
    }

    let data: AuthResponse = resp
        .json()
        .map_err(|e| format!("Failed to parse refresh response: {e}"))?;

    Ok((data.auth_token, data.refresh_token))
}

// ── Login orchestrator ─────────────────────────────────────────────────

pub fn login(state: AppState, username: &str, password: &str) {
    state.loading.set(true);
    state.error_msg.set(String::new());

    let auth = Auth::new();
    match auth.login_blocking(username, password) {
        Ok(token) => {
            day::prefs::set(TOKEN_KEY, &token.access_token);
            day::prefs::set(REFRESH_KEY, &token.refresh_token);
            state.is_authenticated.set(true);
            state.loading.set(false);
            super::diary::load_all(state);
        }
        Err(e) => {
            state.error_msg.set(format!("Ошибка входа: {e}"));
            state.loading.set(false);
        }
    }
}

pub fn logout(state: AppState) {
    for key in [TOKEN_KEY, REFRESH_KEY, SCHOOL_ID_KEY, PROFILE_ID_KEY,
                 CLASS_ID_KEY, FULL_NAME_KEY, SCHOOL_NAME_KEY] {
        day::prefs::set(key, "");
    }
    state.is_authenticated.set(false);
    state.full_name.set(String::new());
    state.school_name.set(String::new());
    state.class_label.set(String::new());
    state.lessons.set(Vec::new());
    state.bell_times.set(Vec::new());
    state.timetable_days.set(Vec::new());
    state.subjects_teachers.set(Vec::new());
}

pub fn save_token(state: AppState, access: &str, refresh: &str) {
    day::prefs::set(TOKEN_KEY, access);
    day::prefs::set(REFRESH_KEY, refresh);
    state.is_authenticated.set(true);
    super::diary::load_all(state);
}

pub fn try_refresh_token() -> Option<String> {
    let refresh = day::prefs::get(REFRESH_KEY).filter(|r| !r.is_empty())?;
    nslog::nslog("[Auth] Attempting token refresh...");

    match refresh_access_token(&refresh) {
        Ok((new_access, new_refresh)) => {
            nslog::nslog("[Auth] Token refresh OK");
            day::prefs::set(TOKEN_KEY, &new_access);
            day::prefs::set(REFRESH_KEY, &new_refresh);
            Some(new_access)
        }
        Err(e) => {
            nslog::nslog(&format!("[Auth] Token refresh failed: {}", e));
            None
        }
    }
}

pub fn get_token() -> Option<String> {
    day::prefs::get(TOKEN_KEY).filter(|t| !t.is_empty())
}

pub fn get_stored_ids() -> (String, String, String) {
    (
        day::prefs::get(SCHOOL_ID_KEY).unwrap_or_default(),
        day::prefs::get(CLASS_ID_KEY).unwrap_or_default(),
        day::prefs::get(PROFILE_ID_KEY).unwrap_or_default(),
    )
}

pub fn store_ids(school_id: &str, class_id: &str, profile_id: &str, full_name: &str, school_name: &str) {
    day::prefs::set(SCHOOL_ID_KEY, school_id);
    day::prefs::set(CLASS_ID_KEY, class_id);
    day::prefs::set(PROFILE_ID_KEY, profile_id);
    day::prefs::set(FULL_NAME_KEY, full_name);
    day::prefs::set(SCHOOL_NAME_KEY, school_name);
}

// ── HTML parsing helpers ───────────────────────────────────────────────

fn extract_csrf(html: &str) -> Option<String> {
    let re = regex::Regex::new(r#"name="__RequestVerificationToken"[^>]*value="([^"]+)""#).ok()?;
    let caps = re.captures(html)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn extract_return_url(html: &str) -> Option<String> {
    let re = regex::Regex::new(r#"name="Input\.ReturnUrl"\s+value="([^"]+)""#).ok()?;
    let caps = re.captures(html)?;
    Some(decode_entities(caps.get(1)?.as_str()))
}

fn extract_origin(url: &str) -> String {
    let scheme_end = url.find("://").unwrap_or(0) + 3;
    let host_end = url[scheme_end..]
        .find('/')
        .map(|i| scheme_end + i)
        .unwrap_or(url.len());
    url[..host_end].to_string()
}

fn extract_uuid(url: &str) -> Option<String> {
    let re = regex::Regex::new(r"preauthorized\?data=([^&]+)").ok()?;
    let caps = re.captures(url)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}
