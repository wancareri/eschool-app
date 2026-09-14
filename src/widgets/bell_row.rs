use crate::app::AppState;
use crate::shared::colors;
use day::prelude::*;

pub fn render(state: AppState, number: u32) -> impl Piece {
    let start = move || {
        state.bell_times.get()
            .iter()
            .find(|b| b.number == number)
            .map(|b| b.start_time.get(..5).unwrap_or(&b.start_time).to_string())
            .unwrap_or_default()
    };
    let end = move || {
        state.bell_times.get()
            .iter()
            .find(|b| b.number == number)
            .map(|b| b.end_time.get(..5).unwrap_or(&b.end_time).to_string())
            .unwrap_or_default()
    };
    row((
        label(format!("{}", number))
            .font(Font::Title3)
            .color(colors::PRIMARY)
            .align(TextAlign::Center),
        column((
            label(format!("{} урок", number))
                .font(Font::Body),
            label(move || format!("{} — {}", start(), end()))
                .font(Font::Caption)
                .secondary(),
        ))
        .spacing(2.0)
        .align(HAlign::Leading)
        .grow(),
    ))
    .spacing(12.0)
    .padding(Insets { top: 8.0, leading: 16.0, bottom: 8.0, trailing: 20.0 })
}
