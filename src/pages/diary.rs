use crate::app::AppState;
use crate::features;
use eschool_api::entities::*;
use crate::shared::{colors, utils};
use crate::widgets;
use crate::widgets::stat_block;
use crate::res;
use day::prelude::*;

const PAD: f64 = 20.0;

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let show_summary = Signal::new(false);

    scroll(column((
        column((
            label(move || res::str::diary_title().format())
                .font(Font::LargeTitle)
                .align(TextAlign::Center),
        ))
        .spacing(6.0)
        .padding(Insets { top: 16.0, leading: PAD, bottom: 4.0, trailing: PAD }),

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

        // Inline spinner — shows alongside content, not blocking
        when(
            move || state.is_authenticated.get() && state.marks_loading.get(),
            || row((spacer().grow(), spinner(), label("  Загрузка…").font(Font::Caption).secondary(), spacer().grow()))
                .padding(Insets { top: 8.0, leading: PAD, bottom: 8.0, trailing: PAD }),
        ),

        when(
            move || state.is_authenticated.get() && !show_summary.get(),
            move || week_view(state),
        ),
        when(
            move || state.is_authenticated.get() && show_summary.get(),
            move || summary_view(state),
        ),
    ))
    .spacing(0.0)
    .grow())
    .grow()
}

// ── Quarter / Year tabs ────────────────────────────────────────────────

fn quarter_tabs(state: AppState) -> impl Piece {
    row((
        quarter_btn(state, "I", 0),
        quarter_btn(state, "II", 1),
        quarter_btn(state, "III", 2),
        quarter_btn(state, "IV", 3),
        year_btn(state),
    ))
    .spacing(6.0)
    .padding(Insets { top: 0.0, leading: 12.0, bottom: 8.0, trailing: 12.0 })
}

fn quarter_btn(state: AppState, lbl: &'static str, q: usize) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let l = lbl.to_string();
    button(move || {
        let cur = s1.current_quarter.get();
        if cur == q && !s1.marks_loading.get() { format!("[{}]", l) } else { l.clone() }
    })
    .action(move || {
        // Show existing per-quarter data immediately
        let q_all = s2.quarter_all_marks.get();
        if let Some(marks) = q_all.get(q) {
            if !marks.is_empty() {
                s2.quarter_marks.set(marks.clone());
                let q_off = s2.quarter_official_marks.get();
                if let Some(off) = q_off.get(q) {
                    s2.official_marks.set(off.clone());
                }
            }
        }
        s2.current_quarter.set(q);
        features::diary::load_quarter(s2, q);
    })
    .id(format!("q-{lbl}"))
}

fn year_btn(state: AppState) -> impl Piece {
    let s1 = state;
    let s2 = state;
    button(move || {
        let cur = s1.current_quarter.get();
        if cur == 4 && !s1.marks_loading.get() { String::from("[Год]") } else { String::from("Год") }
    })
    .action(move || { features::diary::load_year(s2); })
    .id("q-year")
}

fn sub_tabs(state: AppState, show_summary: Signal<bool>) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let s3 = state;
    let s4 = state;
    row((
        button(move || {
            if !show_summary.get() { "● Недели" } else { "○ Недели" }
        })
        .action(move || {
            show_summary.set(false);
            let q = s1.current_quarter.get();
            let weeks = s2.all_weeks.get();
            let range = features::diary::quarter_week_indices(q);
            if range.start < weeks.len() { features::diary::load_week(s2, range.start as i32); }
        })
        .id("sub-weeks"),
        button(move || {
            if show_summary.get() { "● Итоги" } else { "○ Итоги" }
        })
        .action(move || {
            show_summary.set(true);
            // Show existing per-quarter data immediately
            let q = s3.current_quarter.get();
            let q_all = s3.quarter_all_marks.get();
            if let Some(marks) = q_all.get(q) {
                if !marks.is_empty() {
                    s3.quarter_marks.set(marks.clone());
                    let q_off = s3.quarter_official_marks.get();
                    if let Some(off) = q_off.get(q) {
                        s3.official_marks.set(off.clone());
                    }
                }
            }
            if s3.quarter_marks.get().is_empty() { features::diary::load_quarter(s4, q); }
        })
        .id("sub-summary"),
    ))
    .spacing(12.0)
    .padding(Insets { top: 0.0, leading: 16.0, bottom: 8.0, trailing: 16.0 })
}

