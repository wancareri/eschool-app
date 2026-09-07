use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use super::auth::Token;

const BASE_URL: &str = "https://diary.e-schools.by";
const OAUTH_URL: &str = "https://oauth.rios.unibel.by";

pub struct ApiClient {
    client: Client,
    token: Option<Token>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    #[serde(flatten)]
    pub data: T,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            token: None,
        }
    }

    pub fn set_token(&mut self, token: Token) {
        self.token = Some(token);
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        if let Some(token) = &self.token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token.access_token))
                    .expect("Invalid token"),
            );
        }
        
        headers
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, reqwest::Error> {
        let url = format!("{}{}", BASE_URL, path);
        let response = self.client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?
            .json::<T>()
            .await?;
        Ok(response)
    }

    pub async fn post<T: serde::de::DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T, reqwest::Error> {
        let url = format!("{}{}", BASE_URL, path);
        let response = self.client
            .post(&url)
            .headers(self.headers())
            .json(body)
            .send()
            .await?
            .json::<T>()
            .await?;
        Ok(response)
    }

    pub fn base_url() -> &'static str {
        BASE_URL
    }

    pub fn oauth_url() -> &'static str {
        OAUTH_URL
    }
}
