//! JSON file cache for API data — shows stale data on startup while refreshing.

use std::fs;
use std::path::PathBuf;

fn cache_dir() -> PathBuf {
    let mut p = PathBuf::new();
    p.push("cache");
    let _ = fs::create_dir_all(&p);
    p
}

pub fn save(key: &str, data: &str) {
    let path = cache_dir().join(format!("{key}.json"));
    let _ = fs::write(&path, data);
}

pub fn load(key: &str) -> Option<String> {
    let path = cache_dir().join(format!("{key}.json"));
    fs::read_to_string(&path).ok().filter(|s| !s.is_empty())
}

pub fn save_json<T: serde::Serialize>(key: &str, val: &T) {
    if let Ok(j) = serde_json::to_string(val) {
        save(key, &j);
    }
}

pub fn load_json<T: for<'de> serde::Deserialize<'de>>(key: &str) -> Option<T> {
    load(key).and_then(|j| serde_json::from_str(&j).ok())
}

pub fn has(key: &str) -> bool {
    let path = cache_dir().join(format!("{key}.json"));
    path.exists()
}

pub fn clear() {
    let _ = fs::remove_dir_all(cache_dir());
}
