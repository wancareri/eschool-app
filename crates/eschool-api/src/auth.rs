//! Authentication — OAuth 7-step login flow and token refresh.

use serde::{Deserialize, Serialize};

use crate::client::blocking;
use crate::client::endpoints;

const UA: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

/// Result of parent login indicating if initial password setup is required.
#[derive(Debug, Clone)]
pub enum ParentLoginResult {
    /// Standard successful login
    Success(Token),
    /// Account logged in with temporary password / invitation code and needs permanent password
    RequiresNewPassword {
        token: Token,
        parent_uuid: String,
        temp_password: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
    #[serde(alias = "access_token")]
    pub auth_token: String,
    #[serde(default)]
    pub refresh_token: String,
}

pub struct Auth;

impl Auth {
    pub fn new() -> Self {
        Self
    }

    /// Login with username and password as a student (OAuth 7-step redirect flow).
    pub fn login_student_blocking(&self, username: &str, password: &str) -> Result<Token, String> {
        let client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(UA)
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("Client build: {e}"))?;

        // Step 1: GET diary login/student → 302 to OAuth
        let resp1 = client
            .get(format!("{}/{}", blocking::BASE_URL, endpoints::LOGIN_STUDENT.trim_start_matches('/')))
            .send()
            .map_err(|e| format_reqwest_error("Шаг 1 (старт авторизации)", &e))?;
        let oauth_login_url = resp1
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or("Шаг 1: сервер не вернул редирект (Location)")?
            .to_string();

        // Step 2: GET OAuth login page → extract CSRF + ReturnUrl
        let login_page_url = if oauth_login_url.starts_with("http") {
            oauth_login_url
        } else {
            format!("{}{}", endpoints::OAUTH_URL, oauth_login_url)
        };
        let resp2 = client
            .get(&login_page_url)
            .send()
            .map_err(|e| format_reqwest_error("Шаг 2 (страница входа OAuth)", &e))?;
        let login_html = resp2.text().map_err(|e| format!("Шаг 2 (чтение HTML): {e}"))?;

        let csrf = extract_csrf(&login_html).ok_or_else(|| "CSRF токен не найден на странице входа".to_string())?;
        let return_url =
            extract_return_url(&login_html).ok_or_else(|| "ReturnUrl не найден".to_string())?;

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

        let resp3 = client
            .post(format!("{}/Account/Login", oauth_origin))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .map_err(|e| format_reqwest_error("Шаг 3 (отправка логина и пароля)", &e))?;

