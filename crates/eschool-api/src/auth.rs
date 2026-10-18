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

    /// Login with username and password — pure HTTP redirect flow.
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
        let resp1 = client
            .get(format!("{}/{}", blocking::BASE_URL, endpoints::LOGIN_STUDENT.trim_start_matches('/')))
            .send()
            .map_err(|e| format!("Step 1: {e}"))?;
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
            format!("{}{}", endpoints::OAUTH_URL, oauth_login_url)
        };
        let resp2 = client
            .get(&login_page_url)
            .send()
            .map_err(|e| format!("Step 2: {e}"))?;
        let login_html = resp2.text().map_err(|e| format!("Step 2 text: {e}"))?;

        let csrf = extract_csrf(&login_html).ok_or_else(|| "CSRF not found".to_string())?;
        let return_url =
            extract_return_url(&login_html).ok_or_else(|| "ReturnUrl not found".to_string())?;

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
            .map_err(|e| format!("Step 3: {e}"))?;

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
            .map_err(|e| format!("Step 4: {e}"))?;

        let diary_callback_raw =
            match resp4.headers().get("location").and_then(|v| v.to_str().ok()) {
                Some(loc) => loc.to_string(),
                None => {
                    let body = resp4.text().unwrap_or_default();
                    return Err(format!(
                        "Step 4: no Location. Body: {}",
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
            .map_err(|e| format!("Step 5: {e}"))?;

        let preauth_redirect_raw =
            match resp5.headers().get("location").and_then(|v| v.to_str().ok()) {
                Some(loc) => loc.to_string(),
                None => {
                    let body = resp5.text().unwrap_or_default();
                    return Err(format!(
                        "Step 5: no Location. Body: {}",
                        &body[..body.len().min(300)]
                    ));
                }
            };
        let preauth_redirect = decode_entities(&preauth_redirect_raw);
        let uuid = extract_uuid(&preauth_redirect)
            .ok_or_else(|| "UUID not found in preauth redirect".to_string())?;

        // Step 6: GET data_for_login/{UUID}
        let data_url = format!("{}/{}", blocking::BASE_URL, endpoints::data_for_login(&uuid).trim_start_matches('/'));
        let resp6 = client
            .get(&data_url)
            .send()
            .map_err(|e| format!("Step 6: {e}"))?;
        let login_data: serde_json::Value = resp6
            .json()
            .map_err(|e| format!("Step 6 parse: {e}"))?;

        let profile_id = login_data["profile_id"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| login_data["profile_id"].as_i64().map(|n| n.to_string()))
            .ok_or("Step 6: no profile_id")?;
        let school_id = login_data["schools"]
            .as_array()
            .and_then(|s| s.first())
            .and_then(|s| s.get("id"))
            .and_then(|id| {
                id.as_str()
                    .map(|s| s.to_string())
                    .or_else(|| id.as_i64().map(|n| n.to_string()))
            })
            .ok_or("Step 6: no school_id")?;
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
            .map_err(|e| format!("Step 7: {e}"))?;

        let auth_data: AuthResponse = resp7
            .json()
            .map_err(|e| format!("Step 7 parse: {e}"))?;

        Ok(Token {
            access_token: auth_data.auth_token,
            refresh_token: auth_data.refresh_token,
        })
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
        .danger_accept_invalid_certs(true)
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
}
