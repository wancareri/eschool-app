use crate::app::AppState;
use day::prelude::*;

pub fn render(state: AppState) -> impl Piece {
    column((
        label("Четверть")
            .font(Font::Headline)
            .color(move || Color::hex(state.accent_color.get()))
            .padding(Insets { top: 16.0, leading: 20.0, bottom: 6.0, trailing: 20.0 }),

        row((
            spacer().grow(),
            super::stat_block::render(state, "Четверть", move || {
                let idx = state.current_week_index.get();
                let q = if idx < 9 { "I" } else if idx < 18 { "II" } else if idx < 27 { "III" } else { "IV" };
                format!("{} четверть", q)
            }),
            super::stat_block::render(state, "Оценок", move || {
                state.all_marks.get().len().to_string()
            }),
            super::stat_block::render(state, "Средний балл", move || {
                let marks = state.all_marks.get();
                if marks.is_empty() { "—".into() }
                else {
                    let avg: f64 = marks.iter().map(|(_, v)| v).sum::<f64>() / marks.len() as f64;
                    format!("{:.2}", avg)
                }
            }),
            spacer().grow(),
        ))
        .spacing(8.0)
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),

        label(move || {
            let marks = state.all_marks.get();
            if marks.is_empty() {
                return "Нет оценок за четверть".into();
            }
            let mut by_subject: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
            for (subj, val) in &marks {
                by_subject.entry(subj.clone()).or_default().push(*val);
            }
            let mut subjects: Vec<_> = by_subject.into_iter().collect();
            subjects.sort_by(|a, b| a.0.cmp(&b.0));

            subjects.iter().map(|(subj, vals)| {
                let avg = vals.iter().sum::<f64>() / vals.len() as f64;
                let cnt = vals.len();
                format!("{} — {:.1}  ({})", subj, avg, cnt)
            }).collect::<Vec<_>>().join("\n")
        })
        .font(Font::Caption)
        .secondary()
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 }),
    ))
    .spacing(0.0)
}