        let callback_raw = match resp3.headers().get("location").and_then(|v| v.to_str().ok()) {
            Some(loc) => loc.to_string(),
            None => {
                let body = resp3.text().unwrap_or_default();
                if let Some(err) = extract_login_error(&body) {
                    return Err(err);
                }
                return Err("Неверное имя пользователя или пароль".to_string());
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
            .map_err(|e| format_reqwest_error("Шаг 4 (OAuth callback)", &e))?;

        let diary_callback_raw =
            match resp4.headers().get("location").and_then(|v| v.to_str().ok()) {
                Some(loc) => loc.to_string(),
                None => {
                    let body = resp4.text().unwrap_or_default();
                    return Err(format!(
                        "Шаг 4: сервер не вернул Location. Ответ: {}",
                        &body[..body.len().min(300)]
                    ));
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
            .map_err(|e| format_reqwest_error("Шаг 5 (Diary callback)", &e))?;

        let preauth_redirect_raw =
            match resp5.headers().get("location").and_then(|v| v.to_str().ok()) {
                Some(loc) => loc.to_string(),
                None => {
                    let body = resp5.text().unwrap_or_default();
                    return Err(format!(
                        "Шаг 5: сервер не вернул Location. Ответ: {}",
                        &body[..body.len().min(300)]
                    ));
                }
            };
        let preauth_redirect = decode_entities(&preauth_redirect_raw);
        let uuid = extract_uuid(&preauth_redirect)
            .ok_or_else(|| "Шаг 5: UUID не найден в ответе preauthorized".to_string())?;

        // Step 6: GET data_for_login/{UUID}
        let data_url = format!("{}/{}", blocking::BASE_URL, endpoints::data_for_login(&uuid).trim_start_matches('/'));
        let resp6 = client
            .get(&data_url)
            .send()
            .map_err(|e| format_reqwest_error("Шаг 6 (получение данных профиля)", &e))?;
        let login_data: serde_json::Value = resp6
            .json()
            .map_err(|e| format!("Шаг 6 (парсинг JSON профиля): {e}"))?;

        let profile_id = login_data["profile_id"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| login_data["profile_id"].as_i64().map(|n| n.to_string()))
            .ok_or("Шаг 6: не найден profile_id в ответе школы")?;
        let school_id = login_data["schools"]
            .as_array()
            .and_then(|s| s.first())
            .and_then(|s| s.get("id"))
            .and_then(|id| {
                id.as_str()
                    .map(|s| s.to_string())
                    .or_else(|| id.as_i64().map(|n| n.to_string()))
            })
            .ok_or("Шаг 6: не найден school_id в ответе школы")?;
        let kind = login_data["kinds"]
            .as_array()
            .and_then(|k| k.first())
            .and_then(|k| k.as_str())
            .unwrap_or("student")
            .to_lowercase();

        // Step 7: GET auth/login?token={base64}
        use base64::Engine;
        let token_payload = format!("{}:{}:{}", profile_id, school_id, kind);
        let token_b64 =
            base64::engine::general_purpose::STANDARD.encode(token_payload.as_bytes());
        let token_encoded = urlencoding::encode(&token_b64);
        let resp7 = client
            .get(format!(
                "{}/{}?token={}",
                blocking::BASE_URL,
                endpoints::AUTH_LOGIN.trim_start_matches('/'),
                token_encoded
            ))
            .send()
            .map_err(|e| format_reqwest_error("Шаг 7 (финальный вход и получение токена)", &e))?;

        let auth_data: AuthResponse = resp7
            .json()
            .map_err(|e| format!("Шаг 7 (парсинг токенов): {e}"))?;

        Ok(Token {
            access_token: auth_data.auth_token,
            refresh_token: auth_data.refresh_token,
        })
    }

    /// Login as parent: direct POST to `/api/v1/auth/login/parent` with JSON `{"login": "...", "password": "..."}`.
    pub fn login_parent_blocking(&self, username: &str, password: &str) -> Result<Token, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent(UA)
            .build()
            .map_err(|e| format!("Client build: {e}"))?;

        let url = format!("{}/{}", blocking::BASE_URL, endpoints::LOGIN_PARENT.trim_start_matches('/'));
        let body = serde_json::json!({
            "login": username,
            "password": password
        });

        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format_reqwest_error("Вход для родителя", &e))?;

        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().unwrap_or_default();
            if let Ok(err_val) = serde_json::from_str::<serde_json::Value>(&body_text) {
                if let Some(err_msg) = err_val.get("detail").and_then(|v| v.as_str()) {
                    return Err(err_msg.to_string());
                }
                if let Some(err_msg) = err_val.get("message").and_then(|v| v.as_str()) {
                    return Err(err_msg.to_string());
                }
                if let Some(arr) = err_val.get("non_field_errors").and_then(|v| v.as_array()) {
                    if let Some(msg) = arr.first().and_then(|v| v.as_str()) {
                        return Err(msg.to_string());
                    }
                }
            }
            if status.as_u16() == 400 || status.as_u16() == 401 || status.as_u16() == 403 {
                return Err("Неверное имя пользователя или пароль (родительский вход)".to_string());
            }
            return Err(format!("Ошибка входа родителя (HTTP {status}): {body_text}"));
        }

        let auth_data: AuthResponse = resp
            .json()
            .map_err(|e| format!("Парсинг ответа авторизации: {e}"))?;

        Ok(Token {
            access_token: auth_data.auth_token,
            refresh_token: auth_data.refresh_token,
        })
    }

    /// Login as parent and inspect if the account requires temporary password setup (registration).
    pub fn login_parent_with_status_blocking(
        &self,
        username: &str,
        password: &str,
    ) -> Result<ParentLoginResult, String> {
        let token = self.login_parent_blocking(username, password)?;
        if let Some(parent_uuid) = extract_token_usi(&token.access_token) {
            let client = blocking::build_client(&token.access_token);
            if let Ok(info) = blocking::get_parent_details(&client, &parent_uuid) {
                if info.is_temp_password == Some(true) {
                    return Ok(ParentLoginResult::RequiresNewPassword {
                        token,
                        parent_uuid,
                        temp_password: info.temp_password.or_else(|| Some(password.to_string())),
                    });
                }
            }
        }
        Ok(ParentLoginResult::Success(token))
    }

    /// Automatically attempts login based on username format.
    /// If the username consists only of digits (typical for parent IDs), tries parent login first.
    /// If that fails or if the username has letters, tries student login, then falls back.
    pub fn login_auto_blocking(&self, username: &str, password: &str) -> Result<Token, String> {
        let is_all_digits = !username.is_empty() && username.chars().all(|c| c.is_ascii_digit());
        if is_all_digits {
            match self.login_parent_blocking(username, password) {
                Ok(token) => return Ok(token),
                Err(err) => {
                    // Try student login in case it's a student with numeric username
                    if let Ok(token) = self.login_student_blocking(username, password) {
                        return Ok(token);
                    }
                    return Err(err);
                }
            }
        }

        // Otherwise try student login first
        match self.login_student_blocking(username, password) {
            Ok(token) => Ok(token),
            Err(student_err) => {
                // Try parent login as fallback
                match self.login_parent_blocking(username, password) {
                    Ok(token) => Ok(token),
                    Err(_) => Err(student_err),
                }
            }
        }
    }

