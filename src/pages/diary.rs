use crate::app::{AppState, OfficialMark};
use crate::features;
use eschool_api::entities::*;
use crate::shared::{colors, utils};
use crate::widgets;
use crate::res;
use day::prelude::*;
use day_piece_pullrefresh::pull_to_refresh;

const PAD: f64 = 20.0;
const SWIPE_THRESHOLD: f64 = 72.0;
const SWIPE_AXIS_LOCK: f64 = 4.0;
const SWIPE_EDGE_DAMP: f64 = 0.3;

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

fn select_quarter(state: AppState, q: usize) {
    if q == 4 {
        features::diary::load_year(state);
        return;
    }
    // Show existing per-quarter data immediately
    let q_all = state.quarter_all_marks.get();
    if let Some(marks) = q_all.get(q) {
        if !marks.is_empty() {
            state.quarter_marks.set(marks.clone());
            let q_off = state.quarter_official_marks.get();
            if let Some(off) = q_off.get(q) {
                state.official_marks.set(off.clone());
            }
        }
    }
    state.current_quarter.set(q);
    features::diary::load_quarter(state, q);
}

fn quarter_btn(state: AppState, lbl: &'static str, q: usize) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let l = lbl.to_string();
    button(move || {
        let cur = s1.current_quarter.get();
        if cur == q && !s1.marks_loading.get() { format!("[{}]", l) } else { l.clone() }
    })
    .action(move || select_quarter(s2, q))
    .id(format!("q-{lbl}"))
}

fn year_btn(state: AppState) -> impl Piece {
    let s1 = state;
    let s2 = state;
    button(move || {
        let cur = s1.current_quarter.get();
        if cur == 4 && !s1.marks_loading.get() { String::from("[Год]") } else { String::from("Год") }
    })
    .action(move || select_quarter(s2, 4))
    .id("q-year")
}

fn swipe_drag(
    offset_x: Signal<f64>,
    map_dx: impl Fn(f64) -> f64 + 'static,
    on_swipe: impl Fn(f64) + 'static,
) -> impl Fn(Drag) + 'static {
    // None = undecided, Some(true) = horizontal, Some(false) = vertical (ignore)
    let axis: Signal<Option<bool>> = Signal::new(None);
    move |drag: Drag| {
        let dx = drag.translation.x;
        let dy = drag.translation.y;
        match drag.phase {
            DragPhase::Began => {
                axis.set(None);
                offset_x.set(0.0);
            }
            DragPhase::Changed => {
                let mut horiz = axis.get();
                if horiz.is_none() && (dx.abs() > SWIPE_AXIS_LOCK || dy.abs() > SWIPE_AXIS_LOCK) {
                    horiz = Some(dx.abs() >= dy.abs());
                    axis.set(horiz);
                }
                if horiz == Some(true) {
                    offset_x.set(map_dx(dx));
                }
            }
            DragPhase::Ended => {
                let was_horiz = axis.get() == Some(true);
                axis.set(None);
                if was_horiz && dx.abs() >= SWIPE_THRESHOLD {
                    on_swipe(dx);
                }
                with_animation(AnimSpec::ease_out(200), || {
                    offset_x.set(0.0);
                });
            }
        }
    }
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
    let offset_x = Signal::new(0.0f64);
    let edge_state = state;
    let swipe_state = state;
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
    // Swipe left → next week, right → previous week
    .translation(offset_x, 0.0)
    .on_drag(swipe_drag(
        offset_x,
        move |dx: f64| {
            let idx = edge_state.current_week_index.get();
            let total = edge_state.all_weeks.get().len() as i32;
            let mut x = dx;
            if idx <= 0 && x > 0.0 {
                x *= SWIPE_EDGE_DAMP;
            }
            if idx + 1 >= total && x < 0.0 {
                x *= SWIPE_EDGE_DAMP;
            }
            x
        },
        move |dx: f64| {
            let idx = swipe_state.current_week_index.get();
            let total = swipe_state.all_weeks.get().len() as i32;
            if dx < 0.0 && idx + 1 < total {
                features::diary::load_week(swipe_state, idx + 1);
            } else if dx > 0.0 && idx > 0 {
                features::diary::load_week(swipe_state, idx - 1);
            }
        },
    ))
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
    let offset_x = Signal::new(0.0f64);
    let edge_state = state;
    let swipe_state = state;
    column((
        label(move || {
            if state.current_quarter.get() == 4 { "Итоги года" } else { "Итоги четверти" }
        })
        .font(Font::Headline).color(move || Color::hex(state.accent_color.get()))
        .align(TextAlign::Center)
        .padding(Insets { top: 16.0, leading: PAD, bottom: 10.0, trailing: PAD }),

        summary_header(state),

        divider().padding(Insets { top: 0.0, leading: PAD, bottom: 0.0, trailing: PAD }),

        when(move || state.current_quarter.get() != 4, move || quarter_summary_subjects(state)),
        when(move || state.current_quarter.get() == 4, move || year_summary_subjects(state)),
    ))
    .spacing(0.0)
    .grow()
    // Swipe left → next quarter/year, right → previous
    .translation(offset_x, 0.0)
    .on_drag(swipe_drag(
        offset_x,
        move |dx: f64| {
            let q = edge_state.current_quarter.get();
            let mut x = dx;
            if q <= 0 && x > 0.0 {
                x *= SWIPE_EDGE_DAMP;
            }
            if q >= 4 && x < 0.0 {
                x *= SWIPE_EDGE_DAMP;
            }
            x
        },
        move |dx: f64| {
            let q = swipe_state.current_quarter.get();
            if dx < 0.0 && q < 4 {
                select_quarter(swipe_state, q + 1);
            } else if dx > 0.0 && q > 0 {
                select_quarter(swipe_state, q - 1);
            }
        },
    ))
}

