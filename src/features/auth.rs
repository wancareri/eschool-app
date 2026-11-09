//! Authentication. Credential changes are committed only for the current session.
use crate::app::AppState;
use crate::shared::{nslog, secure, session};
use eschool_api::auth::{Auth, ParentLoginResult, Token};
use std::sync::Mutex;

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
const ACCOUNT_KEY: &str = "auth.account_id";
// Never hold the storage lock during a network call or UI callback.
static TOKEN_STORE: Mutex<()> = Mutex::new(());
static REFRESH_LOCK: Mutex<()> = Mutex::new(());

fn persist_tokens(token: &Token) {
    let _lock = TOKEN_STORE.lock().unwrap();
    secure::save(TOKEN_KEY, &token.access_token);
    secure::save(REFRESH_KEY, &token.refresh_token);
}

pub fn ensure_account_id() {
    if secure::load(ACCOUNT_KEY).is_none() {
        if let Some(token) = get_token() {
            use sha2::{Digest, Sha256};
            let id = eschool_api::auth::extract_token_usi(&token)
                .unwrap_or_else(|| format!("{:x}", Sha256::digest(token.as_bytes())));
            secure::save(ACCOUNT_KEY, &id);
        }
    }
}

fn remember_user(state: AppState, username: &str, remember: bool) {
    if remember {
        secure::save(USERNAME_KEY, username);
    } else {
        secure::delete(USERNAME_KEY);
    }
    secure::save(REMEMBER_KEY, if remember { "true" } else { "false" });
    secure::delete(PASSWORD_KEY);
    state.remember_me.set(remember);
}

fn finish_login(state: AppState, token: Token, username: &str, remember: bool) {
    persist_tokens(&token);
    ensure_account_id();
    remember_user(state, username, remember);
    state.setup_new_password_active.set(false);
    state.setup_temp_password_data.set(None);
    state.loading.set(false);
    // Leave filled native inputs intact until the auth hierarchy is removed:
    // that removal is UIKit's signal that the credential submission succeeded.
    state.is_authenticated.set(true);
    let ticket = session::ticket();
    ticket.publish(|state| {
        if state.is_authenticated.get() {
            crate::pages::login::reset_auth_flow();
        }
    });
}