    /// Login with username and password (auto-detects student or parent account).
    pub fn login_blocking(&self, username: &str, password: &str) -> Result<Token, String> {
        self.login_auto_blocking(username, password)
    }
}

/// Refresh an expired access token using the stored refresh token.
///
/// Uses the diary API's own refresh endpoint (not IdentityServer's /connect/token):
/// ```text
/// PUT /api/v1/auth/refresh
/// Content-Type: text/plain
/// Body: <refresh_token>
/// ```
pub fn refresh_access_token(refresh_token: &str) -> Result<(String, String), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create client: {e}"))?;

    let url = format!("{}{}", blocking::BASE_URL, endpoints::AUTH_REFRESH);
    let resp = client
        .put(&url)
        .header("Content-Type", "text/plain")
        .body(refresh_token.to_string())
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

/// Full re-login: tries refresh first, falls back to username/password.
pub fn relogin(
    username: &str,
    password: &str,
    refresh_token: Option<&str>,
) -> Result<Token, String> {
    // Try refresh first (works on most IPs)
    if let Some(rt) = refresh_token {
        if !rt.is_empty() {
            match refresh_access_token(rt) {
                Ok((access, refresh)) => {
                    return Ok(Token {
                        access_token: access,
                        refresh_token: refresh,
                    });
                }
                Err(e) => {
                    eprintln!("[eschool-api] Refresh failed ({e}), falling back to full login");
                }
            }
        }
    }

    // Fallback: full 7-step OAuth login
    Auth::new().login_blocking(username, password)
}

fn format_reqwest_error(step: &str, e: &reqwest::Error) -> String {
    let mut parts = Vec::new();
    if e.is_timeout() {
        parts.push("Превышено время ожидания ответа (таймаут)".to_string());
    } else if e.is_connect() {
        parts.push("Не удалось подключиться к серверу".to_string());
    }
    let mut curr: Option<&(dyn std::error::Error + 'static)> = Some(e);
    while let Some(err) = curr {
        let s = err.to_string();
        if !parts.contains(&s) {
            parts.push(s);
        }
        curr = err.source();
    }
    format!("{step}: {}", parts.join(" — "))
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

fn decode_html_entities(s: &str) -> String {
    let mut out = decode_entities(s);
    if let Ok(hex_re) = regex::Regex::new(r"&#x([0-9a-fA-F]+);") {
        out = hex_re
            .replace_all(&out, |caps: &regex::Captures| {
                if let Ok(code) = u32::from_str_radix(&caps[1], 16) {
                    if let Some(ch) = char::from_u32(code) {
                        return ch.to_string();
                    }
                }
                caps[0].to_string()
            })
            .to_string();
    }
    if let Ok(dec_re) = regex::Regex::new(r"&#([0-9]+);") {
        out = dec_re
            .replace_all(&out, |caps: &regex::Captures| {
                if let Ok(code) = caps[1].parse::<u32>() {
                    if let Some(ch) = char::from_u32(code) {
                        return ch.to_string();
                    }
                }
                caps[0].to_string()
            })
            .to_string();
    }
    out
}

fn extract_login_error(html: &str) -> Option<String> {
    let re = regex::Regex::new(
        r#"(?s)<div[^>]*class="[^"]*(?:validation-summary-errors|alert-danger)[^"]*"[^>]*>.*?<li>([^<]+)</li>"#,
    ).ok()?;
    let caps = re.captures(html)?;
    let raw = caps.get(1)?.as_str().trim();
    let decoded = decode_html_entities(raw);
    if !decoded.is_empty() {
        Some(decoded)
    } else {
        None
    }
}

/// Extracts the `usi` (user/parent UUID) claim from a JWT access token.
pub fn extract_token_usi(token: &str) -> Option<String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    use base64::Engine;
    let payload_bytes = base64::engine::general_purpose::STANDARD
        .decode(parts[1])
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1]))
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(parts[1]))
        .ok()?;
    let json: serde_json::Value = serde_json::from_slice(&payload_bytes).ok()?;
    json.get("usi").and_then(|v| v.as_str()).map(|s| s.to_string())
}

