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

/// The same body as a navigable section, for the platforms with no menu bar.
pub(crate) fn settings_page() -> impl Piece {
    column((
        label(res::str::nav_settings())
            .font(Font::Title)
            .id("settings-title"),
        settings_body(),
    ))
    .spacing(12.0)
    .align(HAlign::Leading)
    .padding(16.0)
}