pub fn login(state: AppState, username: &str, password: &str, remember: bool) {
    session::invalidate();
    let ticket = session::ticket();
    super::diary::clear_data(state);
    // A password login starts a new account context; no old student IDs survive.
    logout_keys();
    state.loading.set(true);
    let username = username.to_string();
    let password = password.to_string();
    std::thread::spawn(move || {
        let auth = Auth::new();
        let numeric = !username.is_empty() && username.chars().all(|c| c.is_ascii_digit());
        let result = if numeric {
            auth.login_parent_with_status_blocking(&username, &password)
                .or_else(|parent_error| {
                    auth.login_student_blocking(&username, &password)
                        .map(ParentLoginResult::Success)
                        .map_err(|_| parent_error)
                })
        } else {
            auth.login_student_blocking(&username, &password)
                .map(ParentLoginResult::Success)
                .or_else(|student_error| {
                    auth.login_parent_with_status_blocking(&username, &password)
                        .map_err(|_| student_error)
                })
        };
        ticket.publish(move |state| match result {
            Ok(ParentLoginResult::Success(token)) => {
                finish_login(state, token, &username, remember)
            }
            Ok(ParentLoginResult::RequiresNewPassword {
                token,
                parent_uuid,
                temp_password,
            }) => {
                persist_tokens(&token);
                ensure_account_id();
                remember_user(state, &username, remember);
                state.setup_temp_password_data.set(Some((
                    parent_uuid,
                    temp_password.unwrap_or(password),
                    username,
                )));
                state.setup_new_password_active.set(true);
                state.loading.set(false);
                crate::shared::keyboard::dismiss();
            }
            Err(e) => {
                state.error_msg.set(format!("Ошибка входа: {e}"));
                state.loading.set(false);
            }
        });
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
    if let Err(e) = eschool_api::auth::validate_parent_password(new_password) {
        state.error_msg.set(e);
        return;
    }
    let Some(token) = get_token() else {
        state
            .error_msg
            .set("Сессия истекла. Начните вход заново.".into());
        return;
    };
    let ticket = session::ticket();
    state.loading.set(true);
    state.error_msg.set(String::new());
    let (parent, old, new, username) = (
        parent_uuid.to_string(),
        old_password.to_string(),
        new_password.to_string(),
        username.to_string(),
    );
    std::thread::spawn(move || {
        let client = eschool_api::client::blocking::build_client(&token);
        let result = eschool_api::client::blocking::change_parent_temp_password(
            &client, &parent, &old, &new,
        );
        match result {
            Ok(()) => {
                let result = Auth::new().login_parent_blocking(&username, &new);
                ticket.publish(move |state| match result {
                    Ok(token) => finish_login(state, token, &username, remember),
                    Err(e) => {
                        // The old temporary password is no longer valid. Return to
                        // login instead of inviting the user to submit it again.
                        logout(state);
                        state
                            .error_msg
                            .set(format!("Пароль изменён. Войдите с новым паролем: {e}"));
                    }
                });
            }
            Err(e) => ticket.publish(move |state| {
                state
                    .error_msg
                    .set(format!("Не удалось установить пароль: {e}"));
                state.loading.set(false);
            }),
        }
    });
}

pub fn change_logged_in_parent_password(
    state: AppState,
    old_password: &str,
    new_password: &str,
    on_success: impl Fn() + Send + 'static,
) {
    if let Err(e) = eschool_api::auth::validate_parent_password(new_password) {
        state.error_msg.set(e);
        return;
    }
    let Some(token) = get_token() else {
        state
            .error_msg
            .set("Сессия истекла. Войдите заново.".into());
        return;
    };
    let Some(parent) = eschool_api::auth::extract_token_usi(&token) else {
        state
            .error_msg
            .set("Не удалось определить ID родителя.".into());
        return;
    };
    let ticket = session::ticket();
    let username = secure::load(USERNAME_KEY);
    let (old, new) = (old_password.to_string(), new_password.to_string());
    state.loading.set(true);
    state.error_msg.set(String::new());
    std::thread::spawn(move || {
        let client = eschool_api::client::blocking::build_client(&token);
        let result =
            eschool_api::client::blocking::change_parent_password(&client, &parent, &old, &new);
        let renewed = if result.is_ok() {
            username.and_then(|u| Auth::new().login_parent_blocking(&u, &new).ok())
        } else {
            None
        };
        ticket.publish(move |state| {
            state.loading.set(false);
            match result {
                Ok(()) => {
                    if let Some(token) = renewed {
                        persist_tokens(&token);
                    }
                    secure::delete(PASSWORD_KEY);
                    on_success();
                }
                Err(e) => state
                    .error_msg
                    .set(format!("Не удалось сменить пароль: {e}")),
            }
        });
    });
}

pub fn logout(state: AppState) {
    session::invalidate();
    // Clear the namespace before deleting the keys identifying it.
    crate::shared::cache::clear();
    logout_keys();
    day::reactive::batch(|| {
        super::diary::clear_data(state);
        crate::shared::keyboard::dismiss();
        crate::pages::login::reset_auth_flow();
        state.error_msg.set(String::new());
        state.notice_route.set(None);
        state.notice.set(None);
        state.show_network_modal.set(None);
        state.loading.set(false);
        state.settings_tab.set(0);
        state.remember_me.set(false);
        state.biometric_ok.set(false);
        state.pin_lock_active.set(false);
        state.pin_setup_active.set(false);
        state.pin_setup_route.set(None);
        state.pin_input.set(String::new());
        state.pin_error.set(false);
        state.is_parent.set(false);
        state.parent_children.set(Vec::new());
        state.active_child_index.set(0);
        state.setup_new_password_active.set(false);
        state.setup_temp_password_data.set(None);
        state.current_section.set(crate::Section::Diary);
        state.is_authenticated.set(false);
    });
}

pub fn logout_keys() {
    let _lock = TOKEN_STORE.lock().unwrap();
    for key in [
        TOKEN_KEY,
        REFRESH_KEY,
        SCHOOL_ID_KEY,
        PROFILE_ID_KEY,
        CLASS_ID_KEY,
        FULL_NAME_KEY,
        SCHOOL_NAME_KEY,
        REMEMBER_KEY,
        PASSWORD_KEY,
        ACCOUNT_KEY,
    ] {
        secure::delete(key);
    }
    if !crate::shared::biometric::is_enabled() {
        secure::delete(USERNAME_KEY);
    }
}

pub fn save_token(state: AppState, access: &str, refresh: &str) {
    persist_tokens(&Token {
        access_token: access.into(),
        refresh_token: refresh.into(),
    });
    ensure_account_id();
    if state.is_authenticated.get() {
        super::diary::load_all(state);
    } else {
        state.is_authenticated.set(true);
    }
}

/// Serialize refresh requests and refuse to resurrect logged-out credentials.
pub fn try_refresh_token() -> Option<String> {
    let _refresh = REFRESH_LOCK.lock().unwrap();
    let (old_refresh, account) = {
        let _store = TOKEN_STORE.lock().unwrap();
        (secure::load(REFRESH_KEY)?, secure::load(ACCOUNT_KEY))
    };
    let result = eschool_api::auth::refresh_access_token(&old_refresh);
    match result {
        Ok((access, refresh)) => {
            let _store = TOKEN_STORE.lock().unwrap();
            if secure::load(REFRESH_KEY).as_deref() != Some(&old_refresh)
                || secure::load(ACCOUNT_KEY) != account
            {
                return None;
            }
            secure::save(TOKEN_KEY, &access);
            secure::save(REFRESH_KEY, &refresh);
            Some(access)
        }
        Err(e) => {
            nslog::nslog(&format!("[Auth] Refresh failed: {e}"));
            None
        }
    }
}

pub fn get_token() -> Option<String> {
    secure::load(TOKEN_KEY)
}

pub fn has_stored_credentials() -> bool {
    secure::load(USERNAME_KEY).is_some() && secure::load(REFRESH_KEY).is_some()
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

static BG_REFRESH_STARTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// The process-wide worker survives logout and temporary network failures.
/// It reads the current credentials each time, so a later login needs no restart.
pub fn start_background_refresh() {
    use std::sync::atomic::Ordering;
    if BG_REFRESH_STARTED.swap(true, Ordering::AcqRel) {
        return;
    }
    std::thread::spawn(|| {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(55 * 60));
            if get_token().is_some() {
                let _ = try_refresh_token();
            }
        }
    });
}
