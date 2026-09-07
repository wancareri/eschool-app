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
    column((
        native::header::render(
            res::str::settings_title().format(),
            "Настройки приложения",
        ),
        settings_body(),
    ))
    .spacing(12.0)
    .align(HAlign::Leading)
    .padding(16.0)
    .grow()
}
