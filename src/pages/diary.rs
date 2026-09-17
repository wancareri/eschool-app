use crate::app::AppState;
use crate::features;
use eschool_api::entities::*;
use crate::shared::{colors, utils};
use crate::widgets;
use crate::widgets::stat_block;
use crate::res;
use day::prelude::*;
use day_piece_pullrefresh::pull_to_refresh;

const PAD: f64 = 20.0;

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let show_summary = Signal::new(false);
    let refreshing = Signal::new(false);

    pull_to_refresh(refreshing, scroll(column((
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

        // Initial load_all spinner — shown on top of content
        when(
            move || state.is_authenticated.get() && state.loading.get() && state.lessons.get().is_empty(),
            || row((spacer().grow(), spinner(), label("  Загрузка данных…").font(Font::Caption).secondary(), spacer().grow()))
                .padding(Insets { top: 24.0, leading: PAD, bottom: 24.0, trailing: PAD }),
        ),

        // Quarter/year marks loading spinner
        when(
            move || state.is_authenticated.get() && state.marks_loading.get(),
            || row((spacer().grow(), spinner(), label("  Загрузка оценок…").font(Font::Caption).secondary(), spacer().grow()))
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
    .grow()))
    .on_refresh(move || {
        let state = AppState::ambient();
        if state.is_authenticated.get() {
            features::diary::load_all(state);
        }
    })
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
            let q = s3.current_quarter.get();
            features::diary::load_quarter(s3, q);
        })
        .id("sub-summary"),
    ))
    .spacing(12.0)
    .padding(Insets { top: 0.0, leading: 16.0, bottom: 8.0, trailing: 16.0 })
}

// ── Week view ──────────────────────────────────────────────────────────

fn week_view(state: AppState) -> impl Piece {
    column((
        // Show spinner during week loading (alongside existing content)
        when(
            move || state.lessons_loading.get(),
            || row((spacer().grow(), spinner(), label("  Загрузка…").font(Font::Caption).secondary(), spacer().grow()))
                .padding(Insets { top: 8.0, leading: PAD, bottom: 8.0, trailing: PAD }),
        ),
        when(
            move || !state.lessons.get().is_empty() || !state.lessons_loading.get(),
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
        }).id("wk-prev").frame(44.0, 36.0),
        label(move || strip_week_summary(&state.current_week.get()))
            .font(Font::Headline)
            .align(TextAlign::Center)
            .grow(),
        button(">").action(move || {
            let idx = state.current_week_index.get();
            let total = state.all_weeks.get().len() as i32;
            if idx + 1 < total { features::diary::load_week(state, idx + 1); }
        }).id("wk-next").frame(44.0, 36.0),
    ))
    .spacing(12.0)
    .padding(Insets { top: 0.0, leading: 16.0, bottom: 6.0, trailing: 16.0 })
}

/// Strip leading zeros from week summary (e.g. "01 сентября" → "1 сентября")
fn strip_week_summary(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if chars[i].is_ascii_digit() {
            // Skip leading zeros in this number
            let mut started = false;
            while i < len && chars[i].is_ascii_digit() {
                if chars[i] != '0' || started {
                    started = true;
                    result.push(chars[i]);
                }
                i += 1;
            }
            if !started {
                // All zeros or single zero — keep one zero
                result.push('0');
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
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
        .font(Font::Headline).color(move || Color::hex(state.accent_color.get()))
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
        .font(Font::Headline).color(move || Color::hex(state.accent_color.get()))
        .align(TextAlign::Center)
        .padding(Insets { top: 16.0, leading: PAD, bottom: 6.0, trailing: PAD }),

        when(move || state.current_quarter.get() != 4 && {
            let off = state.official_marks.get();
            off.values().any(|v| !v.is_empty())
        }, move || quarter_stats(state)),
        when(move || state.current_quarter.get() == 4, move || year_stats(state)),

        section_header(state, "Оценки по предметам"),

        when(move || state.current_quarter.get() == 4 && !state.year_quarter_data.get().is_empty(),
            move || year_quarter_subjects(state)),
        when(move || state.current_quarter.get() != 4,
            move || subject_list(state)),
    ))
    .spacing(0.0)
    .grow()
}

fn section_header(state: AppState, text: &'static str) -> impl Piece {
    label(text)
        .font(Font::Title3).color(move || Color::hex(state.accent_color.get()))
        .align(TextAlign::Center)
        .padding(Insets { top: 10.0, leading: PAD, bottom: 6.0, trailing: PAD })
}

// ── Quarter stats ──────────────────────────────────────────────────────

