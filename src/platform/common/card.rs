use day::prelude::*;

pub fn render(title: impl Into<String>, value: impl Into<String>, _icon: impl Into<String>) -> AnyPiece {
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
