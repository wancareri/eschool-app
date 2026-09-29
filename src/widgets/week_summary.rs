use crate::app::AppState;
use eschool_api::entities::*;
use day::prelude::*;

/// A пропуск is the «н» mark in the Е-школа journal (some payloads tag the
/// record via `kind` instead of spelling the mark).
fn is_absence(s: &LessonSlot) -> bool {
    let Some(lm) = s.lesson_mark.as_ref() else {
        return false;
    };
    if let Some(k) = lm.kind.as_deref() {
        let k = k.trim().to_lowercase();
        if k == "absence" || k == "absent" || k == "пропуск" {
            return true;
        }
    }
    matches!(
        lm.mark.as_deref().map(|m| m.trim().to_lowercase()),
        Some(ref m) if m == "н" || m == "n"
    )
}

pub fn render<F>(state: AppState, get_lessons: F) -> impl Piece
where
    F: Fn() -> Vec<DaySchedule> + Copy + 'static,
{
    column((
        label("Неделя")
            .font(Font::Headline)
            .color(move || Color::hex(state.accent_color.get()))
            .align(TextAlign::Center)
            .padding(Insets { top: 16.0, leading: 20.0, bottom: 6.0, trailing: 20.0 }),

        row((
            super::stat_block::render(state, "Уроков", move || {
                let lessons = get_lessons();
                lessons.iter().map(|d| d.slots.len()).sum::<usize>().to_string()
            }),
            super::stat_block::render(state, "Оценок", move || {
                let lessons = get_lessons();
                lessons.iter()
                    .flat_map(|d| &d.slots)
                    .filter(|s| s.lesson_mark.is_some())
                    .count()
                    .to_string()
            }),
            super::stat_block::render(state, "Ср. балл", move || {
                let lessons = get_lessons();
                let marks: Vec<f64> = lessons.iter()
                    .flat_map(|d| &d.slots)
                    .filter_map(|s| s.lesson_mark.as_ref())
                    .filter_map(|m| m.mark.as_ref())
                    .filter_map(|m| m.parse::<f64>().ok())
                    .collect();
                if marks.is_empty() { "—".into() }
                else { format!("{:.2}", marks.iter().sum::<f64>() / marks.len() as f64) }
            }),
            super::stat_block::render(state, "Пропусков", move || {
                let lessons = get_lessons();
                lessons.iter()
                    .flat_map(|d| &d.slots)
                    .filter(|s| is_absence(s))
                    .count()
                    .to_string()
            }),
        ))
        .spacing(8.0)
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 }),
    ))
    .align(HAlign::Center)
    .spacing(0.0)
}
