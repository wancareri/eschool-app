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
    let error_local = Signal::new(String::new());

    column((
        spacer(),
        label("Вход в электронный дневник")
            .font(Font::Title2)
            .align(TextAlign::Center),
        // Instructions
        column((
            label("Как получить токен:")
                .font(Font::Headline)
                .align(TextAlign::Center),
            label("1. Откройте diary.e-schools.by и войдите")
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center),
            label("2. Откройте DevTools (F12) → Network")
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center),
            label("3. Нажмите любую кнопку в дневнике")
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center),
            label("4. Найдите запрос к diary.e-schools.by")
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center),
            label("5. Скопируйте заголовок Authorization")
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center),
            label("   (это JWT токен, длинная строка)")
                .font(Font::Caption)
                .secondary()
                .align(TextAlign::Center),
        ))
        .spacing(2.0)
        .padding(Insets { top: 8.0, leading: 24.0, bottom: 8.0, trailing: 24.0 }),
        // Token input
        text_field(token_input)
            .placeholder("Вставьте JWT токен…")
            .id("token-field"),
        // Login button
        button("Войти")
            .action(move || {
                let t = token_input.get_untracked();
                if t.trim().is_empty() {
                    error_local.set("Введите токен".into());
                    return;
                }
                error_local.set(String::new());
                state.save_token(t.trim(), "");
            })
            .id("login-btn"),
        // Error display
        when(
            move || {
                let local_err = error_local.get();
                let state_err = state.error_msg.get();
                !local_err.is_empty() || !state_err.is_empty()
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
        spacer(),
    ))
    .spacing(12.0)
    .align(HAlign::Center)
    .grow()
}
