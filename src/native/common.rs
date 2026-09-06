//! Shared fallback components for unsupported platforms.

use day::prelude::*;

pub fn glass_header(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
    let title_str = title.into();
    let subtitle_str = subtitle.into();
    column((
        label(title_str.clone())
            .font(Font::LargeTitle),
        label(subtitle_str.clone())
            .font(Font::Subheadline),
    ))
    .spacing(4.0)
    .padding(16.0)
    .any()
}

pub fn glass_card(title: impl Into<String>, value: impl Into<String>, _icon: impl Into<String>) -> AnyPiece {
    let title_str = title.into();
    let value_str = value.into();
    column((
        label(value_str.clone())
            .font(Font::Title),
        label(title_str.clone())
            .font(Font::Caption),
    ))
    .spacing(4.0)
    .padding(12.0)
    .any()
}

pub fn glass_button(title: impl Into<String>, _icon: impl Into<String>) -> AnyPiece {
    let title_str = title.into();
    button(title_str).any()
}
