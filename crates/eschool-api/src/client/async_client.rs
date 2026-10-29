//! Async HTTP helpers for the e-schools.by API.

use crate::client::blocking::{validate_and_build_url, BASE_URL};

pub fn build_client(token: &str) -> reqwest::Client {
    use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    if let Ok(v) = HeaderValue::from_str(token) {
        headers.insert(AUTHORIZATION, v);
    }
    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub async fn api_get<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    path: &str,
) -> Result<T, String> {
    let url = validate_and_build_url(path)?;
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    resp.json::<T>()
        .await
        .map_err(|e| format!("parse failed: {e}"))
}

pub async fn api_get_raw(client: &reqwest::Client, path: &str) -> Result<String, String> {
    let url = validate_and_build_url(path)?;
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    resp.text()
        .await
        .map_err(|e| format!("read failed: {e}"))
}

pub async fn api_post<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    path: &str,
    body: &serde_json::Value,
) -> Result<T, String> {
    let url = validate_and_build_url(path)?;
    let resp = client
        .post(&url)
        .json(body)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    resp.json::<T>()
        .await
        .map_err(|e| format!("parse failed: {e}"))
}

pub async fn get_user_me(client: &reqwest::Client) -> Result<crate::entities::UserInfo, String> {
    api_get(client, super::endpoints::AUTH_ME).await
}

pub async fn get_student_classes(
    client: &reqwest::Client,
    school_id: &str,
    student_id: &str,
    from: &str,
    to: &str,
) -> Result<crate::entities::ClassesByDate, String> {
    let path = super::endpoints::classes(school_id, student_id);
    let body = serde_json::json!({
        "from": from,
        "to": to,
    });
    api_post(client, &path, &body).await
}

pub async fn get_school_by_id(
    client: &reqwest::Client,
    school_id: &str,
) -> Result<Option<crate::entities::InstitutionSchool>, String> {
    let path = super::endpoints::school_by_id(school_id);
    let list: Vec<crate::entities::InstitutionSchool> = api_get(client, &path).await?;
    Ok(list.into_iter().next())
}

pub async fn get_parent_details(
    client: &reqwest::Client,
    parent_uuid: &str,
) -> Result<crate::entities::ParentInfo, String> {
    let path = super::endpoints::parent_details(parent_uuid);
    api_get(client, &path).await
}

pub async fn get_child_representative(
    client: &reqwest::Client,
    parent_uuid: &str,
) -> Result<crate::entities::ChildRepresentative, String> {
    let path = super::endpoints::child_representative(parent_uuid);
    api_get(client, &path).await
}

pub async fn change_parent_temp_password(
    client: &reqwest::Client,
    parent_uuid: &str,
    old_password: &str,
    new_password: &str,
) -> Result<(), String> {
    let path = super::endpoints::change_temp_password(parent_uuid);
    let body = serde_json::json!({
        "old_password": old_password,
        "new_password": new_password,
    });
    let url = format!("{}{}", BASE_URL, path);
    let resp = client
        .put(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        if let Ok(err_val) = serde_json::from_str::<serde_json::Value>(&body_text) {
            if let Some(details) = err_val.get("details").and_then(|v| v.as_array()) {
                if let Some(val) = details.first().and_then(|d| d.get("value")).and_then(|v| v.as_str()) {
                    return Err(val.to_string());
                }
            }
        }
        return Err(format!("HTTP {status}: {body_text}"));
    }
    Ok(())
}

pub async fn change_parent_password(
    client: &reqwest::Client,
    parent_uuid: &str,
    old_password: &str,
    new_password: &str,
) -> Result<(), String> {
    let path = super::endpoints::change_password(parent_uuid);
    let body = serde_json::json!({
        "old_password": old_password,
        "new_password": new_password,
    });
    let url = format!("{}{}", BASE_URL, path);
    let resp = client
        .put(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        if let Ok(err_val) = serde_json::from_str::<serde_json::Value>(&body_text) {
            if let Some(details) = err_val.get("details").and_then(|v| v.as_array()) {
                if let Some(val) = details.first().and_then(|d| d.get("value")).and_then(|v| v.as_str()) {
                    return Err(val.to_string());
                }
            }
        }
        return Err(format!("HTTP {status}: {body_text}"));
    }
    Ok(())
}