fn summary_header(state: AppState) -> impl Piece {
    row((
        label("Предмет").font(Font::Caption).secondary().grow(),
        label("Выст.").font(Font::Caption).secondary().frame(50.0, 0.0).align(TextAlign::Center),
        label("Вых.").font(Font::Caption).secondary().frame(50.0, 0.0).align(TextAlign::Center),
        when(
            move || state.current_quarter.get() == 4,
            move || row((
                label("I").font(Font::Caption).secondary().frame(38.0, 0.0).align(TextAlign::Center),
                label("II").font(Font::Caption).secondary().frame(38.0, 0.0).align(TextAlign::Center),
                label("III").font(Font::Caption).secondary().frame(38.0, 0.0).align(TextAlign::Center),
                label("IV").font(Font::Caption).secondary().frame(38.0, 0.0).align(TextAlign::Center),
            )).spacing(4.0),
        ),
    ))
    .spacing(8.0)
    .padding(Insets { top: 4.0, leading: PAD, bottom: 4.0, trailing: PAD })
}

fn official_avg(marks: &std::collections::HashMap<String, Vec<OfficialMark>>, subject: &str) -> Option<f64> {
    marks.get(subject).and_then(|vals| {
        if vals.is_empty() { None }
        else { Some(vals.iter().map(|m| m.value).sum::<f64>() / vals.len() as f64) }
    })
}

fn marks_avg(marks: &std::collections::HashMap<String, Vec<f64>>, subject: &str) -> Option<f64> {
    marks.get(subject).and_then(|vals| {
        if vals.is_empty() { None }
        else { Some(vals.iter().sum::<f64>() / vals.len() as f64) }
    })
}

fn mark_label(avg: Option<f64>) -> (String, bool) {
    match avg {
        Some(v) => (format!("{:.1}", v), v >= 4.0),
        None => ("—".into(), false),
    }
}

// ── Quarter summary subjects ───────────────────────────────────────────

