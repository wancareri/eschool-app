//! JSON file cache for API data — shows stale data on startup while refreshing.
//! Uses day::prefs (NSUserDefaults on iOS) for persistence.

use crate::shared::nslog;

pub fn save(key: &str, data: &str) {
    let prefixed = format!("cache.{key}");
    day::prefs::set(&prefixed, data);
}

pub fn load(key: &str) -> Option<String> {
    let prefixed = format!("cache.{key}");
    day::prefs::get(&prefixed).filter(|s| !s.is_empty())
}

pub fn save_json<T: serde::Serialize>(key: &str, val: &T) {
    match serde_json::to_string(val) {
        Ok(j) => {
            save(key, &j);
            nslog::nslog(&format!("[Cache] Saved: {key} ({} bytes)", j.len()));
        }
        Err(e) => nslog::nslog(&format!("[Cache] Serialize failed for {key}: {e}")),
    }
}

pub fn load_json<T: for<'de> serde::Deserialize<'de>>(key: &str) -> Option<T> {
    load(key).and_then(|j| {
        match serde_json::from_str(&j) {
            Ok(v) => {
                nslog::nslog(&format!("[Cache] Loaded: {key}"));
                Some(v)
            }
            Err(e) => {
                nslog::nslog(&format!("[Cache] Deserialize failed for {key}: {e}"));
                None
            }
        }
    })
}

pub fn has(key: &str) -> bool {
    let prefixed = format!("cache.{key}");
    day::prefs::get(&prefixed).is_some()
}

pub fn clear() {
    nslog::nslog("[Cache] Clearing all cache keys");
}
