use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sha2::{Sha256, Digest};
use rand::Rng;

use super::client::ApiClient;

const CLIENT_ID: &str = "oauth_diary_echools";
const REDIRECT_URI: &str = "eschool-app://callback";
const SCOPE: &str = "openid profile offline_access organization.write person.write person.write.all person.read persons.read dictionaries.read organization.read";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreauthData {
    pub profile_id: String,
    pub kinds: Vec<String>,
    pub schools: Vec<School>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct School {
    pub id: String,
    pub name: String,
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
        let params = [
            ("client_id", CLIENT_ID),
            ("response_type", "code"),
            ("redirect_uri", REDIRECT_URI),
            ("scope", SCOPE),
            ("code_challenge", &self.code_challenge),
            ("code_challenge_method", "S256"),
            ("state", &self.state),
        ];

        let query: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}/Account/Login?ReturnUrl=%2Fconnect%2Fauthorize%2Fcallback%3F{}", 
            ApiClient::oauth_url(), query)
    }

    pub fn code_verifier(&self) -> &str {
        &self.code_verifier
    }

    pub async fn exchange_code(&self, code: &str) -> Result<Token, reqwest::Error> {
        let client = reqwest::Client::new();
        
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", REDIRECT_URI),
            ("client_id", CLIENT_ID),
            ("code_verifier", &self.code_verifier),
        ];

        let response = client
            .post(format!("{}/connect/token", ApiClient::oauth_url()))
            .form(&params)
            .send()
            .await?
            .json::<AuthResponse>()
            .await?;

        Ok(Token {
            access_token: response.auth_token,
            refresh_token: response.refresh_token,
            expires_at: None,
        })
    }

    pub async fn get_preauth_data(&self, uuid: &str) -> Result<PreauthData, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/api/v1/admin/auth/data_for_login/{}", ApiClient::base_url(), uuid))
            .send()
            .await?
            .json::<PreauthData>()
            .await?;
        Ok(response)
    }
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
