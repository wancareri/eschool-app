use crate::app::AppState;
use crate::shared::colors;
use day::prelude::*;

pub fn render(state: AppState, number: u32) -> impl Piece {
    let start = move || {
        state.bell_times.get()
            .iter()
            .find(|b| b.number == number)
            .map(|b| strip_seconds(&b.start_time))
            .unwrap_or_default()
    };
    let end = move || {
        state.bell_times.get()
            .iter()
            .find(|b| b.number == number)
            .map(|b| strip_seconds(&b.end_time))
            .unwrap_or_default()
    };
    row((
        label(move || number.to_string())
            .font(Font::Title3)
            .color(move || Color::hex(state.accent_color.get()))
            .frame(32.0, 32.0),
        column((
            label(move || format!("{} урок", number))
                .font(Font::Headline),
            label(move || format!("{} — {}", start(), end()))
                .font(Font::Subheadline)
                .secondary(),
        ))
        .spacing(2.0)
        .align(HAlign::Leading)
        .grow(),
    ))
    .spacing(12.0)
    .padding(Insets { top: 10.0, leading: 16.0, bottom: 10.0, trailing: 20.0 })
    .background(Color::rgba(0.95, 0.95, 0.97, 1.0))
    .corner_radius(8.0)
    .padding(Insets { top: 0.0, leading: 20.0, bottom: 4.0, trailing: 20.0 })
}

fn strip_seconds(t: &str) -> String {
    t.get(..5).unwrap_or(t).to_string()
}
