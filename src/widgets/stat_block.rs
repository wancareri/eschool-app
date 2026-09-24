use crate::app::AppState;
use day::prelude::*;

pub fn render(state: AppState, title: &'static str, value_fn: impl Fn() -> String + 'static) -> impl Piece {
    let s = state;
    column((
        label(move || value_fn())
            .font(Font::Title2)
            .color(move || Color::hex(s.accent_color.get()))
            .align(TextAlign::Center),
        label(title)
            .font(Font::Caption)
            .secondary()
            .align(TextAlign::Center),
    ))
    .align(HAlign::Center)
    .spacing(2.0)
    .padding(Insets { top: 8.0, leading: 8.0, bottom: 8.0, trailing: 8.0 })
    .grow()
}
