use crate::core::colors;
use crate::core::state::ESchoolState;
use crate::native;
use crate::res;
use day::prelude::*;

/// Login page — shows either auth form or logged-in profile.
pub(crate) fn login_page() -> impl Piece {
    let state = ESchoolState::ambient();

    column((
        native::header::render(
            res::str::app_title().format(),
            "Авторизация",
        ),
        // ── logged in — show profile ──
        when(
            move || state.is_authenticated.get(),
            move || profile_view(state),
        ),
        // ── not logged in — show auth form ──
        when(
            move || !state.is_authenticated.get(),
            move || auth_form(state),
        ),
    ))
    .spacing(16.0)
    .padding(24.0)
    .grow()
}

// ── profile (logged-in state) ────────────────────────────────────────────

fn profile_view(state: ESchoolState) -> impl Piece {
    column((
        spacer(),
        label("👤")
            .font(Font::LargeTitle)
            .align(TextAlign::Center),
        label(move || state.full_name.get())
            .font(Font::Title2)
            .align(TextAlign::Center),
        label(move || state.school_name.get())
            .font(Font::Body)
            .secondary()
            .align(TextAlign::Center),
        when(
            move || !state.class_label.get().is_empty(),
            move || label(move || format!("Класс: {}", state.class_label.get()))
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center),
        ),
        when(
            move || state.loading.get(),
            || label("Загрузка данных…")
                .font(Font::Caption)
                .secondary()
                .align(TextAlign::Center),
        ),
        when(
            move || !state.error_msg.get().is_empty(),
            move || label(move || state.error_msg.get())
                .font(Font::Caption)
                .color(colors::ERROR)
                .align(TextAlign::Center),
        ),
        spacer(),
        // Refresh button
        native::button::render("Обновить данные", "arrow.clockwise"),
        // Logout button
        native::button::render("Выйти", "rectangle.portrait.and.arrow.right"),
    ))
    .spacing(12.0)
    .align(HAlign::Center)
    .grow()
}

// ── auth form (not logged in) ────────────────────────────────────────────

fn auth_form(state: ESchoolState) -> impl Piece {
    let username = Signal::new(String::new());
    let password = Signal::new(String::new());
    let error_local = Signal::new(String::new());
    let logging_in = Signal::new(false);

    column((
        spacer(),
        label("Вход в электронный дневник")
            .font(Font::Title2)
            .align(TextAlign::Center),
        label("ИЯ РИОС — сервер аутентификации")
            .font(Font::Subheadline)
            .secondary()
            .align(TextAlign::Center),
        // Username field
        text_field(username)
            .placeholder("Имя пользователя")
            .id("username-field"),
        // Password field
        text_field(password)
            .placeholder("Пароль")
            .id("password-field"),
        // Login button
        button(move || {
            if logging_in.get() { "Вход…" } else { "Войти" }
        })
        .action(move || {
            let u = username.get_untracked();
            let p = password.get_untracked();
            if u.trim().is_empty() {
                error_local.set("Введите имя пользователя".into());
                return;
            }
            if p.is_empty() {
                error_local.set("Введите пароль".into());
                return;
            }
            error_local.set(String::new());
            logging_in.set(true);
            state.login_with_password(&u, &p);
            logging_in.set(false);
        })
        .id("login-btn"),
        // Error display
        when(
            move || {
                let local_err = error_local.get();
                let state_err = state.error_msg.get();
                let is_loading = state.loading.get();
                (!local_err.is_empty() || !state_err.is_empty()) && !is_loading
            },
            move || {
                let local_err = error_local.get();
                let state_err = state.error_msg.get();
                let msg = if !local_err.is_empty() { local_err } else { state_err };
                label(msg)
                    .font(Font::Caption)
                    .color(colors::ERROR)
                    .align(TextAlign::Center)
            }
        ),
        // Loading indicator
        when(
            move || state.loading.get(),
            || label("Выполняется вход…")
                .font(Font::Caption)
                .secondary()
                .align(TextAlign::Center),
        ),
        spacer(),
    ))
    .spacing(12.0)
    .align(HAlign::Center)
    .grow()
}