const SPECIAL_PASSWORD_CHARS: &str = "!@#$%^*()-_=+[]{};:,.?/";

/// Validates parent password against e-schools rules:
/// - 12 to 15 chars
/// - at least 1 lowercase letter (a-z)
/// - at least 1 uppercase letter (A-Z)
/// - at least 1 digit (0-9)
/// - at least 1 special character (!@#$%^*()-_=+[]{};:,.?/)
/// - no spaces, quotes, backslashes, angle brackets, cyrillic, etc.
pub fn validate_parent_password(password: &str) -> Result<(), String> {
    let mut missing = Vec::new();
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        missing.push("одну строчную латинскую букву");
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        missing.push("одну заглавную латинскую букву");
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        missing.push("одну цифру");
    }
    if !password.chars().any(|c| SPECIAL_PASSWORD_CHARS.contains(c)) {
        missing.push("один спецсимвол (!@#$%^*()-_=+[]{};:,.?/)");
    }

    let len_invalid = password.len() < 12 || password.len() > 15;
    if len_invalid && !missing.is_empty() {
        return Err(format!(
            "Пароль должен содержать от 12 до 15 символов, включая минимум: {}",
            missing.join(", ")
        ));
    }
    if len_invalid {
        return Err("Пароль должен содержать от 12 до 15 символов".to_string());
    }
    if !missing.is_empty() {
        return Err(format!(
            "Пароль должен содержать минимум: {}",
            missing.join(", ")
        ));
    }

    for c in password.chars() {
        if c.is_whitespace()
            || matches!(c, '"' | '\'' | '\\' | '<' | '>' | '~' | '`' | '&' | '|')
            || ('\u{0400}'..='\u{04FF}').contains(&c)
            || c.is_control()
        {
            return Err("Пароль содержит запрещенные символы (пробел, кавычки, \\, <, >, ~, `, &, |, кириллица)".to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_html_entities() {
        let input = "&#x41D;&#x435;&#x432;&#x435;&#x440;&#x43D;&#x43E;&#x435; &amp; test";
        assert_eq!(decode_html_entities(input), "Неверное & test");
    }

    #[test]
    fn test_extract_login_error() {
        let html = r#"
            <div class="alert alert-danger">
            <div class="danger validation-summary-errors" data-valmsg-summary="true"><ul><li>&#x41D;&#x435;&#x432;&#x435;&#x440;&#x43D;&#x43E;&#x435; &#x438;&#x43C;&#x44F; &#x43F;&#x43E;&#x43B;&#x44C;&#x437;&#x43E;&#x432;&#x430;&#x442;&#x435;&#x43B;&#x44F; &#x438;&#x43B;&#x438; &#x43F;&#x430;&#x440;&#x43E;&#x43B;&#x44C;</li>
            </ul></div></div>
        "#;
        assert_eq!(
            extract_login_error(html),
            Some("Неверное имя пользователя или пароль".into())
        );
    }

    #[test]
    fn test_extract_token_usi() {
        // Token payload with usi="d3a55788-19b4-47e4-abe6-bf5141b2ed50"
        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJqdGkiOiI5ZmMwZjNhZC1hNzg1LTQ4MjMtOTFjNi00OGY1Mjc1N2M1NDIiLCJzdWIiOiJBVVRIIiwiaWF0IjoxNzkyNDQ1NDk3LCJleHAiOjE3OTI0NTI2OTcsInVzaSI6ImQzYTU1Nzg4LTE5YjQtNDdlNC1hYmU2LWJmNTE0MWIyZWQ1MCJ9.JXVjMwy7MjdHpruXIC9Ltur26EE2ChgfqP1lOCpzMlo";
        let usi = extract_token_usi(token);
        assert_eq!(usi, Some("d3a55788-19b4-47e4-abe6-bf5141b2ed50".to_string()));
    }

    #[test]
    fn test_validate_parent_password() {
        // Valid password: 12-15 chars, uppercase, lowercase, digit, special char
        assert!(validate_parent_password("*tQ6gjZUz9yP").is_ok());

        // Too short (< 12)
        assert!(validate_parent_password("Short1!").is_err());

        // Too long (> 15)
        assert!(validate_parent_password("VeryLongPassword123!@#").is_err());

        // Missing digit
        assert!(validate_parent_password("PasswordSpecial!").is_err());

        // Missing uppercase
        assert!(validate_parent_password("password12345!").is_err());

        // Missing lowercase
        assert!(validate_parent_password("PASSWORD12345!").is_err());

        // Missing special char
        assert!(validate_parent_password("Password123456").is_err());

        // Prohibited char (cyrillic)
        assert!(validate_parent_password("*tQ6gjZUz9yП").is_err());
    }
}
