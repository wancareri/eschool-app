use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use regex::Regex;

const CLIENT_ID: &str = "oauth_diary_echools";
const REDIRECT_URI: &str = "https://diary.e-schools.by/api/v1/admin/auth/callback";
const SCOPE: &str = "openid profile offline_access organization.write person.write person.write.all person.read persons.read dictionaries.read organization.read";
const OAUTH_URL: &str = "https://oauth.rios.unibel.by";
pub(crate) const BASE_URL: &str = "https://diary.e-schools.by";
const UA: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";

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
    state: String,
}

impl Auth {
    pub fn new() -> Self {
        Self {
            state: "schools".to_string(),
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

    pub(crate) fn build_return_url(&self) -> String {
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
        eprintln!("[OAuth] Step 1: GET diary login/student");
        let resp1 = client
            .get(format!("{}/api/v1/admin/auth/login/student", BASE_URL))
            .send()
            .map_err(|e| format!("Step 1: {e}"))?;
        eprintln!("[OAuth] Step 1: status={}", resp1.status());
        let oauth_login_url = resp1
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or("Step 1: no Location header")?
            .to_string();
        eprintln!("[OAuth] Step 1: redirect → {}", &oauth_login_url[..oauth_login_url.len().min(120)]);

        // Step 2: GET OAuth login page → extract CSRF + ReturnUrl
        let login_page_url = if oauth_login_url.starts_with("http") {
            oauth_login_url
        } else {
            format!("{}{}", OAUTH_URL, oauth_login_url)
        };
        eprintln!("[OAuth] Step 2: GET {}", &login_page_url[..login_page_url.len().min(120)]);
        let resp2 = client
            .get(&login_page_url)
            .send()
            .map_err(|e| format!("Step 2: {e}"))?;
        eprintln!("[OAuth] Step 2: status={}", resp2.status());
        let login_html = resp2.text().map_err(|e| format!("Step 2 text: {e}"))?;

        let csrf = extract_csrf(&login_html)
            .ok_or_else(|| {
                let snippet: String = login_html.chars().take(300).collect();
                eprintln!("[OAuth] Step 2: CSRF not found. HTML: {snippet}");
                format!("CSRF not found. HTML: {snippet}")
            })?;
        let return_url = extract_return_url(&login_html)
            .ok_or_else(|| {
                let snippet: String = login_html.chars().take(300).collect();
                eprintln!("[OAuth] Step 2: ReturnUrl not found. HTML: {snippet}");
                format!("ReturnUrl not found. HTML: {snippet}")
            })?;
        eprintln!("[OAuth] Step 2: CSRF={}… ReturnUrl={}…", &csrf[..csrf.len().min(30)], &return_url[..return_url.len().min(80)]);

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

        eprintln!("[OAuth] Step 3: POST {}/Account/Login", oauth_origin);
        let resp3 = client
            .post(format!("{}/Account/Login", oauth_origin))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .map_err(|e| format!("Step 3: {e}"))?;
        eprintln!("[OAuth] Step 3: status={}", resp3.status());

        let callback_raw = resp3
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                let body = resp3.text().unwrap_or_default();
                let snippet: String = body.chars().take(300).collect();
                eprintln!("[OAuth] Step 3: no Location. Body: {snippet}");
                format!("Step 3: no Location header. Body: {snippet}")
            })?
            .to_string();
        let callback_url = decode_entities(&callback_raw);
        eprintln!("[OAuth] Step 3: callback → {}", &callback_url[..callback_url.len().min(120)]);

        // Step 4: GET callback → 302 to diary callback
        let resp4_url = if callback_url.starts_with("http") {
            callback_url
        } else {
            format!("{}{}", oauth_origin, callback_url)
        };
        eprintln!("[OAuth] Step 4: GET {}", &resp4_url[..resp4_url.len().min(120)]);
        let resp4 = client
            .get(&resp4_url)
            .header("Referer", format!("{}/", oauth_origin))
            .send()
            .map_err(|e| format!("Step 4: {e}"))?;
        eprintln!("[OAuth] Step 4: status={}", resp4.status());

        let diary_callback_raw = resp4
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                let body = resp4.text().unwrap_or_default();
                let snippet: String = body.chars().take(300).collect();
                eprintln!("[OAuth] Step 4: no Location. Body: {snippet}");
                format!("Step 4: no Location header (expected diary callback). Body: {snippet}")
            })?
            .to_string();
        let diary_callback_url = decode_entities(&diary_callback_raw);
        eprintln!("[OAuth] Step 4: diary callback → {}", &diary_callback_url[..diary_callback_url.len().min(120)]);

        // Step 5: GET diary callback → 302 to preauthorized?data=UUID
        let resp5_url = if diary_callback_url.starts_with("http") {
            diary_callback_url
        } else {
            format!("{}{}", BASE_URL, diary_callback_url)
        };
        eprintln!("[OAuth] Step 5: GET {}", &resp5_url[..resp5_url.len().min(120)]);
        let resp5 = client
            .get(&resp5_url)
            .send()
            .map_err(|e| format!("Step 5: {e}"))?;
        eprintln!("[OAuth] Step 5: status={}", resp5.status());

        let preauth_redirect_raw = resp5
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                let body = resp5.text().unwrap_or_default();
                let snippet: String = body.chars().take(300).collect();
                eprintln!("[OAuth] Step 5: no Location. Body: {snippet}");
                format!("Step 5: no Location header (expected preauthorized). Body: {snippet}")
            })?
            .to_string();
        let preauth_redirect = decode_entities(&preauth_redirect_raw);
        eprintln!("[OAuth] Step 5: preauth → {}", &preauth_redirect[..preauth_redirect.len().min(120)]);

        let uuid = extract_uuid(&preauth_redirect)
            .ok_or_else(|| format!("UUID not found in preauth redirect: {}", &preauth_redirect))?;
        eprintln!("[OAuth] Step 5: UUID = {}", uuid);

        // Step 6: GET data_for_login/{UUID}
        let data_url = format!("{}/api/v1/admin/auth/data_for_login/{}", BASE_URL, uuid);
        eprintln!("[OAuth] Step 6: GET data_for_login/{}", uuid);
        let resp6 = client
            .get(&data_url)
            .send()
            .map_err(|e| format!("Step 6: {e}"))?;
        eprintln!("[OAuth] Step 6: status={}", resp6.status());
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
        eprintln!("[OAuth] Step 6: profile={} school={} kind={}", profile_id, school_id, kind);

        // Step 7: GET auth/login?token={base64}
        let token_payload = format!("{}:{}:{}", profile_id, school_id, kind);
        let token_b64 = BASE64.encode(token_payload.as_bytes());
        eprintln!("[OAuth] Step 7: GET auth/login?token=…");
        let resp7 = client
            .get(format!("{}/api/v1/auth/login?token={}", BASE_URL, token_b64))
            .send()
            .map_err(|e| format!("Step 7: {e}"))?;
        eprintln!("[OAuth] Step 7: status={}", resp7.status());

        let auth_data: AuthResponse = resp7
            .json()
            .map_err(|e| format!("Step 7 parse: {e}"))?;

        eprintln!("[OAuth] Done!");
        Ok(Token {
            access_token: auth_data.auth_token,
            refresh_token: auth_data.refresh_token,
            expires_at: None,
        })
    }
}

/// Refresh an expired access token using the stored refresh token.
pub(crate) fn refresh_access_token(
    refresh_token: &str,
) -> Result<(String, String), String> {
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

fn extract_origin(url: &str) -> String {
    let scheme_end = url.find("://").unwrap_or(0) + 3;
    let host_end = url[scheme_end..]
        .find('/')
        .map(|i| scheme_end + i)
        .unwrap_or(url.len());
    url[..host_end].to_string()
}

fn extract_uuid(url: &str) -> Option<String> {
    let re = Regex::new(r"preauthorized\?data=([^&]+)").ok()?;
    let caps = re.captures(url)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}
