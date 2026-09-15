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
        year_btn(state),
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
        if cur == q && !s1.marks_loading.get() { format!("[{}]", lbl) } else { lbl.clone() }
    })
    .action(move || {
        features::diary::load_quarter(s2, q);
    })
    .id(format!("q-{label}"))
}

fn year_btn(state: AppState) -> impl Piece {
    let s1 = state;
    let s2 = state;
    button(move || {
        let cur = s1.current_quarter.get();
        if cur == 4 && !s1.marks_loading.get() { "[Год]".to_string() } else { "Год".to_string() }
    })
    .action(move || {
        features::diary::load_year(s2);
    })
    .id("q-year")
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
        label(move || {
            let q = state.current_quarter.get();
            if q == 4 { "Итоги года" } else { "Итоги четверти" }
        })
        .font(Font::Headline)
        .color(colors::PRIMARY)
        .padding(Insets { top: 16.0, leading: 20.0, bottom: 6.0, trailing: 20.0 }),

        // Quarter stats
        when(
            move || state.current_quarter.get() != 4,
            move || quarter_stats(state),
        ),

        // Year stats with 3 averages
        when(
            move || state.current_quarter.get() == 4,
            move || year_stats(state),
        ),

        // Per-quarter subject breakdown (year view only)
        when(
            move || state.current_quarter.get() == 4 && !state.year_quarter_data.get().is_empty(),
            move || year_quarter_subjects(state),
        ),

        // Subject list (quarter view)
        when(
            move || state.current_quarter.get() != 4,
            move || subject_list(state),
        ),
    ))
    .spacing(0.0)
    .grow()
}

fn quarter_stats(state: AppState) -> impl Piece {
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
            let total: usize = marks.values().map(|v| v.len()).sum();
            if total == 0 { "—".into() }
            else {
                let sum: f64 = marks.values().flat_map(|v| v.iter()).sum();
                format!("{:.2}", sum / total as f64)
            }
        }),
    ))
    .spacing(8.0)
    .padding(Insets { top: 0.0, leading: 20.0, bottom: 12.0, trailing: 20.0 })
}

fn year_stats(state: AppState) -> impl Piece {
    column((
        row((
            stat_block::render("Предметов", move || {
                state.quarter_marks.get().len().to_string()
            }),
            stat_block::render("Оценок", move || {
                let marks = state.quarter_marks.get();
                marks.values().map(|v| v.len()).sum::<usize>().to_string()
            }),
        ))
        .spacing(8.0)
        .padding(Insets { top: 0.0, leading: 20.0, bottom: 8.0, trailing: 20.0 }),

        // Average 1: by all marks
        year_avg_row("Средний (все оценки)", move || {
            let marks = state.quarter_marks.get();
            let total: usize = marks.values().map(|v| v.len()).sum();
            if total == 0 { return "—".into(); }
            let sum: f64 = marks.values().flat_map(|v| v.iter()).sum();
            format!("{:.2}", sum / total as f64)
        }),

        // Average 2: by quarter averages
        year_avg_row("Средний (по четвертям)", move || {
            let yqd = state.year_quarter_data.get();
            if yqd.is_empty() { return "—".into(); }
            let q_avgs: Vec<f64> = yqd.iter().map(|(_, marks)| {
                let total: usize = marks.values().map(|v| v.len()).sum();
                if total == 0 { 0.0 }
                else {
                    let sum: f64 = marks.values().flat_map(|v| v.iter()).sum();
                    sum / total as f64
                }
            }).filter(|a| *a > 0.0).collect();
            if q_avgs.is_empty() { "—".into() }
            else {
                let avg = q_avgs.iter().sum::<f64>() / q_avgs.len() as f64;
                format!("{:.2}", avg)
            }
        }),

        // Average 3: weighted by marks count per quarter
        year_avg_row("Средний (взвешенный)", move || {
            let yqd = state.year_quarter_data.get();
            if yqd.is_empty() { return "—".into(); }
            let mut weighted_sum = 0.0;
            let mut total_marks = 0usize;
            for (_, marks) in yqd.iter() {
                let q_total: usize = marks.values().map(|v| v.len()).sum();
                let q_sum: f64 = marks.values().flat_map(|v| v.iter()).sum();
                weighted_sum += q_sum;
                total_marks += q_total;
            }
            if total_marks == 0 { "—".into() }
            else { format!("{:.2}", weighted_sum / total_marks as f64) }
        }),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 12.0, trailing: 0.0 })
}

