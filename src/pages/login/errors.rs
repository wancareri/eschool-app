use super::*;

pub(super) struct AuthErrorInfo {
    title: String,
    message: String,
}

pub(super) fn auth_error_code(raw: &str) -> &'static str {
    let lower = raw.to_lowercase();
    if lower.contains("введите логин") || lower.contains("введите имя") {
        "empty_login"
    } else if lower.contains("введите пароль") {
        "empty_password"
    } else if lower.contains("403") || lower.contains("заблокир") || lower.contains("forbidden")
    {
        "forbidden"
    } else if lower.contains("таймаут") || lower.contains("timeout") || lower.contains("timed out")
    {
        "timeout"
    } else if ["500", "502", "503", "504", "internal server"]
        .iter()
        .any(|s| lower.contains(s))
    {
        "server"
    } else if [
        "error sending request",
        "не удалось подключиться",
        "dns",
        "connection refused",
        "failed to connect",
        "network",
        "connection reset",
    ]
    .iter()
    .any(|s| lower.contains(s))
    {
        "network"
    } else if [
        "401",
        "неверн",
        "unauthorized",
        "invalid credentials",
        "invalid password",
        "invalid login",
    ]
    .iter()
    .any(|s| lower.contains(s))
    {
        "credentials"
    } else {
        "generic"
    }
}

pub(super) fn parse_auth_error(raw: &str) -> AuthErrorInfo {
    let (title, message) = match auth_error_code(raw) {
        "empty_login" => (
            res::str::auth_error_login_title().format(),
            res::str::auth_error_login_message().format(),
        ),
        "empty_password" => (
            res::str::auth_error_password_title().format(),
            res::str::auth_error_password_message().format(),
        ),
        "forbidden" => (
            res::str::auth_error_forbidden_title().format(),
            res::str::auth_error_forbidden_message().format(),
        ),
        "timeout" => (
            res::str::auth_error_timeout_title().format(),
            res::str::auth_error_timeout_message().format(),
        ),
        "server" => (
            res::str::auth_error_server_title().format(),
            res::str::auth_error_server_message().format(),
        ),
        "network" => (
            res::str::auth_error_network_title().format(),
            res::str::auth_error_network_message().format(),
        ),
        "credentials" => (
            res::str::auth_error_credentials_title().format(),
            res::str::auth_error_credentials_message().format(),
        ),
        _ => (
            res::str::auth_error_generic_title().format(),
            res::str::auth_error_generic_message().format(),
        ),
    };
    AuthErrorInfo { title, message }
}

pub fn error_notice(raw: &str) -> crate::components::notice::Notice {
    let error = parse_auth_error(raw);
    crate::components::notice::Notice::error(error.title, error.message)
}
