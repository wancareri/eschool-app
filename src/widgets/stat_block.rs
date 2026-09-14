use crate::shared::colors;
use day::prelude::*;

pub fn render(title: &'static str, value_fn: impl Fn() -> String + 'static) -> impl Piece {
    column((
        label(move || value_fn())
            .font(Font::Title2)
            .color(colors::PRIMARY)
            .align(TextAlign::Center),
        label(title)
            .font(Font::Caption)
            .secondary()
            .align(TextAlign::Center),
    ))
    .spacing(2.0)
    .padding(Insets { top: 8.0, leading: 8.0, bottom: 8.0, trailing: 8.0 })
    .grow()
}
