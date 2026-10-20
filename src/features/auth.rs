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
    let set_loading = state.loading.setter();
    let set_error = state.error_msg.setter();
    let set_authenticated = state.is_authenticated.setter();
    let set_setup_active = state.setup_new_password_active.setter();
    let set_setup_data = state.setup_temp_password_data.setter();

    let u = username.to_string();
    let p = password.to_string();
    let r = remember;

    set_loading.set(true);
    set_error.set(String::new());

    std::thread::spawn(move || {
        let auth = eschool_api::auth::Auth::new();

        // If username consists solely of digits, try parent login first to detect initial password setup
        let is_digits = !u.is_empty() && u.chars().all(|c| c.is_ascii_digit());
        if is_digits {
            if let Ok(res) = auth.login_parent_with_status_blocking(&u, &p) {
                match res {
                    eschool_api::auth::ParentLoginResult::RequiresNewPassword { token, parent_uuid, temp_password } => {
                        secure::save(TOKEN_KEY, &token.access_token);
                        secure::save(REFRESH_KEY, &token.refresh_token);
                        let old_pwd = temp_password.unwrap_or_else(|| p.clone());
                        day::reactive::on_main(move || {
                            crate::shared::keyboard::dismiss();
                            set_loading.set(false);
                            set_setup_active.set(true);
                            set_setup_data.set(Some((parent_uuid, old_pwd, u)));
                        });
                        return;
                    }
                    eschool_api::auth::ParentLoginResult::Success(token) => {
                        secure::save(TOKEN_KEY, &token.access_token);
                        secure::save(REFRESH_KEY, &token.refresh_token);
                        if r {
                            secure::save(USERNAME_KEY, &u);
                            secure::save(PASSWORD_KEY, &p);
                            secure::save(REMEMBER_KEY, "true");
                        } else {
                            secure::delete(USERNAME_KEY);
                            secure::delete(PASSWORD_KEY);
                            secure::save(REMEMBER_KEY, "false");
                        }
                        day::reactive::on_main(move || {
                            crate::shared::keyboard::dismiss();
                            set_loading.set(false);
                            nslog::nslog("[Auth] Parent login OK, setting is_authenticated=true");
                            set_authenticated.set(true);
                        });
                        return;
                    }
                }
            }
        }

        match auth.login_blocking(&u, &p) {
            Ok(token) => {
                secure::save(TOKEN_KEY, &token.access_token);
                secure::save(REFRESH_KEY, &token.refresh_token);
                if r {
                    secure::save(USERNAME_KEY, &u);
                    secure::save(PASSWORD_KEY, &p);
                    secure::save(REMEMBER_KEY, "true");
                } else {
                    secure::delete(USERNAME_KEY);
                    secure::delete(PASSWORD_KEY);
                    secure::save(REMEMBER_KEY, "false");
                }
                day::reactive::on_main(move || {
                    crate::shared::keyboard::dismiss();
                    set_loading.set(false);
                    nslog::nslog("[Auth] Login OK, setting is_authenticated=true on main thread");
                    // Set is_authenticated LAST — triggers the reactive watch in window_shell
                    // which will call load_all.
                    set_authenticated.set(true);
                });
            }
            Err(e) => {
                day::reactive::on_main(move || {
                    set_error.set(format!("Ошибка входа: {e}"));
                    set_loading.set(false);
                });
            }
        }
    });
}

pub fn complete_parent_new_password(
    state: AppState,
    parent_uuid: &str,
    old_password: &str,
    new_password: &str,
    username: &str,
    remember: bool,
) {
    let set_loading = state.loading.setter();
    let set_error = state.error_msg.setter();
    let set_authenticated = state.is_authenticated.setter();
    let set_setup_active = state.setup_new_password_active.setter();
    let set_setup_data = state.setup_temp_password_data.setter();

    if let Err(e) = eschool_api::auth::validate_parent_password(new_password) {
        set_error.set(e);
        return;
    }

    set_loading.set(true);
    set_error.set(String::new());

    let puuid = parent_uuid.to_string();
    let old_p = old_password.to_string();
    let new_p = new_password.to_string();
    let u = username.to_string();
    let r = remember;

    std::thread::spawn(move || {
        let token = match get_token() {
            Some(t) => t,
            None => {
                day::reactive::on_main(move || {
                    set_error.set("Сессия истекла. Пожалуйста, начните вход заново.".into());
                    set_loading.set(false);
                });
                return;
            }
        };

        let client = eschool_api::client::blocking::build_client(&token);
        match eschool_api::client::blocking::change_parent_temp_password(&client, &puuid, &old_p, &new_p) {
            Ok(()) => {
                let auth = eschool_api::auth::Auth::new();
                match auth.login_parent_blocking(&u, &new_p) {
                    Ok(new_token) => {
                        secure::save(TOKEN_KEY, &new_token.access_token);
                        secure::save(REFRESH_KEY, &new_token.refresh_token);
                        if r {
                            secure::save(USERNAME_KEY, &u);
                            secure::save(PASSWORD_KEY, &new_p);
                            secure::save(REMEMBER_KEY, "true");
                        }
                        day::reactive::on_main(move || {
                            crate::shared::keyboard::dismiss();
                            set_setup_active.set(false);
                            set_setup_data.set(None);
                            set_loading.set(false);
                            set_authenticated.set(true);
                        });
                    }
                    Err(e) => {
                        day::reactive::on_main(move || {
                            set_error.set(format!("Пароль изменён, но ошибка входа: {e}"));
                            set_loading.set(false);
                        });
                    }
                }
            }
            Err(e) => {
                day::reactive::on_main(move || {
                    set_error.set(format!("Не удалось установить пароль: {e}"));
                    set_loading.set(false);
                });
            }
        }
    });
}

pub fn logout(state: AppState) {
    logout_keys();
    state.is_authenticated.set(false);
    state.remember_me.set(false);
    state.full_name.set(String::new());
    state.school_name.set(String::new());
    state.class_label.set(String::new());
    state.is_parent.set(false);
    state.parent_children.set(Vec::new());
    state.active_child_index.set(0);
    state.setup_new_password_active.set(false);
    state.setup_temp_password_data.set(None);
    state.lessons.set(Vec::new());
    state.bell_times.set(Vec::new());
    state.timetable_days.set(Vec::new());
    state.subjects_teachers.set(Vec::new());
    state.final_marks.set(crate::app::FinalMarks::new());
}

pub fn logout_keys() {
    // If biometric is enabled, preserve username/password for biometric re-login
    let keep_creds = crate::shared::biometric::is_enabled();
    for key in [
        TOKEN_KEY, REFRESH_KEY, SCHOOL_ID_KEY, PROFILE_ID_KEY, CLASS_ID_KEY, FULL_NAME_KEY,
        SCHOOL_NAME_KEY, REMEMBER_KEY,
    ] {
        secure::delete(key);
    }
    if !keep_creds {
        secure::delete(USERNAME_KEY);
        secure::delete(PASSWORD_KEY);
    }
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

pub fn has_stored_credentials() -> bool {
    secure::load(USERNAME_KEY).is_some() && secure::load(PASSWORD_KEY).is_some()
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
