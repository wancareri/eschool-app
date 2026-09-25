use crate::app::AppState;
use day::prelude::*;

pub fn render(state: AppState, title: &'static str, value_fn: impl Fn() -> String + 'static) -> impl Piece {
    let s = state;
    column((
        // .align(TextAlign) is a no-op on the UIKit back-end, so the text centers the
        // SwiftUI way: the label hugs and the column centers the hug box. The labels
        // must cross-grow or the whole column measures to content and packs left
        // inside its flex share of the row.
        label(move || value_fn())
            .font(Font::Title2)
            .color(move || Color::hex(s.accent_color.get()))
            .align(TextAlign::Center)
            .grow_w(),
        label(title)
            .font(Font::Caption)
            .secondary()
            .align(TextAlign::Center)
            .grow_w(),
    ))
    .align(HAlign::Center)
    .spacing(2.0)
    .padding(Insets { top: 8.0, leading: 8.0, bottom: 8.0, trailing: 8.0 })
    .grow_w()
}
