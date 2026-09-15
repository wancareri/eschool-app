//! Authentication — login, logout, token refresh.
//! Wraps eschool_api::auth with day::prefs storage.

use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::shared::nslog;

const TOKEN_KEY: &str = "auth.token";
const REFRESH_KEY: &str = "auth.refresh_token";
const SCHOOL_ID_KEY: &str = "auth.school_id";
const PROFILE_ID_KEY: &str = "auth.profile_id";
const CLASS_ID_KEY: &str = "auth.class_id";
const FULL_NAME_KEY: &str = "auth.full_name";
const SCHOOL_NAME_KEY: &str = "auth.school_name";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: Option<u64>,
}

pub fn login(state: AppState, username: &str, password: &str) {
    state.loading.set(true);
    state.error_msg.set(String::new());

    let auth = eschool_api::auth::Auth::new();
    match auth.login_blocking(username, password) {
        Ok(token) => {
            day::prefs::set(TOKEN_KEY, &token.access_token);
            day::prefs::set(REFRESH_KEY, &token.refresh_token);
            state.is_authenticated.set(true);
            state.loading.set(false);
            super::diary::load_all(state);
        }
        Err(e) => {
            state.error_msg.set(format!("Ошибка входа: {e}"));
            state.loading.set(false);
        }
    }
}

pub fn logout(state: AppState) {
    for key in [
        TOKEN_KEY, REFRESH_KEY, SCHOOL_ID_KEY, PROFILE_ID_KEY, CLASS_ID_KEY, FULL_NAME_KEY,
        SCHOOL_NAME_KEY,
    ] {
        day::prefs::set(key, "");
    }
    state.is_authenticated.set(false);
    state.full_name.set(String::new());
    state.school_name.set(String::new());
    state.class_label.set(String::new());
    state.lessons.set(Vec::new());
    state.bell_times.set(Vec::new());
    state.timetable_days.set(Vec::new());
    state.subjects_teachers.set(Vec::new());
}

pub fn save_token(state: AppState, access: &str, refresh: &str) {
    day::prefs::set(TOKEN_KEY, access);
    day::prefs::set(REFRESH_KEY, refresh);
    state.is_authenticated.set(true);
    super::diary::load_all(state);
}

/// Try to refresh the token. Falls back to full re-login if refresh fails.
pub fn try_refresh_token() -> Option<String> {
    let refresh_str = day::prefs::get(REFRESH_KEY).filter(|r| !r.is_empty())?;
    nslog::nslog("[Auth] Attempting token refresh...");

    // Try refresh first
    match eschool_api::auth::refresh_access_token(&refresh_str) {
        Ok((new_access, new_refresh)) => {
            nslog::nslog("[Auth] Token refresh OK");
            day::prefs::set(TOKEN_KEY, &new_access);
            day::prefs::set(REFRESH_KEY, &new_refresh);
            return Some(new_access);
        }
        Err(e) => {
            nslog::nslog(&format!("[Auth] Token refresh failed: {e}"));
        }
    }

    None
}

pub fn get_token() -> Option<String> {
    day::prefs::get(TOKEN_KEY).filter(|t| !t.is_empty())
}

pub fn get_stored_ids() -> (String, String, String) {
    (
        day::prefs::get(SCHOOL_ID_KEY).unwrap_or_default(),
        day::prefs::get(CLASS_ID_KEY).unwrap_or_default(),
        day::prefs::get(PROFILE_ID_KEY).unwrap_or_default(),
    )
}

pub fn store_ids(
    school_id: &str,
    class_id: &str,
    profile_id: &str,
    full_name: &str,
    school_name: &str,
) {
    day::prefs::set(SCHOOL_ID_KEY, school_id);
    day::prefs::set(CLASS_ID_KEY, class_id);
    day::prefs::set(PROFILE_ID_KEY, profile_id);
    day::prefs::set(FULL_NAME_KEY, full_name);
    day::prefs::set(SCHOOL_NAME_KEY, school_name);
}
