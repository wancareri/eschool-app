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
            || label("Обновление данных…")
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
    let token_input = Signal::new(String::new());

    column((
        spacer(),
        label("Вход в электронный дневник")
            .font(Font::Title2)
            .align(TextAlign::Center),
        label("Введите токен авторизации, полученный на diary.e-schools.by")
            .font(Font::Body)
            .secondary()
            .align(TextAlign::Center),
        // Token input
        text_field(token_input)
            .placeholder("Вставьте токен…")
            .id("token-field"),
        // Login button
        button("Войти")
            .action(move || {
                let t = token_input.get_untracked();
                if !t.is_empty() {
                    state.save_token(&t, "");
                }
            })
            .id("login-btn"),
        when(
            move || !state.error_msg.get().is_empty(),
            move || label(move || state.error_msg.get())
                .font(Font::Caption)
                .color(colors::ERROR)
                .align(TextAlign::Center),
        ),
        spacer(),
    ))
    .spacing(16.0)
    .align(HAlign::Center)
    .grow()
}