// ── Week view ──────────────────────────────────────────────────────────

fn week_view(state: AppState) -> impl Piece {
    column((
        when(
            move || state.lessons_loading.get() && state.lessons.get().is_empty(),
            || row((spacer().grow(), spinner(), label("  Загрузка…").font(Font::Caption).secondary(), spacer().grow()))
                .padding(Insets { top: 24.0, ..Default::default() }),
        ),
        when(
            move || !state.lessons.get().is_empty(),
            move || {
                column((
                    week_header(state),
                    when(
                        move || !state.lessons_loading.get(),
                        move || widgets::week_summary::render(state),
                    ),
                    diary_list(state),
                )).spacing(0.0)
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
            .font(Font::Headline).grow().align(TextAlign::Center),
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
        items(move || state.lessons.get(), |d: &DaySchedule| d.date),
        move |day_slot| {
            let date = day_slot.key();
            day_card(state, date).any()
        },
    )
}

fn day_card(state: AppState, date: u64) -> impl Piece {
    column((
        label(move || {
            state.lessons.get().iter().find(|d| d.date == date)
                .map(|d| utils::format_date_header(d.day_of_week, d.date))
                .unwrap_or_default()
        })
        .font(Font::Headline).color(colors::PRIMARY)
        .padding(Insets { top: 16.0, leading: PAD, bottom: 6.0, trailing: PAD }),
        each(
            items(
                move || {
                    state.lessons.get().into_iter().find(|d| d.date == date)
                        .map(|d| d.slots).unwrap_or_default()
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

// ── Summary view ───────────────────────────────────────────────────────

fn summary_view(state: AppState) -> impl Piece {
    column((
        label(move || {
            if state.current_quarter.get() == 4 { "Итоги года" } else { "Итоги четверти" }
        })
        .font(Font::Headline).color(colors::PRIMARY)
        .align(TextAlign::Center)
        .padding(Insets { top: 16.0, leading: PAD, bottom: 6.0, trailing: PAD }),

        when(move || state.current_quarter.get() != 4, move || quarter_stats(state)),
        when(move || state.current_quarter.get() == 4, move || year_stats(state)),

        // ── Расчётные (сначала) ──
        section_header("Расчётные оценки"),

        when(move || state.current_quarter.get() == 4 && !state.year_quarter_data.get().is_empty(),
            move || year_quarter_subjects(state)),
        when(move || state.current_quarter.get() != 4,
            move || subject_list(state)),

        divider().padding(Insets { top: 8.0, leading: PAD, bottom: 8.0, trailing: PAD }),

        // ── Выставленные (потом) ──
        section_header("Выставленные оценки"),

        when(move || state.current_quarter.get() != 4,
            move || official_quarter_list(state)),
        when(move || state.current_quarter.get() == 4 && !state.year_quarter_data.get().is_empty(),
            move || official_year_list(state)),
    ))
    .spacing(0.0)
    .grow()
}

fn section_header(text: &'static str) -> impl Piece {
    label(text)
        .font(Font::Title3).color(colors::PRIMARY)
        .align(TextAlign::Center)
        .padding(Insets { top: 10.0, leading: PAD, bottom: 6.0, trailing: PAD })
}

// ── Quarter stats ──────────────────────────────────────────────────────

fn quarter_stats(state: AppState) -> impl Piece {
    row((
        stat_block::render("Четверть", move || {
            let labels = ["I", "II", "III", "IV"];
            format!("{} четверть", labels[state.current_quarter.get()])
        }),
        stat_block::render("Предметов", move || state.quarter_marks.get().len().to_string()),
        stat_block::render("Средний балл", move || {
            let marks = state.quarter_marks.get();
            let total: usize = marks.values().map(|v| v.len()).sum();
            if total == 0 { "—".into() }
            else { format!("{:.2}", marks.values().flat_map(|v| v.iter()).sum::<f64>() / total as f64) }
        }),
    ))
    .spacing(8.0)
    .padding(Insets { top: 0.0, leading: PAD, bottom: 12.0, trailing: PAD })
}

fn year_stats(state: AppState) -> impl Piece {
    column((
        row((
            stat_block::render("Предметов", move || state.quarter_marks.get().len().to_string()),
            stat_block::render("Оценок", move || {
                state.quarter_marks.get().values().map(|v| v.len()).sum::<usize>().to_string()
            }),
        ))
        .spacing(8.0)
        .padding(Insets { top: 0.0, leading: PAD, bottom: 8.0, trailing: PAD }),

        year_avg_row("Средний (все оценки)", move || {
            let marks = state.quarter_marks.get();
            let total: usize = marks.values().map(|v| v.len()).sum();
            if total == 0 { return "—".into(); }
            format!("{:.2}", marks.values().flat_map(|v| v.iter()).sum::<f64>() / total as f64)
        }),
        year_avg_row("Средний (по четвертям)", move || {
            let yqd = state.year_quarter_data.get();
            if yqd.is_empty() { return "—".into(); }
            let q_avgs: Vec<f64> = yqd.iter().map(|(_, m)| {
                let t: usize = m.values().map(|v| v.len()).sum();
                if t == 0 { 0.0 } else { m.values().flat_map(|v| v.iter()).sum::<f64>() / t as f64 }
            }).filter(|a| *a > 0.0).collect();
            if q_avgs.is_empty() { "—".into() } else { format!("{:.2}", q_avgs.iter().sum::<f64>() / q_avgs.len() as f64) }
        }),
        year_avg_row("Средний (взвешенный)", move || {
            let yqd = state.year_quarter_data.get();
            if yqd.is_empty() { return "—".into(); }
            let (mut ws, mut tm) = (0.0, 0usize);
            for (_, m) in yqd.iter() { tm += m.values().map(|v| v.len()).sum::<usize>(); ws += m.values().flat_map(|v| v.iter()).sum::<f64>(); }
            if tm == 0 { "—".into() } else { format!("{:.2}", ws / tm as f64) }
        }),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 12.0, trailing: 0.0 })
}

fn year_avg_row(lbl: &'static str, value: impl Fn() -> String + 'static) -> impl Piece {
    row((
        day::prelude::label(lbl).font(Font::Body).grow(),
        day::prelude::label(value).font(Font::Headline).color(colors::PRIMARY),
    ))
    .spacing(8.0)
    .padding(Insets { top: 4.0, leading: PAD, bottom: 4.0, trailing: PAD })
}

// ── Calculated: year per-quarter subjects ───────────────────────────────

fn year_quarter_subjects(state: AppState) -> impl Piece {
    each(
        items(
            move || {
                let mut subjects: Vec<String> = state.quarter_marks.get().keys().cloned().collect();
                subjects.sort();
                subjects
            },
            |s: &String| s.clone(),
        ),
        move |item| {
            let subj_name = item.get();
            year_subject_row(state, subj_name).any()
        },
    )
}

fn year_subject_row(state: AppState, subject: String) -> impl Piece {
    let sj = subject.clone();
    let sj2 = subject.clone();

    column((
        row((
            label(subject).font(Font::Headline).grow(),
            label(move || {
                let marks = state.quarter_marks.get();
                if let Some(vals) = marks.get(&*sj) {
                    if vals.is_empty() { "—".into() }
                    else { format!("{:.2}", vals.iter().sum::<f64>() / vals.len() as f64) }
                } else { "—".into() }
            }).font(Font::Headline).color(colors::PRIMARY),
        ))
        .spacing(8.0)
        .padding(Insets { top: 6.0, leading: PAD, bottom: 2.0, trailing: PAD }),
        quarter_avg_row(state, sj2),
        divider(),
    )).spacing(0.0)
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
    .padding(Insets { top: 0.0, leading: 20.0, bottom: 6.0, trailing: 20.0 })
}

fn quarter_cell(state: AppState, subject: String, q: usize) -> impl Piece {
    let ql = ["I", "II", "III", "IV"][q].to_string();
    column((
        day::prelude::label(ql).font(Font::Caption).secondary().align(TextAlign::Center),
        day::prelude::label(move || {
            let yqd = state.year_quarter_data.get();
            if let Some((_, m)) = yqd.get(q) {
                if let Some(v) = m.get(&*subject) {
                    if v.is_empty() { "—".into() } else { format!("{:.1}", v.iter().sum::<f64>() / v.len() as f64) }
                } else { "—".into() }
            } else { "—".into() }
        }).font(Font::Body).align(TextAlign::Center),
    ))
    .spacing(2.0).grow()
}

// ── Calculated: quarter subject list ───────────────────────────────────

fn subject_list(state: AppState) -> impl Piece {
    each(
        items(
            move || {
                let marks = state.quarter_marks.get();
                let mut subjects: Vec<SubjectMark> = marks.iter().map(|(name, vals)| {
                    let (sum, len) = (vals.iter().sum::<f64>(), vals.len());
                    SubjectMark { name: name.clone(), avg: if len > 0 { sum / len as f64 } else { 0.0 }, count: len }
                }).collect();
                subjects.sort_by(|a, b| a.name.cmp(&b.name));
                subjects
            },
            |s: &SubjectMark| s.name.clone(),
        ),
        move |item| {
            let s = item.get();
            row((
                label(s.name.clone()).font(Font::Body).grow(),
                if s.count == 0 { label("—").font(Font::Headline).secondary() }
                else {
                    label(format!("{:.1}", s.avg)).font(Font::Headline)
                        .color(if s.avg >= 4.0 { colors::SUCCESS } else if s.avg >= 3.0 { colors::WARNING } else { colors::ERROR })
                },
                label(format!("({})", s.count)).font(Font::Caption).secondary(),
            ))
            .spacing(8.0)
            .padding(Insets { top: 6.0, leading: PAD, bottom: 6.0, trailing: PAD })
            .any()
        },
    )
}

// ── Official: quarter list ─────────────────────────────────────────────

fn official_quarter_list(state: AppState) -> impl Piece {
    each(
        items(
            move || {
                let marks = state.official_marks.get();
                let mut subjects: Vec<OfficialSubject> = marks.iter()
                    .filter(|(_, vals)| !vals.is_empty())
                    .map(|(name, vals)| {
                        let avg = vals.iter().map(|m| m.value).sum::<f64>() / vals.len() as f64;
                        OfficialSubject { name: name.clone(), avg, count: vals.len() }
                    }).collect();
                subjects.sort_by(|a, b| a.name.cmp(&b.name));
                subjects
            },
            |s: &OfficialSubject| s.name.clone(),
        ),
        move |item| {
            let s = item.get();
            row((
                label(s.name.clone()).font(Font::Body).grow(),
                label(format!("{:.1}", s.avg)).font(Font::Headline)
                    .color(if s.avg >= 4.0 { colors::SUCCESS } else if s.avg >= 3.0 { colors::WARNING } else { colors::ERROR }),
                label(format!("({})", s.count)).font(Font::Caption).secondary(),
            ))
            .spacing(8.0)
            .padding(Insets { top: 6.0, leading: PAD, bottom: 6.0, trailing: PAD })
            .any()
        },
    )
}

// ── Official: year list ────────────────────────────────────────────────

fn official_year_list(state: AppState) -> impl Piece {
    each(
        items(
            move || {
                let marks = state.official_marks.get();
                let mut subjects: Vec<String> = marks.keys().cloned().collect();
                subjects.sort();
                subjects
            },
            |s: &String| s.clone(),
        ),
        move |item| {
            let subj_name = item.get();
            official_year_row(state, subj_name).any()
        },
    )
}

fn official_year_row(state: AppState, subject: String) -> impl Piece {
    let sj = subject.clone();
    let sj2 = subject.clone();

    column((
        row((
            label(subject).font(Font::Headline).grow(),
            label(move || {
                let marks = state.official_marks.get();
                if let Some(vals) = marks.get(&*sj) {
                    if vals.is_empty() { "—".into() }
                    else { format!("{:.2} ({})", vals.iter().map(|m| m.value).sum::<f64>() / vals.len() as f64, vals.len()) }
                } else { "—".into() }
            }).font(Font::Headline).color(colors::PRIMARY),
        ))
        .spacing(8.0)
        .padding(Insets { top: 6.0, leading: PAD, bottom: 2.0, trailing: PAD }),
        label(move || {
            let marks = state.official_marks.get();
            if let Some(vals) = marks.get(&*sj2) {
                if vals.is_empty() { return "Нет оценок".into(); }
                vals.iter().map(|m| format!("{:.0}", m.value)).collect::<Vec<_>>().join(" · ")
            } else { "Нет оценок".into() }
        })
        .font(Font::Caption).secondary()
        .padding(Insets { top: 0.0, leading: PAD, bottom: 6.0, trailing: PAD }),
        divider(),
    )).spacing(0.0)
}

// ── Structs ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct OfficialSubject { name: String, avg: f64, count: usize }

#[derive(Clone, Debug)]
struct SubjectMark { name: String, avg: f64, count: usize }
