//! Blocking HTTP helpers for the e-schools.by API.

pub const BASE_URL: &str = "https://diary.e-schools.by";

pub fn build_client(token: &str) -> reqwest::blocking::Client {
    use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    if let Ok(v) = HeaderValue::from_str(token) {
        headers.insert(AUTHORIZATION, v);
    }
    reqwest::blocking::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(15))
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}

pub fn api_get<T: serde::de::DeserializeOwned>(
    client: &reqwest::blocking::Client,
    path: &str,
) -> Result<T, String> {
    let url = if path.starts_with("http") {
        path.to_string()
    } else {
        format!("{BASE_URL}{path}")
    };
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    resp.json::<T>()
        .map_err(|e| format!("parse failed: {e}"))
}

pub fn api_get_raw(client: &reqwest::blocking::Client, path: &str) -> Result<String, String> {
    let url = if path.starts_with("http") {
        path.to_string()
    } else {
        format!("{BASE_URL}{path}")
    };
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    resp.text()
        .map_err(|e| format!("read failed: {e}"))
}

pub fn api_post<T: serde::de::DeserializeOwned>(
    client: &reqwest::blocking::Client,
    path: &str,
    body: &serde_json::Value,
) -> Result<T, String> {
    let url = if path.starts_with("http") {
        path.to_string()
    } else {
        format!("{BASE_URL}{path}")
    };
    let resp = client
        .post(&url)
        .json(body)
        .send()
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    resp.json::<T>()
        .map_err(|e| format!("parse failed: {e}"))
}
