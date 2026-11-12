use super::data::*;
use crate::app::AppState;
use crate::app::preferences::colors;
use crate::app::format;
use day::prelude::*;

pub(super) fn year_summary_at(state: AppState) -> impl Piece {
    let s_subj = state;
    let cell_state = state;
    each(
        items(
            move || get_all_subject_names(s_subj),
            |s: &String| s.clone(),
        ),
        move |item| {
            let subj_name = item.get();
            let sj0 = subj_name.clone();
            let sj1 = subj_name.clone();
            let sj2 = subj_name.clone();
            let sj3 = subj_name.clone();
            let sja = subj_name.clone();
            let sjf = subj_name.clone();
            column((
                row((
                    label(subj_name)
                        .font(Font::Subheadline)
                        .grow(Dimension::Both),
                    year_q_cell(cell_state, sj0, 0),
                    year_q_cell(cell_state, sj1, 1),
                    year_q_cell(cell_state, sj2, 2),
                    year_q_cell(cell_state, sj3, 3),
                    year_avg_cell(cell_state, sja),
                    year_final_cell(cell_state, sjf),
                ))
                .spacing(4.0)
                .padding(Insets {
                    top: 8.0,
                    leading: 12.0,
                    bottom: 8.0,
                    trailing: 12.0,
                }),
                divider().padding(Insets {
                    top: 0.0,
                    leading: 12.0,
                    bottom: 0.0,
                    trailing: 12.0,
                }),
            ))
            .spacing(0.0)
            .any()
        },
    )
}

fn year_q_cell(state: AppState, subject: String, q: usize) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let sj1 = subject.clone();
    let sj2 = subject;
    zstack((
        label(move || final_mark_text(final_mark(s1, &sj1, quarter_period(q))))
            .font(Font::Subheadline)
            .color(move || {
                match final_mark(s2, &sj2, quarter_period(q)).and_then(|v| v.parse::<f64>().ok()) {
                    Some(v) => format::avg_grade_color(v),
                    None => colors::SECONDARY,
                }
            })
            .align(TextAlign::Center),
    ))
    .width(28.0)
}

fn calc_year_avg(state: AppState, subject: &str) -> Option<f64> {
    let mut q_marks: Vec<f64> = Vec::new();
    for q in 0..4 {
        if let Some(m) =
            final_mark(state, subject, quarter_period(q)).and_then(|v| v.parse::<f64>().ok())
        {
            q_marks.push(m);
        }
    }
    if q_marks.is_empty() {
        None
    } else {
        Some(q_marks.iter().sum::<f64>() / q_marks.len() as f64)
    }
}

fn year_avg_cell(state: AppState, subject: String) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let sj1 = subject.clone();
    let sj2 = subject;
    zstack((label(move || {
        let avg = calc_year_avg(s1, &sj1);
        match avg {
            Some(v) => format!("{:.2}", v),
            None => "—".into(),
        }
    })
    .font(Font::Headline)
    .color(move || {
        let avg = calc_year_avg(s2, &sj2);
        match avg {
            Some(v) => format::avg_grade_color(v),
            None => colors::SECONDARY,
        }
    })
    .align(TextAlign::Center),))
    .width(42.0)
}

fn year_final_cell(state: AppState, subject: String) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let sj1 = subject.clone();
    let sj2 = subject;
    zstack((label(move || {
        // Strictly the official year mark («Годовая», /final/whole) — a
        // calculated average is not a teacher's decision, so an unset mark
        // renders as a dash.
        final_mark_text(final_mark(s1, &sj1, "YEAR"))
    })
    .font(Font::Headline)
    .color(
        move || match final_mark(s2, &sj2, "YEAR").and_then(|v| v.parse::<f64>().ok()) {
            Some(v) => format::avg_grade_color(v),
            None => colors::SECONDARY,
        },
    )
    .align(TextAlign::Center),))
    .width(42.0)
}
