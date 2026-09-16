//! Authentication — login, logout, token refresh.
//! Uses secure keychain/keystore for sensitive data.

use crate::app::AppState;
use crate::shared::{nslog, secure};

const TOKEN_KEY: &str = "auth.token";
const REFRESH_KEY: &str = "auth.refresh_token";
const SCHOOL_ID_KEY: &str = "auth.school_id";
const PROFILE_ID_KEY: &str = "auth.profile_id";
const CLASS_ID_KEY: &str = "auth.class_id";
const FULL_NAME_KEY: &str = "auth.full_name";
const SCHOOL_NAME_KEY: &str = "auth.school_name";
const USERNAME_KEY: &str = "auth.username";
const PASSWORD_KEY: &str = "auth.password";
const REMEMBER_KEY: &str = "auth.remember_me";

pub fn login(state: AppState, username: &str, password: &str, remember: bool) {
    state.loading.set(true);
    state.error_msg.set(String::new());

    let auth = eschool_api::auth::Auth::new();
    match auth.login_blocking(username, password) {
        Ok(token) => {
            secure::save(TOKEN_KEY, &token.access_token);
            secure::save(REFRESH_KEY, &token.refresh_token);
            if remember {
                secure::save(USERNAME_KEY, username);
                secure::save(PASSWORD_KEY, password);
                secure::save(REMEMBER_KEY, "true");
            } else {
                secure::delete(USERNAME_KEY);
                secure::delete(PASSWORD_KEY);
                secure::save(REMEMBER_KEY, "false");
            }
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
        SCHOOL_NAME_KEY, USERNAME_KEY, PASSWORD_KEY, REMEMBER_KEY,
    ] {
        secure::delete(key);
    }
    state.is_authenticated.set(false);
    state.remember_me.set(false);
    state.full_name.set(String::new());
    state.school_name.set(String::new());
    state.class_label.set(String::new());
    state.lessons.set(Vec::new());
    state.bell_times.set(Vec::new());
    state.timetable_days.set(Vec::new());
    state.subjects_teachers.set(Vec::new());
}

pub fn save_token(state: AppState, access: &str, refresh: &str) {
    secure::save(TOKEN_KEY, access);
    secure::save(REFRESH_KEY, refresh);
    state.is_authenticated.set(true);
    super::diary::load_all(state);
}

/// Try to refresh the token.
pub fn try_refresh_token() -> Option<String> {
    nslog::nslog("[Auth] Attempting token refresh...");

    if let Some(refresh_str) = secure::load(REFRESH_KEY) {
        match eschool_api::auth::refresh_access_token(&refresh_str) {
            Ok((new_access, new_refresh)) => {
                nslog::nslog("[Auth] Token refresh OK");
                secure::save(TOKEN_KEY, &new_access);
                secure::save(REFRESH_KEY, &new_refresh);
                return Some(new_access);
            }
            Err(e) => {
                nslog::nslog(&format!("[Auth] Refresh failed: {e}"));
            }
        }
    }

    // Try re-login with stored credentials
    let has_credentials = secure::load(REMEMBER_KEY).map(|v| v == "true").unwrap_or(false);
    if has_credentials {
        if let (Some(username), Some(password)) = (secure::load(USERNAME_KEY), secure::load(PASSWORD_KEY)) {
            nslog::nslog("[Auth] Trying re-login with stored credentials...");
            let auth = eschool_api::auth::Auth::new();
            match auth.login_blocking(&username, &password) {
                Ok(token) => {
                    nslog::nslog("[Auth] Re-login OK");
                    secure::save(TOKEN_KEY, &token.access_token);
                    secure::save(REFRESH_KEY, &token.refresh_token);
                    return Some(token.access_token);
                }
                Err(e) => {
                    nslog::nslog(&format!("[Auth] Re-login failed: {e}"));
                }
            }
        }
    }

    nslog::nslog("[Auth] All auth methods failed, need manual login");
    None
}

pub fn get_token() -> Option<String> {
    secure::load(TOKEN_KEY)
}

pub fn get_stored_ids() -> (String, String, String) {
    (
        secure::load(SCHOOL_ID_KEY).unwrap_or_default(),
        secure::load(CLASS_ID_KEY).unwrap_or_default(),
        secure::load(PROFILE_ID_KEY).unwrap_or_default(),
    )
}

pub fn store_ids(
    school_id: &str,
    class_id: &str,
    profile_id: &str,
    full_name: &str,
    school_name: &str,
) {
    secure::save(SCHOOL_ID_KEY, school_id);
    secure::save(CLASS_ID_KEY, class_id);
    secure::save(PROFILE_ID_KEY, profile_id);
    secure::save(FULL_NAME_KEY, full_name);
    secure::save(SCHOOL_NAME_KEY, school_name);
}

/// Start background token refresh — every 55 minutes.
pub fn start_background_refresh() {
    std::thread::spawn(|| {
        nslog::nslog("[Auth] Background refresh started (55 min interval)");
        loop {
            std::thread::sleep(std::time::Duration::from_secs(55 * 60));

            if secure::load(TOKEN_KEY).is_none() {
                nslog::nslog("[Auth] Background refresh: no token, stopping");
                break;
            }

            nslog::nslog("[Auth] Background refresh: refreshing token...");
            match try_refresh_token() {
                Some(_) => nslog::nslog("[Auth] Background refresh: OK"),
                None => {
                    nslog::nslog("[Auth] Background refresh: FAILED, need manual login");
                    break;
                }
            }
        }
    });
}
