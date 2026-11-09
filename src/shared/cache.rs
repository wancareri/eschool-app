//! JSON file cache for API data — shows stale data on startup while refreshing.
//! Uses day::prefs (NSUserDefaults on iOS) for persistence.

use crate::shared::nslog;

fn scoped_key(key: &str) -> Option<String> {
    let account = crate::shared::secure::load("auth.account_id")?;
    let school = crate::shared::secure::load("auth.school_id")?;
    let profile = crate::shared::secure::load("auth.profile_id")?;
    Some(key_for(&account, &school, &profile, key))
}

fn key_for(account: &str, school: &str, profile: &str, key: &str) -> String {
    use sha2::{Digest, Sha256};
    let identity = serde_json::to_vec(&(account, school, profile)).unwrap();
    format!("cache.v2.{:x}.{key}", Sha256::digest(identity))
}

pub fn save(key: &str, data: &str) {
    if let Some(prefixed) = scoped_key(key) {
        day::prefs::set(&prefixed, data);
    }
}

pub fn load(key: &str) -> Option<String> {
    let prefixed = scoped_key(key)?;
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
    load(key).and_then(|j| match serde_json::from_str(&j) {
        Ok(v) => {
            nslog::nslog(&format!("[Cache] Loaded: {key}"));
            Some(v)
        }
        Err(e) => {
            nslog::nslog(&format!("[Cache] Deserialize failed for {key}: {e}"));
            None
        }
    })
}

pub fn has(key: &str) -> bool {
    load(key).is_some()
}

/// Preserve local notes from the pre-namespace format for the last stored
/// profile only. API responses are re-fetched instead of importing ambiguous
/// shared diary data. Keep the original notes blob as a recovery copy.
pub fn migrate_legacy_notes() {
    const MIGRATED: &str = "cache.v2.legacy_notes_migrated";
    if day::prefs::get(MIGRATED).as_deref() == Some("true") {
        return;
    }
    if scoped_key("lesson_notes_map").is_some() && !has("lesson_notes_map") {
        if let Some(notes) = day::prefs::get("cache.lesson_notes_map").filter(|s| !s.is_empty()) {
            save("lesson_notes_map", &notes);
        }
    }
    // Never assign a former account's shared notes to a future login.
    day::prefs::set(MIGRATED, "true");
}

const KNOWN_KEYS: &[&str] = &[
    "all_weeks",
    "week_cache",
    "lessons",
    "bell_times",
    "timetable_days",
    "subjects_teachers",
    "quarter_all_marks",
    "final_marks",
    "user_full_name",
    "user_school_name",
    "lesson_notes_map",
    "time_activities",
    "grade_check.last_marks",
];

pub fn delete(key: &str) {
    if let Some(prefixed) = scoped_key(key) {
        day::prefs::set(&prefixed, "");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_school_and_child_have_separate_keys() {
        let first = key_for("parent", "school", "alice", "lessons");
        assert_ne!(first, key_for("parent", "school", "bob", "lessons"));
        assert_ne!(first, key_for("student", "school", "alice", "lessons"));
        assert_ne!(first, key_for("parent", "other", "alice", "lessons"));
        assert_eq!(first, key_for("parent", "school", "alice", "lessons"));
    }
}

pub fn clear() {
    nslog::nslog("[Cache] Clearing all cache keys");
    for &key in KNOWN_KEYS {
        delete(key);
    }
}
