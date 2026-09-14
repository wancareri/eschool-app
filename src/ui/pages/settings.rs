use crate::core::state::ESchoolState;
use crate::native;
use crate::res;
use day::prelude::*;

/// Appearance and language, from `day-piece-settings`.
pub(crate) fn settings_body() -> impl Piece {
    form((day_piece_settings::settings_sections(
        crate::THEME_KEY,
        crate::LOCALE_KEY,
        res::locales::ALL,
    ),))
}

/// Settings page with navigation header
pub(crate) fn settings_page() -> impl Piece {
    let state = ESchoolState::ambient();

    column((
        column((
            label(move || res::str::settings_title().format())
                .font(Font::LargeTitle),
            label("Настройки приложения")
                .font(Font::Subheadline),
        ))
        .spacing(4.0)
        .padding(16.0),
        // ── profile info ────────────────────────────────────────────────
        when(
            move || state.is_authenticated.get(),
            move || {
                column((
                    label(move || state.full_name.get())
                        .font(Font::Title3),
                    label(move || state.school_name.get())
                        .font(Font::Body).secondary(),
                    when(
                        move || !state.class_label.get().is_empty(),
                        move || label(move || format!("Класс: {}", state.class_label.get()))
                            .font(Font::Body).secondary(),
                    ),
                ))
                .spacing(4.0)
                .align(HAlign::Leading)
                .padding(16.0)
                .grow()
            },
        ),
        // ── appearance & language ──
        settings_body(),
        // ── logout ──────────────────────────────────────────────────────
        when(
            move || state.is_authenticated.get(),
            move || column((
                spacer(),
                button("Выйти из аккаунта")
                    .action(move || state.logout())
                    .id("logout-btn"),
                spacer(),
            )).grow(),
        ),
    ))
    .spacing(12.0)
    .align(HAlign::Leading)
    .padding(16.0)
    .grow()
}
