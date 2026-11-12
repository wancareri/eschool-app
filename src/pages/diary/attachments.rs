use crate::app::AppState;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LessonAttachment {
    pub id: String,
    pub title: String,
    pub url: String,
    pub is_link: bool,
    pub file_size: Option<String>,
}

#[cfg(any(feature = "uikit", feature = "appkit"))]
unsafe extern "C" {
    fn eschool_quicklook_file(path: *const std::ffi::c_char);
    fn eschool_share_file(path: *const std::ffi::c_char);
    fn eschool_open_url(url: *const std::ffi::c_char);
}

pub fn native_quicklook(path: &str) {
    crate::ios::nslog::nslog(&format!("[Attachments] native_quicklook: {path}"));
    #[cfg(any(feature = "uikit", feature = "appkit"))]
    {
        if let Ok(c_path) = std::ffi::CString::new(path) {
            unsafe { eschool_quicklook_file(c_path.as_ptr()) };
            return;
        }
    }
    #[cfg(not(feature = "material"))]
    let p = path.to_string();
    #[cfg(not(feature = "material"))]
    day::task(async move {
        if let Ok(url) = reqwest::Url::from_file_path(&p) {
            day::prelude::open_url(url.as_str());
        }
    });
}

pub fn native_share(path: &str) {
    crate::ios::nslog::nslog(&format!("[Attachments] native_share: {path}"));
    #[cfg(any(feature = "uikit", feature = "appkit"))]
    {
        if let Ok(c_path) = std::ffi::CString::new(path) {
            unsafe { eschool_share_file(c_path.as_ptr()) };
            return;
        }
    }
    #[cfg(not(feature = "material"))]
    let p = path.to_string();
    #[cfg(not(feature = "material"))]
    day::task(async move {
        if let Ok(url) = reqwest::Url::from_file_path(&p) {
            if !day::share_url(url.as_str(), "") {
                day::prelude::open_url(url.as_str());
            }
        }
    });
}