fn quarter_summary_subjects(state: AppState) -> impl Piece {
    each(
        items(
            move || {
                let teachers = state.subjects_teachers.get();
                let marks = state.quarter_marks.get();
                let mut names: Vec<String> = teachers.iter()
                    .map(|t| t.subject_title.clone())
                    .collect();
                for name in marks.keys() {
                    if !names.contains(name) {
                        names.push(name.clone());
                    }
                }
                names.dedup();
                names.sort();
                names
            },
            |s: &String| s.clone(),
        ),
        move |item| {
            let subj_name = item.get();
            let sj = subj_name.clone();
            let sj2 = subj_name.clone();

            row((
                label(subj_name).font(Font::Body).grow(),

                {
                    let off = state.official_marks.get();
                    let (text, ok) = mark_label(official_avg(&off, &sj));
                    label(text).font(Font::Headline).frame(50.0, 0.0).align(TextAlign::Center)
                        .color(if ok { colors::SUCCESS } else { colors::SECONDARY })
                },

                {
                    let marks = state.quarter_marks.get();
                    let (text, ok) = mark_label(marks_avg(&marks, &sj2));
                    let c = if ok { colors::SUCCESS } else if text != "—" { colors::WARNING } else { colors::SECONDARY };
                    label(text).font(Font::Headline).frame(50.0, 0.0).align(TextAlign::Center).color(c)
                },
            ))
            .spacing(8.0)
            .padding(Insets { top: 6.0, leading: PAD, bottom: 6.0, trailing: PAD })
            .any()
        },
    )
}

// ── Year summary subjects ──────────────────────────────────────────────

fn year_summary_subjects(state: AppState) -> impl Piece {
    each(
        items(
            move || {
                let teachers = state.subjects_teachers.get();
                let marks = state.quarter_marks.get();
                let mut names: Vec<String> = teachers.iter()
                    .map(|t| t.subject_title.clone())
                    .collect();
                for name in marks.keys() {
                    if !names.contains(name) {
                        names.push(name.clone());
                    }
                }
                names.dedup();
                names.sort();
                names
            },
            |s: &String| s.clone(),
        ),
        move |item| {
            let subj_name = item.get();
            let sj = subj_name.clone();
            let sj2 = subj_name.clone();
            let sj3 = subj_name.clone();

            column((
                row((
                    label(subj_name).font(Font::Body).grow(),

                    {
                        let off = state.official_marks.get();
                        let (text, ok) = mark_label(official_avg(&off, &sj));
                        label(text).font(Font::Headline).frame(50.0, 0.0).align(TextAlign::Center)
                            .color(if ok { colors::SUCCESS } else { colors::SECONDARY })
                    },

                    {
                        let marks = state.quarter_marks.get();
                        let (text, ok) = mark_label(marks_avg(&marks, &sj2));
                        let c = if ok { colors::SUCCESS } else if text != "—" { colors::WARNING } else { colors::SECONDARY };
                        label(text).font(Font::Headline).frame(50.0, 0.0).align(TextAlign::Center).color(c)
                    },

                    year_quarter_cells(state, sj3),
                ))
                .spacing(8.0)
                .padding(Insets { top: 6.0, leading: PAD, bottom: 2.0, trailing: PAD }),

                divider(),
            ))
            .spacing(0.0)
            .any()
        },
    )
}

fn year_quarter_cells(state: AppState, subject: String) -> impl Piece {
    let s = state;
    let sj = subject;
    row((
        year_q_cell(s, sj.clone(), 0),
        year_q_cell(s, sj.clone(), 1),
        year_q_cell(s, sj.clone(), 2),
        year_q_cell(s, sj, 3),
    ))
    .spacing(4.0)
}

fn year_q_cell(state: AppState, subject: String, q: usize) -> impl Piece {
    column((
        day::prelude::label(move || {
            let yqd = state.year_quarter_data.get();
            if let Some((_, m)) = yqd.get(q) {
                if let Some(v) = m.get(&*subject) {
                    if v.is_empty() { "—".into() } else { format!("{:.1}", v.iter().sum::<f64>() / v.len() as f64) }
                } else { "—".into() }
            } else { "—".into() }
        }).font(Font::Caption).align(TextAlign::Center),
    ))
    .frame(38.0, 0.0)
}

// ── Structs ────────────────────────────────────────────────────────────
