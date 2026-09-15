use crate::app::AppState;
use day::prelude::*;

pub fn render(state: AppState) -> impl Piece {
    column((
        label("Неделя")
            .font(Font::Headline)
            .color(move || Color::hex(state.accent_color.get()))
            .padding(Insets { top: 16.0, leading: 20.0, bottom: 6.0, trailing: 20.0 }),

        row((
            super::stat_block::render(state, "Уроков", move || {
                let lessons = state.lessons.get();
                lessons.iter().map(|d| d.slots.len()).sum::<usize>().to_string()
            }),
            super::stat_block::render(state, "Оценок", move || {
                let lessons = state.lessons.get();
                lessons.iter()
                    .flat_map(|d| &d.slots)
                    .filter(|s| s.lesson_mark.is_some())
                    .count()
                    .to_string()
            }),
            super::stat_block::render(state, "Ср. балл", move || {
                let lessons = state.lessons.get();
                let marks: Vec<f64> = lessons.iter()
                    .flat_map(|d| &d.slots)
                    .filter_map(|s| s.lesson_mark.as_ref())
                    .filter_map(|m| m.mark.as_ref())
                    .filter_map(|m| m.parse::<f64>().ok())
                    .collect();
                if marks.is_empty() { "—".into() }
                else { format!("{:.1}", marks.iter().sum::<f64>() / marks.len() as f64) }
            }),
        ))
        .spacing(8.0)
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 }),
    ))
    .spacing(0.0)
}