pub fn native_open_url(url: &str) {
    crate::ios::nslog::nslog(&format!("[Attachments] native_open_url: {url}"));
    #[cfg(any(feature = "uikit", feature = "appkit"))]
    {
        if let Ok(c_url) = std::ffi::CString::new(url) {
            unsafe { eschool_open_url(c_url.as_ptr()) };
            return;
        }
    }
    day::prelude::open_url(url);
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn sanitize_filename(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| {
            if c == '/'
                || c == '\\'
                || c == ':'
                || c == '*'
                || c == '?'
                || c == '"'
                || c == '<'
                || c == '>'
                || c == '|'
                || c == '\0'
            {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = clean.trim().trim_start_matches('.').trim();
    let sanitized = if trimmed.is_empty() || trimmed == ".." {
        "file".to_string()
    } else {
        trimmed.replace("..", "_")
    };
    // Filesystems limit bytes, not Unicode scalar values.
    let mut end = sanitized.len().min(180);
    while !sanitized.is_char_boundary(end) {
        end -= 1;
    }
    sanitized[..end].to_string()
}

fn may_send_token(url: &reqwest::Url) -> bool {
    url.scheme() == "https"
        && url.host_str() == Some("diary.e-schools.by")
        && url.port_or_known_default() == Some(443)
        && url.username().is_empty()
        && url.password().is_none()
}

pub fn parse_attachments(raw_json: &str) -> Vec<LessonAttachment> {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(raw_json) else {
        return Vec::new();
    };

    let mut result = Vec::new();

    fn parse_single_item(
        item: &serde_json::Value,
        default_is_link: bool,
    ) -> Option<LessonAttachment> {
        let id = item
            .get("id")
            .or_else(|| item.get("uuid"))
            .and_then(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .or_else(|| v.as_i64().map(|n| n.to_string()))
            })
            .unwrap_or_default();

        let title = item
            .get("title")
            .or_else(|| item.get("name"))
            .or_else(|| item.get("file_name"))
            .or_else(|| item.get("filename"))
            .or_else(|| item.get("original_filename"))
            .or_else(|| item.get("description"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let raw_url = item
            .get("url")
            .or_else(|| item.get("file_url"))
            .or_else(|| item.get("file"))
            .or_else(|| item.get("link"))
            .or_else(|| item.get("download_url"))
            .or_else(|| item.get("href"))
            .or_else(|| item.get("src"))
            .or_else(|| item.get("path"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if raw_url.is_empty() && title.is_empty() {
            return None;
        }

        let mut is_link = default_is_link;
        if let Some(kind) = item.get("type").and_then(|v| v.as_str()) {
            if kind == "link" || kind == "url" {
                is_link = true;
            } else if kind == "file" || kind == "attachment" {
                is_link = false;
            }
        }
        if item
            .get("is_link")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            is_link = true;
        }

        let full_url = if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
            raw_url.to_string()
        } else if raw_url.starts_with('/') {
            format!("https://diary.e-schools.by{raw_url}")
        } else if !raw_url.is_empty() {
            format!("https://diary.e-schools.by/{raw_url}")
        } else {
            String::new()
        };

        let clean_title = if !title.trim().is_empty() {
            title.trim().to_string()
        } else if let Some(last) = full_url.split('/').last().filter(|s| !s.is_empty()) {
            urlencoding::decode(last)
                .map(|s| s.into_owned())
                .unwrap_or_else(|_| last.to_string())
        } else {
            "Файл".to_string()
        };

        let file_size = item
            .get("file_size")
            .or_else(|| item.get("size"))
            .or_else(|| item.get("bytes"))
            .and_then(|v| v.as_u64())
            .map(format_size);

        Some(LessonAttachment {
            id,
            title: clean_title,
            url: full_url,
            is_link,
            file_size,
        })
    }

    match &val {
        serde_json::Value::Array(arr) => {
            for it in arr {
                if let Some(att) = parse_single_item(it, false) {
                    result.push(att);
                }
            }
        }
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::Array(arr)) = map
                .get("attachments")
                .or_else(|| map.get("files"))
                .or_else(|| map.get("items"))
            {
                for it in arr {
                    if let Some(att) = parse_single_item(it, false) {
                        result.push(att);
                    }
                }
            }
            if let Some(serde_json::Value::Array(arr)) = map.get("links") {
                for it in arr {
                    if let Some(att) = parse_single_item(it, true) {
                        result.push(att);
                    }
                }
            }
            if result.is_empty() {
                if let Some(att) = parse_single_item(&val, false) {
                    result.push(att);
                }
            }
        }
        _ => {}
    }

    result
}

pub fn download_to_path(token: &str, file_url: &str, dest_path: &Path) -> Result<String, String> {
    let full_url = if file_url.starts_with("http://") || file_url.starts_with("https://") {
        file_url.to_string()
    } else if file_url.starts_with('/') {
        format!("https://diary.e-schools.by{file_url}")
    } else {
        format!("https://diary.e-schools.by/{file_url}")
    };

    crate::ios::nslog::nslog(&format!(
        "[Attachments] Downloading: {full_url} -> {:?}",
        dest_path
    ));

    if let Some(parent) = dest_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 10 {
                attempt.error("Too many redirects")
            } else if attempt.url().scheme() != "https" {
                attempt.error("Insecure attachment redirect")
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|e| format!("Client build error: {e}"))?;

    let url = reqwest::Url::parse(&full_url).map_err(|e| format!("Invalid URL: {e}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("Unsupported attachment URL scheme".into());
    }
    let mut req = client.get(url.clone());
    if !token.is_empty() && may_send_token(&url) {
        req = req.header(reqwest::header::AUTHORIZATION, token);
    }

    let resp = req.send().map_err(|e| format!("HTTP send error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Download returned HTTP {}", resp.status()));
    }

    // Limit download size to max 100 MB to prevent disk exhaustion
    use std::io::Read;
    let limit = 100 * 1024 * 1024;
    if resp.content_length().is_some_and(|size| size > limit) {
        return Err("Attachment exceeds 100 MB".into());
    }
    // Publish only complete files. Failed/interrupted downloads must never
    // pass the existing-file check on the next open.
    let temp_path = dest_path.with_extension(format!("{}.part", rand::random::<u64>()));
    let result: Result<u64, String> = (|| {
        let mut file =
            std::fs::File::create(&temp_path).map_err(|e| format!("File create error: {e}"))?;
        let mut limited_reader = resp.take(limit + 1);
        let bytes = std::io::copy(&mut limited_reader, &mut file)
            .map_err(|e| format!("File write error: {e}"))?;
        if bytes > limit {
            return Err("Attachment exceeds 100 MB".into());
        }
        if bytes == 0 {
            return Err("Attachment is empty".into());
        }
        file.sync_all()
            .map_err(|e| format!("File sync error: {e}"))?;
        std::fs::rename(&temp_path, dest_path).map_err(|e| format!("File rename error: {e}"))?;
        Ok(bytes)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp_path);
    }
    let bytes_copied = result?;

    crate::ios::nslog::nslog(&format!(
        "[Attachments] Successfully saved {bytes_copied} bytes to {:?}",
        dest_path
    ));

    dest_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid UTF-8 in destination path".to_string())
}

pub fn get_local_path_for_attachment(att: &LessonAttachment) -> PathBuf {
    #[cfg(feature = "uikit")]
    let base_dir = {
        let ns_str = objc2_foundation::NSTemporaryDirectory();
        PathBuf::from(ns_str.to_string()).join("eschool_attachments")
    };
    #[cfg(not(feature = "uikit"))]
    let base_dir = std::env::temp_dir().join("eschool_attachments");

    let mut safe_name = sanitize_filename(&att.title);
    if !safe_name.contains('.') {
        if let Some(ext) = Path::new(&att.url).extension().and_then(|e| e.to_str()) {
            let clean_ext: String = ext.chars().take_while(|c| c.is_alphanumeric()).collect();
            if !clean_ext.is_empty() {
                safe_name = format!("{safe_name}.{clean_ext}");
            }
        }
    }
    use sha2::{Digest, Sha256};
    let identity = serde_json::to_vec(&(
        crate::app::data::credentials::load("auth.account_id"),
        crate::app::data::credentials::load("auth.school_id"),
        crate::app::data::credentials::load("auth.profile_id"),
        &att.id,
        &att.url,
    ))
    .unwrap();
    base_dir
        .join(format!("{:x}", Sha256::digest(identity)))
        .join(&safe_name)
}

pub fn is_attachment_downloaded(att: &LessonAttachment) -> bool {
    let p = get_local_path_for_attachment(att);
    p.exists() && std::fs::metadata(&p).map(|m| m.len() > 0).unwrap_or(false)
}

pub fn open_or_preview_attachment(state: AppState, att: &LessonAttachment) {
    if att.is_link {
        native_open_url(&att.url);
        return;
    }

    let target_path = get_local_path_for_attachment(att);
    if target_path.exists()
        && std::fs::metadata(&target_path)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
    {
        if let Some(path_str) = target_path.to_str() {
            native_quicklook(path_str);
        }
        return;
    }

    let att_id = if !att.id.is_empty() {
        att.id.clone()
    } else {
        att.url.clone()
    };
    let mut cur = state.downloading_files.get();
    cur.insert(att_id.clone());
    state.downloading_files.set(cur);

    let url = att.url.clone();
    let id_clone = att_id.clone();
    let token = crate::app::data::credentials::load("auth.token").unwrap_or_default();

    let request = crate::app::data::session::ticket();
    std::thread::spawn(move || {
        let res = download_to_path(&token, &url, &target_path);

        request.publish(move |_| {
            if let Some(state) = AppState::get_main() {
                let mut cur = state.downloading_files.get();
                cur.remove(&id_clone);
                state.downloading_files.set(cur);
            }

            match res {
                Ok(path) => {
                    native_quicklook(&path);
                }
                Err(err) => {
                    crate::ios::nslog::nslog(&format!("[Attachments] Download failed: {err}"));
                }
            }
        });
    });
}

pub fn share_attachment(state: AppState, att: &LessonAttachment) {
    if att.is_link {
        native_open_url(&att.url);
        return;
    }

    let target_path = get_local_path_for_attachment(att);
    if target_path.exists()
        && std::fs::metadata(&target_path)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
    {
        if let Some(path_str) = target_path.to_str() {
            native_share(path_str);
        }
        return;
    }

    let att_id = if !att.id.is_empty() {
        att.id.clone()
    } else {
        att.url.clone()
    };
    let mut cur = state.downloading_files.get();
    cur.insert(att_id.clone());
    state.downloading_files.set(cur);

    let url = att.url.clone();
    let id_clone = att_id.clone();
    let token = crate::app::data::credentials::load("auth.token").unwrap_or_default();

    let request = crate::app::data::session::ticket();
    std::thread::spawn(move || {
        let res = download_to_path(&token, &url, &target_path);

        request.publish(move |_| {
            if let Some(state) = AppState::get_main() {
                let mut cur = state.downloading_files.get();
                cur.remove(&id_clone);
                state.downloading_files.set(cur);
            }

            match res {
                Ok(path) => {
                    native_share(&path);
                }
                Err(err) => {
                    crate::ios::nslog::nslog(&format!(
                        "[Attachments] Download for share failed: {err}"
                    ));
                }
            }
        });
    });
}

pub fn fetch_attachments_async(state: AppState, lesson_uuid: &str) {
    if lesson_uuid.is_empty() {
        return;
    }
    let uuid = lesson_uuid.to_string();

    if state.lesson_attachments.with(|m| m.contains_key(&uuid)) {
        return;
    }
    if state.attachments_loading.with(|s| s.contains(&uuid)) {
        return;
    }

    let mut loading = state.attachments_loading.get();
    loading.insert(uuid.clone());
    state.attachments_loading.set(loading);

    let token = crate::app::data::credentials::load("auth.token").unwrap_or_default();
    let school_id = crate::app::data::credentials::load("auth.school_id").unwrap_or_default();
    let class_id = crate::app::data::credentials::load("auth.class_id").unwrap_or_default();
    let profile_id = crate::app::data::credentials::load("auth.profile_id").unwrap_or_default();

    let request = crate::app::data::session::ticket();
    std::thread::spawn(move || {
        let client = eschool_api::client::blocking::build_client(&token);
        let path =
            eschool_api::client::endpoints::attachments(&school_id, &class_id, &profile_id, &uuid);
        crate::ios::nslog::nslog(&format!("[Attachments] Fetching: {path}"));

        match eschool_api::client::blocking::api_get_raw(&client, &path) {
            Ok(raw) => {
                crate::ios::nslog::nslog(&format!(
                    "[Attachments] Response for {uuid} ({} bytes)",
                    raw.len()
                ));
                let parsed = parse_attachments(&raw);
                crate::ios::nslog::nslog(&format!(
                    "[Attachments] Parsed {} items for {uuid}",
                    parsed.len()
                ));

                request.publish(move |_| {
                    if let Some(state) = AppState::get_main() {
                        let mut atts = state.lesson_attachments.get();
                        atts.insert(uuid.clone(), parsed);
                        state.lesson_attachments.set(atts);

                        let mut loading = state.attachments_loading.get();
                        loading.remove(&uuid);
                        state.attachments_loading.set(loading);
                    }
                });
            }
            Err(e) => {
                crate::ios::nslog::nslog(&format!("[Attachments] Fetch failed for {uuid}: {e}"));
                request.publish(move |_| {
                    if let Some(state) = AppState::get_main() {
                        let mut loading = state.attachments_loading.get();
                        loading.remove(&uuid);
                        state.attachments_loading.set(loading);
                    }
                });
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_only_sent_to_the_exact_https_origin() {
        for url in [
            "https://example.com/schools.by/file",
            "https://diary.e-schools.by.evil.test/file",
            "https://diary.e-schools.by@evil.test/file",
            "http://diary.e-schools.by/file",
            "https://diary.e-schools.by:8443/file",
        ] {
            assert!(!may_send_token(&reqwest::Url::parse(url).unwrap()), "{url}");
        }
        assert!(may_send_token(
            &reqwest::Url::parse("https://diary.e-schools.by/file").unwrap()
        ));
    }

    #[test]
    fn identically_named_attachments_do_not_share_a_file() {
        let a = LessonAttachment {
            id: "a".into(),
            title: "Задание.pdf".into(),
            url: "https://diary.e-schools.by/a".into(),
            is_link: false,
            file_size: None,
        };
        let mut b = a.clone();
        b.id = "b".into();
        b.url = "https://diary.e-schools.by/b".into();
        assert_ne!(
            get_local_path_for_attachment(&a),
            get_local_path_for_attachment(&b)
        );
        assert_eq!(
            get_local_path_for_attachment(&a),
            get_local_path_for_attachment(&a)
        );
    }
}
