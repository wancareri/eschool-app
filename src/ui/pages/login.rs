use crate::core::colors;
use crate::core::state::ESchoolState;
use crate::native;
use crate::res;
use day::prelude::*;

pub(crate) fn login_page() -> impl Piece {
    let state = ESchoolState::ambient();

    column((
        when(
            move || state.is_authenticated.get(),
            move || profile_view(state),
        ),
        when(
            move || !state.is_authenticated.get(),
            move || auth_form(state),
        ),
    ))
    .spacing(0.0)
    .grow()
}

// ── profile ──────────────────────────────────────────────────────────────

fn profile_view(state: ESchoolState) -> impl Piece {
    column((
        column((
            label(move || res::str::settings_title().format())
                .font(Font::LargeTitle),
            label("Профиль")
                .font(Font::Subheadline)
                .secondary(),
        ))
        .spacing(6.0)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 12.0, trailing: 20.0 }),
        spacer(),
        column((
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
        ))
        .spacing(8.0)
        .align(HAlign::Center),
        spacer(),
        column((
            native::button::render("Обновить данные", "arrow.clockwise"),
            native::button::render("Выйти", "rectangle.portrait.and.arrow.right"),
        ))
        .spacing(8.0)
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 20.0, trailing: 20.0 }),
    ))
    .spacing(0.0)
    .grow()
}

// ── auth form ────────────────────────────────────────────────────────────

fn auth_form(state: ESchoolState) -> impl Piece {
    let username = Signal::new(String::new());
    let password = Signal::new(String::new());
    let error_local = Signal::new(String::new());
    let logging_in = Signal::new(false);

    column((
        column((
            label(move || res::str::app_title().format())
                .font(Font::LargeTitle),
            label("Вход в дневник")
                .font(Font::Subheadline)
                .secondary(),
        ))
        .spacing(6.0)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 12.0, trailing: 20.0 }),
        spacer(),
        column((
            text_field(username)
                .placeholder("Имя пользователя")
                .id("username-field"),
            text_field(password)
                .placeholder("Пароль")
                .secure()
                .id("password-field"),
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
            when(
                move || state.loading.get(),
                || label("Выполняется вход…")
                    .font(Font::Caption)
                    .secondary()
                    .align(TextAlign::Center),
            ),
        ))
        .spacing(12.0)
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 0.0, trailing: 20.0 }),
        spacer(),
    ))
    .spacing(0.0)
    .grow()
}
