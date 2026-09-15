use crate::app::AppState;
use crate::features;
use eschool_api::entities::*;
use crate::shared::{colors, utils};
use crate::widgets;
use crate::widgets::stat_block;
use crate::res;
use day::prelude::*;

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let show_summary = Signal::new(false);

    scroll(column((
        column((
            label(move || res::str::diary_title().format())
                .font(Font::LargeTitle),
        ))
        .spacing(6.0)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 4.0, trailing: 20.0 }),

        when(
            move || state.is_authenticated.get(),
            move || quarter_tabs(state),
        ),

        when(
            move || state.is_authenticated.get(),
            move || sub_tabs(state, show_summary),
        ),

        when(
            move || !state.is_authenticated.get(),
            || column((
                spacer(),
                label("Войдите для просмотра дневника")
                    .font(Font::Body).secondary().align(TextAlign::Center),
                spacer(),
            )).grow(),
        ),
        when(
            move || state.is_authenticated.get() && state.marks_loading.get(),
            || column((
                spacer(),
                spinner(),
                label("Загрузка оценок…")
                    .font(Font::Caption).secondary(),
                spacer(),
            )).grow(),
        ),
        when(
            move || state.is_authenticated.get() && !state.marks_loading.get() && !show_summary.get(),
            move || week_view(state),
        ),
        when(
            move || state.is_authenticated.get() && !state.marks_loading.get() && show_summary.get(),
            move || summary_view(state),
        ),
    ))
    .spacing(0.0)
    .grow())
    .grow()
}

fn quarter_tabs(state: AppState) -> impl Piece {
    row((
        quarter_btn(state, "I", 0),
        quarter_btn(state, "II", 1),
        quarter_btn(state, "III", 2),
        quarter_btn(state, "IV", 3),
    ))
    .spacing(6.0)
    .padding(Insets { top: 0.0, leading: 16.0, bottom: 8.0, trailing: 16.0 })
}

fn quarter_btn(state: AppState, label: &'static str, q: usize) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let lbl = label.to_string();
    button(move || {
        let cur = s1.current_quarter.get();
        if cur == q { format!("[{}]", lbl) } else { lbl.clone() }
    })
    .action(move || {
        features::diary::load_quarter(s2, q);
    })
    .id(format!("q-{label}"))
}

fn sub_tabs(state: AppState, show_summary: Signal<bool>) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let s3 = state;
    let s4 = state;
    row((
        button(move || {
            let active = !show_summary.get();
            if active { "● Недели" } else { "○ Недели" }
        })
        .action(move || {
            show_summary.set(false);
            let q = s1.current_quarter.get();
            let weeks = s2.all_weeks.get();
            let range = features::diary::quarter_week_indices(q);
            let start = range.start;
            if start < weeks.len() {
                features::diary::load_week(s2, start as i32);
            }
        })
        .id("sub-weeks"),
        button(move || {
            let active = show_summary.get();
            if active { "● Итоги" } else { "○ Итоги" }
        })
        .action(move || {
            show_summary.set(true);
            let q = s3.current_quarter.get();
            let marks = s4.quarter_marks.get();
            if marks.is_empty() {
                features::diary::load_quarter(s3, q);
            }
        })
        .id("sub-summary"),
    ))
    .spacing(12.0)
    .padding(Insets { top: 0.0, leading: 16.0, bottom: 8.0, trailing: 16.0 })
}

fn week_view(state: AppState) -> impl Piece {
    column((
        when(
            move || state.lessons_loading.get(),
            || column((
                spacer(),
                spinner(),
                spacer(),
            )).grow(),
        ),
        when(
            move || !state.lessons_loading.get(),
            move || {
                column((
                    week_header(state),
                    widgets::week_summary::render(state),
                    diary_list(state),
                ))
                .spacing(0.0)
            },
        ),
    ))
    .spacing(0.0)
    .grow()
}

fn week_header(state: AppState) -> impl Piece {
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
    .padding(Insets { top: 0.0, leading: 16.0, bottom: 6.0, trailing: 16.0 })
}

fn diary_list(state: AppState) -> impl Piece {
    each(
        items(
            move || state.lessons.get(),
            |d: &DaySchedule| d.date,
        ),
        move |day_slot| {
            let date = day_slot.key();
            day_card(state, date).any()
        },
    )
}

fn day_card(state: AppState, date: u64) -> impl Piece {
    column((
        label(move || {
            let lessons = state.lessons.get();
            lessons
                .iter()
                .find(|d| d.date == date)
                .map(|d| utils::format_date_header(d.day_of_week, d.date))
                .unwrap_or_default()
        })
        .font(Font::Headline)
        .color(colors::PRIMARY)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 6.0, trailing: 20.0 }),
        each(
            items(
                move || {
                    state.lessons.get()
                        .into_iter()
                        .find(|d| d.date == date)
                        .map(|d| d.slots)
                        .unwrap_or_default()
                },
                |s: &LessonSlot| s.number,
            ),
            move |slot| {
                let num = slot.key();
                widgets::lesson_row::render(state, date, num).any()
            },
        ),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
}

fn summary_view(state: AppState) -> impl Piece {
    column((
        label("Итоги четверти")
            .font(Font::Headline)
            .color(colors::PRIMARY)
            .padding(Insets { top: 16.0, leading: 20.0, bottom: 6.0, trailing: 20.0 }),

        row((
            stat_block::render("Четверть", move || {
                let q = state.current_quarter.get();
                let labels = ["I", "II", "III", "IV"];
                format!("{} четверть", labels[q])
            }),
            stat_block::render("Предметов", move || {
                state.quarter_marks.get().len().to_string()
            }),
            stat_block::render("Средний балл", move || {
                let marks = state.quarter_marks.get();
                if marks.is_empty() { "—".into() }
                else {
                    let total: usize = marks.values().map(|v| v.len()).sum();
                    let sum: f64 = marks.values().flat_map(|v| v.iter()).sum();
                    format!("{:.2}", sum / total as f64)
                }
            }),
        ))
        .spacing(8.0)
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 }),

        subject_list(state),
    ))
    .spacing(0.0)
    .grow()
}

fn subject_list(state: AppState) -> impl Piece {
    each(
        items(
            move || {
                let marks = state.quarter_marks.get();
                let mut subjects: Vec<SubjectMark> = marks.iter().map(|(name, vals)| {
                    let sum: f64 = vals.iter().sum();
                    let len = vals.len();
                    let avg = if len > 0 { sum / len as f64 } else { 0.0 };
                    SubjectMark { name: name.clone(), avg, count: len }
                }).collect();
                subjects.sort_by(|a, b| a.name.cmp(&b.name));
                subjects
            },
            |s: &SubjectMark| s.name.clone(),
        ),
        move |item| {
            let s = item.get();
            row((
                label(s.name.clone())
                    .font(Font::Body)
                    .grow(),
                label(format!("{:.1}", s.avg))
                    .font(Font::Headline)
                    .color(if s.avg >= 4.0 { colors::SUCCESS } else if s.avg >= 3.0 { colors::WARNING } else { colors::ERROR }),
                label(format!("({})", s.count))
                    .font(Font::Caption)
                    .secondary(),
            ))
            .spacing(8.0)
            .padding(Insets { top: 8.0, leading: 20.0, bottom: 8.0, trailing: 20.0 })
            .any()
        },
    )
}

#[derive(Clone, Debug)]
struct SubjectMark {
    name: String,
    avg: f64,
    count: usize,
}