fn year_avg_row(lbl: &'static str, value: impl Fn() -> String + 'static) -> impl Piece {
    row((
        day::prelude::label(lbl)
            .font(Font::Body)
            .grow(),
        day::prelude::label(value)
            .font(Font::Headline)
            .color(colors::PRIMARY),
    ))
    .spacing(8.0)
    .padding(Insets { top: 4.0, leading: 20.0, bottom: 4.0, trailing: 20.0 })
}

fn year_quarter_subjects(state: AppState) -> impl Piece {
    column((
        label("Оценки по четвертям")
            .font(Font::Headline)
            .color(colors::PRIMARY)
            .padding(Insets { top: 8.0, leading: 20.0, bottom: 6.0, trailing: 20.0 }),
        each(
            items(
                move || {
                    let marks = state.quarter_marks.get();
                    let mut subjects: Vec<String> = marks.keys().cloned().collect();
                    subjects.sort();
                    subjects
                },
                |s: &String| s.clone(),
            ),
            move |item| {
                let subj_name = item.get();
                year_subject_row(state, subj_name).any()
            },
        ),
    ))
    .spacing(0.0)
}

fn year_subject_row(state: AppState, subject: String) -> impl Piece {
    let subj = subject.clone();
    let subj2 = subject.clone();

    column((
        // Subject name + overall average
        row((
            label(subject)
                .font(Font::Headline)
                .grow(),
            label(move || {
                let marks = state.quarter_marks.get();
                if let Some(vals) = marks.get(&*subj) {
                    if vals.is_empty() { "—".into() }
                    else {
                        let avg = vals.iter().sum::<f64>() / vals.len() as f64;
                        format!("{:.2}", avg)
                    }
                } else { "—".into() }
            })
            .font(Font::Headline)
            .color(colors::PRIMARY),
        ))
        .spacing(8.0)
        .padding(Insets { top: 8.0, leading: 20.0, bottom: 4.0, trailing: 20.0 }),

        // Per-quarter row
        quarter_avg_row(state, subj2),

        divider(),
    ))
    .spacing(0.0)
}

fn quarter_avg_row(state: AppState, subject: String) -> impl Piece {
    let s = state;
    let sj = subject;
    row((
        quarter_cell(s, sj.clone(), 0),
        quarter_cell(s, sj.clone(), 1),
        quarter_cell(s, sj.clone(), 2),
        quarter_cell(s, sj, 3),
    ))
    .spacing(4.0)
    .padding(Insets { top: 0.0, leading: 20.0, bottom: 8.0, trailing: 20.0 })
}

fn quarter_cell(state: AppState, subject: String, q: usize) -> impl Piece {
    let q_label = ["I", "II", "III", "IV"][q].to_string();
    column((
        day::prelude::label(q_label)
            .font(Font::Caption)
            .secondary()
            .align(TextAlign::Center),
        day::prelude::label(move || {
            let yqd = state.year_quarter_data.get();
            if let Some((_, marks)) = yqd.get(q) {
                if let Some(vals) = marks.get(&*subject) {
                    if vals.is_empty() { "—".into() }
                    else {
                        let avg = vals.iter().sum::<f64>() / vals.len() as f64;
                        format!("{:.1}", avg)
                    }
                } else { "—".into() }
            } else { "—".into() }
        })
        .font(Font::Body)
        .align(TextAlign::Center),
    ))
    .spacing(2.0)
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
                if s.count == 0 {
                    label("—")
                        .font(Font::Headline)
                        .secondary()
                } else {
                    label(format!("{:.1}", s.avg))
                        .font(Font::Headline)
                        .color(if s.avg >= 4.0 { colors::SUCCESS } else if s.avg >= 3.0 { colors::WARNING } else { colors::ERROR })
                },
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