fn quarter_stats(state: AppState) -> impl Piece {
    row((
        stat_block::render(state, "Четверть", move || {
            let labels = ["I", "II", "III", "IV"];
            format!("{} четверть", labels[state.current_quarter.get()])
        }),
        stat_block::render(state, "Предметов", move || {
            // Count ALL subjects from subjects_teachers
            let teachers = state.subjects_teachers.get();
            let marks = state.quarter_marks.get();
            let mut count = teachers.len();
            for name in marks.keys() {
                if !teachers.iter().any(|t| t.subject_title == *name) {
                    count += 1;
                }
            }
            count.to_string()
        }),
        stat_block::render(state, "Средний балл", move || {
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
            stat_block::render(state, "Предметов", move || {
                // Count ALL subjects from subjects_teachers
                let teachers = state.subjects_teachers.get();
                let marks = state.quarter_marks.get();
                let mut count = teachers.len();
                // Add any extra from marks not in teachers
                for name in marks.keys() {
                    if !teachers.iter().any(|t| t.subject_title == *name) {
                        count += 1;
                    }
                }
                count.to_string()
            }),
            stat_block::render(state, "Оценок", move || {
                state.quarter_marks.get().values().map(|v| v.len()).sum::<usize>().to_string()
            }),
        ))
        .spacing(8.0)
        .padding(Insets { top: 0.0, leading: PAD, bottom: 8.0, trailing: PAD }),

        year_avg_row(state, "Средний (все оценки)", move || {
            let marks = state.quarter_marks.get();
            let total: usize = marks.values().map(|v| v.len()).sum();
            if total == 0 { return "—".into(); }
            format!("{:.2}", marks.values().flat_map(|v| v.iter()).sum::<f64>() / total as f64)
        }),
        year_avg_row(state, "Средний (по четвертям)", move || {
            let yqd = state.year_quarter_data.get();
            if yqd.is_empty() { return "—".into(); }
            let q_avgs: Vec<f64> = yqd.iter().map(|(_, m)| {
                let t: usize = m.values().map(|v| v.len()).sum();
                if t == 0 { 0.0 } else { m.values().flat_map(|v| v.iter()).sum::<f64>() / t as f64 }
            }).filter(|a| *a > 0.0).collect();
            if q_avgs.is_empty() { "—".into() } else { format!("{:.2}", q_avgs.iter().sum::<f64>() / q_avgs.len() as f64) }
        }),
        year_avg_row(state, "Средний (взвешенный)", move || {
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

fn year_avg_row(state: AppState, lbl: &'static str, value: impl Fn() -> String + 'static) -> impl Piece {
    row((
        day::prelude::label(lbl).font(Font::Body).grow(),
        day::prelude::label(value).font(Font::Headline).color(move || Color::hex(state.accent_color.get())),
    ))
    .spacing(8.0)
    .padding(Insets { top: 4.0, leading: PAD, bottom: 4.0, trailing: PAD })
}

// ── Calculated: year per-quarter subjects ───────────────────────────────

fn year_quarter_subjects(state: AppState) -> impl Piece {
    each(
        items(
            move || {
                // Start with ALL subjects from subjects_teachers
                let teachers = state.subjects_teachers.get();
                let marks = state.quarter_marks.get();
                let mut subject_names: Vec<String> = teachers.iter()
                    .map(|t| t.subject_title.clone())
                    .collect();
                for name in marks.keys() {
                    if !subject_names.contains(name) {
                        subject_names.push(name.clone());
                    }
                }
                subject_names.dedup();
                subject_names.sort();
                subject_names
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
            }).font(Font::Headline).color(move || Color::hex(state.accent_color.get())),
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
                // Start with ALL subjects from subjects_teachers
                let teachers = state.subjects_teachers.get();
                let marks = state.quarter_marks.get();
                let mut subject_names: Vec<String> = teachers.iter()
                    .map(|t| t.subject_title.clone())
                    .collect();
                // Add any subjects from marks that aren't in teachers (shouldn't happen, but safety)
                for name in marks.keys() {
                    if !subject_names.contains(name) {
                        subject_names.push(name.clone());
                    }
                }
                subject_names.dedup();
                subject_names.sort();
                subject_names
            },
            |s: &String| s.clone(),
        ),
        move |item| {
            let subj_name = item.get();
            let s2 = subj_name.clone();
            row((
                label(subj_name).font(Font::Body).grow(),
                {
                    let marks = state.quarter_marks.get();
                    match marks.get(&*s2) {
                        Some(vals) if !vals.is_empty() => {
                            let avg = vals.iter().sum::<f64>() / vals.len() as f64;
                            label(format!("{:.1}", avg)).font(Font::Headline)
                                .color(if avg >= 4.0 { colors::SUCCESS } else if avg >= 3.0 { colors::WARNING } else { colors::ERROR })
                        }
                        _ => label("—").font(Font::Headline).secondary(),
                    }
                },
                {
                    let marks = state.quarter_marks.get();
                    let count = marks.get(&s2).map(|v| v.len()).unwrap_or(0);
                    label(format!("({})", count)).font(Font::Caption).secondary()
                },
            ))
            .spacing(8.0)
            .padding(Insets { top: 6.0, leading: PAD, bottom: 6.0, trailing: PAD })
            .any()
        },
    )
}

// ── Structs ────────────────────────────────────────────────────────────
