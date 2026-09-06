use day::prelude::*;

pub fn render(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
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
