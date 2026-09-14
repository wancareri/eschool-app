use crate::app::AppState;
use crate::features;
use day::prelude::*;

pub fn render(state: AppState) -> impl Piece {
    column((
        row((
            button("<").action(move || {
                let idx = state.current_week_index.get();
                if idx > 0 { features::diary::load_week(state, idx - 1); }
            }).id("wk-prev"),
            label(move || state.current_week.get())
                .font(Font::Headline)
                .grow(),
            button(">").action(move || {
                let idx = state.current_week_index.get();
                let total = state.all_weeks.get().len() as i32;
                if idx + 1 < total { features::diary::load_week(state, idx + 1); }
            }).id("wk-next"),
        ))
        .spacing(12.0)
        .padding(Insets { top: 0.0, leading: 16.0, bottom: 6.0, trailing: 16.0 }),

        row((
            quarter_tab(state, "I", 0),
            quarter_tab(state, "II", 9),
            quarter_tab(state, "III", 18),
            quarter_tab(state, "IV", 27),
        ))
        .spacing(6.0)
        .padding(Insets { top: 0.0, leading: 16.0, bottom: 8.0, trailing: 16.0 }),
    ))
    .spacing(4.0)
}

fn quarter_tab(state: AppState, label: &'static str, from_week: i32) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let lbl = label.to_string();
    button(move || {
        let idx = s1.current_week_index.get();
        let is_active = idx >= from_week && idx < from_week + 9;
        if is_active { format!("[{}]", lbl) } else { lbl.clone() }
    })
    .action(move || {
        features::diary::reset_marks(s2);
        features::diary::load_week(s2, from_week);
    })
    .id(format!("q-{label}"))
}
