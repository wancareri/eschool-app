use crate::shared::nslog;
use crate::app::{AppState, OfficialMark};
use crate::features;
use eschool_api::entities::*;
use crate::shared::{colors, utils};
use crate::widgets;
use crate::res;
use day::prelude::*;
use day_piece_pullrefresh::pull_to_refresh;

const PAD: f64 = 20.0;
const SWIPE_THRESHOLD: f64 = 30.0;
const SWIPE_AXIS_LOCK: f64 = 4.0;
const SWIPE_EDGE_DAMP: f64 = 0.3;



pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let show_summary = Signal::new(false);
    let refreshing = Signal::new(false);
    let page_width = Signal::new(400.0f64);
    let strip_tx = Signal::new(-400.0f64);
                

zstack((
        spacer().background(Color::rgba(255.0, 255.0, 255.0, 0.01)).grow(),
        pull_to_refresh(refreshing, scroll(column((
        zstack((
            column((
                label(move || res::str::diary_title().format())
                    .font(Font::LargeTitle)
                    .align(TextAlign::Center),
            ))
            .spacing(6.0)
            .padding(Insets { top: 16.0, leading: PAD, bottom: 4.0, trailing: PAD }),
            
            // Connection indicator overlay in top-left
            widgets::conn_status::render(),
        )),

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

        // Old spinners removed. The connection status indicator now handles this:
        
        when(
            move || state.is_authenticated.get() && !show_summary.get(),
            move || week_view(state, page_width, strip_tx),
        ),
        when(
            move || state.is_authenticated.get() && show_summary.get(),
            move || summary_view(state, page_width, strip_tx),
        ),
    ))
    .spacing(0.0)
    .background(Color::CLEAR)
    .grow())
    .grow())
    .on_refresh(move || {
        let state = AppState::ambient();
        if state.is_authenticated.get() {
            features::diary::load_all(state);
        }
    })
    )) // close zstack
    .on_drag(global_drag(strip_tx.clone(), page_width.clone(), state.clone(), show_summary.clone()))
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
    select_quarter(state, q);
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
            select_quarter(s3, q);
        })
        .id("sub-summary"),
    ))
    .spacing(12.0)
    .padding(Insets { top: 0.0, leading: 16.0, bottom: 8.0, trailing: 16.0 })
}

// ── Week view ──────────────────────────────────────────────────────────

fn lessons_at(state: AppState, offset: i32) -> Vec<DaySchedule> {
    if offset == 0 {
        state.lessons.get()
    } else {
        let idx = state.current_week_index.get() + offset;
        state.week_cache.get().get(&idx).cloned().unwrap_or_default()
    }
}

fn pager_go(state: AppState, page_width: Signal<f64>, strip_tx: Signal<f64>, dir: i32) {
    let idx = state.current_week_index.get();
    let total = state.all_weeks.get().len() as i32;
    let new_idx = idx + dir;
    if new_idx < 0 || new_idx >= total {
        return;
    }
    let w = page_width.get();
    features::diary::load_week(state, new_idx);
    if dir > 0 {
        strip_tx.set(strip_tx.get() + w);
    } else {
        strip_tx.set(strip_tx.get() - w);
    }
    with_animation(AnimSpec::ease_out(200), || {
        strip_tx.set(-w);
    });
}




fn global_drag(
    strip_tx: Signal<f64>,
    page_width: Signal<f64>,
    state: AppState,
    show_summary: Signal<bool>,
) -> impl Fn(Drag) + 'static {
    let axis: Signal<Option<bool>> = Signal::new(None);
    move |drag: Drag| {
        let dx = drag.translation.x;
        let dy = drag.translation.y;
        let w = page_width.get();
        match drag.phase {
            DragPhase::Began => {
                axis.set(None);
                strip_tx.set(-w);
            }
            DragPhase::Changed => {
                let mut horiz = axis.get();
                if horiz.is_none() && (dx.abs() > SWIPE_AXIS_LOCK || dy.abs() > SWIPE_AXIS_LOCK) {
                    horiz = Some(dx.abs() >= dy.abs());
                    axis.set(horiz);
                }
                if horiz == Some(true) {
                    let mut x = dx;
                    if show_summary.get() {
                        let q = state.current_quarter.get();
                        if q == 0 && x > 0.0 { x *= SWIPE_EDGE_DAMP; }
                        if q >= 4 && x < 0.0 { x *= SWIPE_EDGE_DAMP; }
                    } else {
                        let idx = state.current_week_index.get();
                        let total = state.all_weeks.get().len() as i32;
                        if idx <= 0 && x > 0.0 { x *= SWIPE_EDGE_DAMP; }
                        if idx + 1 >= total && x < 0.0 { x *= SWIPE_EDGE_DAMP; }
                    }
                    strip_tx.set(-w + x);
                }
            }
            DragPhase::Ended => {
                let was_horiz = axis.get() == Some(true);
                axis.set(None);
                let actual_dx = strip_tx.get() + w;
                if was_horiz && actual_dx.abs() >= SWIPE_THRESHOLD {
                    if actual_dx < 0.0 {
                        // Swipe left -> next
                        if show_summary.get() {
                            let q = state.current_quarter.get();
                            if q < 4 {
                                state.current_quarter.set(q + 1);
                                strip_tx.set(strip_tx.get() + w);
                                with_animation(AnimSpec::ease_out(200), || {
                                    strip_tx.set(-w);
                                });
                                return;
                            }
                        } else {
                            let idx = state.current_week_index.get();
                            let total = state.all_weeks.get().len() as i32;
                            if idx + 1 < total {
                                features::diary::load_week(state, idx + 1);
                                strip_tx.set(strip_tx.get() + w);
                                with_animation(AnimSpec::ease_out(200), || {
                                    strip_tx.set(-w);
                                });
                                return;
                            }
                        }
                    }
                    if actual_dx > 0.0 {
                        // Swipe right -> prev
                        if show_summary.get() {
                            let q = state.current_quarter.get();
                            if q > 0 {
                                state.current_quarter.set(q - 1);
                                strip_tx.set(strip_tx.get() - w);
                                with_animation(AnimSpec::ease_out(200), || {
                                    strip_tx.set(-w);
                                });
                                return;
                            }
                        } else {
                            let idx = state.current_week_index.get();
                            if idx > 0 {
                                features::diary::load_week(state, idx - 1);
                                strip_tx.set(strip_tx.get() - w);
                                with_animation(AnimSpec::ease_out(200), || {
                                    strip_tx.set(-w);
                                });
                                return;
                            }
                        }
                    }
                }
                if was_horiz {
                    with_animation(AnimSpec::ease_out(200), || {
                        strip_tx.set(-w);
                    });
                }
            }
        }
    }
}

fn pager_drag(
    strip_tx: Signal<f64>,
    page_width: Signal<f64>,
    state: AppState,
) -> impl Fn(Drag) + 'static {
    let axis: Signal<Option<bool>> = Signal::new(None);
    move |drag: Drag| {
        let dx = drag.translation.x;
        let dy = drag.translation.y;
        let w = page_width.get();
        match drag.phase {
            DragPhase::Began => {
                nslog::nslog("[Drag] Began");
                axis.set(None);
                strip_tx.set(-w);
            }
            DragPhase::Changed => {
                let mut horiz = axis.get();
                if horiz.is_none() && (dx.abs() > SWIPE_AXIS_LOCK || dy.abs() > SWIPE_AXIS_LOCK) {
                    horiz = Some(dx.abs() >= dy.abs());
                    nslog::nslog(&format!("[Drag] Axis locked: horiz={:?} dx={} dy={}", horiz, dx, dy));
                    axis.set(horiz);
                }
                if horiz == Some(true) {
                    let idx = state.current_week_index.get();
                    let total = state.all_weeks.get().len() as i32;
                    let mut x = dx;
                    if idx <= 0 && x > 0.0 {
                        x *= SWIPE_EDGE_DAMP;
                    }
                    if idx + 1 >= total && x < 0.0 {
                        x *= SWIPE_EDGE_DAMP;
                    }
                    strip_tx.set(-w + x);
                }
            }
            DragPhase::Ended => {
                let was_horiz = axis.get() == Some(true);
                axis.set(None);
                let idx = state.current_week_index.get();
                let total = state.all_weeks.get().len() as i32;
                let actual_dx = strip_tx.get() + w;
                nslog::nslog(&format!("[Drag] Ended: was_horiz={} actual_dx={} threshold={}", was_horiz, actual_dx, SWIPE_THRESHOLD));
                if was_horiz && actual_dx.abs() >= SWIPE_THRESHOLD {
                    if actual_dx < 0.0 && idx + 1 < total {
                        features::diary::load_week(state, idx + 1);
                        strip_tx.set(strip_tx.get() + w);
                        with_animation(AnimSpec::ease_out(200), || {
                            strip_tx.set(-w);
                        });
                        return;
                    }
                    if actual_dx > 0.0 && idx > 0 {
                        features::diary::load_week(state, idx - 1);
                        strip_tx.set(strip_tx.get() - w);
                        with_animation(AnimSpec::ease_out(200), || {
                            strip_tx.set(-w);
                        });
                        return;
                    }
                }
                if was_horiz {
                    with_animation(AnimSpec::ease_out(200), || {
                        strip_tx.set(-w);
                    });
                }
            }
        }
    }
}

fn summary_drag(
    strip_tx: Signal<f64>,
    page_width: Signal<f64>,
    state: AppState,
) -> impl Fn(Drag) + 'static {
    let axis: Signal<Option<bool>> = Signal::new(None);
    move |drag: Drag| {
        let dx = drag.translation.x;
        let dy = drag.translation.y;
        let w = page_width.get();
        match drag.phase {
            DragPhase::Began => {
                nslog::nslog("[Drag] Began");
                axis.set(None);
                strip_tx.set(-w);
            }
            DragPhase::Changed => {
                let mut horiz = axis.get();
                if horiz.is_none() && (dx.abs() > SWIPE_AXIS_LOCK || dy.abs() > SWIPE_AXIS_LOCK) {
                    horiz = Some(dx.abs() >= dy.abs());
                    nslog::nslog(&format!("[Drag] Axis locked: horiz={:?} dx={} dy={}", horiz, dx, dy));
                    axis.set(horiz);
                }
                if horiz == Some(true) {
                    let q = state.current_quarter.get();
                    let mut x = dx;
                    if q <= 0 && x > 0.0 {
                        x *= SWIPE_EDGE_DAMP;
                    }
                    if q >= 4 && x < 0.0 {
                        x *= SWIPE_EDGE_DAMP;
                    }
                    strip_tx.set(-w + x);
                }
            }
            DragPhase::Ended => {
                let was_horiz = axis.get() == Some(true);
                axis.set(None);
                let q = state.current_quarter.get();
                let actual_dx = strip_tx.get() + w;
                if was_horiz && actual_dx.abs() >= SWIPE_THRESHOLD {
                    if actual_dx < 0.0 && q + 1 <= 4 {
                        select_quarter(state, q + 1);
                        strip_tx.set(strip_tx.get() + w);
                        with_animation(AnimSpec::ease_out(200), || {
                            strip_tx.set(-w);
                        });
                        return;
                    }
                    if actual_dx > 0.0 && q > 0 {
                        select_quarter(state, q - 1);
                        strip_tx.set(strip_tx.get() - w);
                        with_animation(AnimSpec::ease_out(200), || {
                            strip_tx.set(-w);
                        });
                        return;
                    }
                }
                if was_horiz {
                    with_animation(AnimSpec::ease_out(200), || {
                        strip_tx.set(-w);
                    });
                }
            }
        }
    }
}

fn week_view(state: AppState, page_width: Signal<f64>, strip_tx: Signal<f64>) -> impl Piece {
    let probe_width = page_width;
    let probe_tx = strip_tx;
    let strip_state = state;
    let header_state = state;
    let header_width = page_width;
    let header_tx = strip_tx;
    column((
        week_header(header_state, header_width, header_tx),
        canvas(move |_draw, size| {
            let w = size.width;
            if w > 1.0 && (probe_width.get() - w).abs() > 0.5 {
                let old_w = probe_width.get();
                let tx = probe_tx.get();
                probe_width.set(w);
                if (tx + old_w).abs() < 2.0 {
                    probe_tx.set(-w);
                }
            }
        })
        .height(0.0),

        when(
            move || {
                page_width.get();
                true
            },
            move || {
                let w = page_width.get();
                row((
                    row((
                        week_page(strip_state, -1, w),
                        week_page(strip_state, 0, w),
                        week_page(strip_state, 1, w),
                    ))
                    .translation(strip_tx, 0.0),
                ))
                .on_drag(pager_drag(strip_tx.clone(), page_width.clone(), state.clone()))
                .grow()
            },
        ),
    ))
    .spacing(0.0)
    .grow()
}

fn week_page(state: AppState, offset: i32, w: f64) -> impl Piece {
    let get_lessons = move || lessons_at(state, offset);
    column((
        when(
            move || true,
            move || widgets::week_summary::render(state, get_lessons),
        ),
        when(
            move || get_lessons().is_empty() && offset == 0 && state.lessons_loading.get(),
            || column((
                spinner(),
            ))
            .padding(Insets { top: 80.0, leading: 0.0, bottom: 80.0, trailing: 0.0 }),
        ),
        when(
            move || get_lessons().is_empty() && (!state.lessons_loading.get() || offset != 0),
            || label("Нет данных за этот период")
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center)
                .padding(Insets { top: 80.0, leading: PAD, bottom: 80.0, trailing: PAD }),
        ),
        diary_list_with(state, get_lessons),
    ))
    .spacing(0.0)
    .width(w)
}

fn week_header(state: AppState, page_width: Signal<f64>, strip_tx: Signal<f64>) -> impl Piece {
    let s1 = state;
    let pw1 = page_width;
    let st1 = strip_tx;
    let s2 = state;
    let pw2 = page_width;
    let st2 = strip_tx;
    row((
        button("<")
            .action(move || pager_go(s1, pw1, st1, -1))
            .id("wk-prev")
            .frame(44.0, 36.0),
        spacer().grow(),
        label(move || strip_week_summary(&state.current_week.get()))
            .font(Font::Headline)
            .align(TextAlign::Center),
        spacer().grow(),
        button(">")
            .action(move || pager_go(s2, pw2, st2, 1))
            .id("wk-next")
            .frame(44.0, 36.0),
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
    result.trim().to_string()
}

fn diary_list_with(
    state: AppState,
    get_lessons: impl Fn() -> Vec<DaySchedule> + Copy + 'static,
) -> impl Piece {
    each(
        items(move || get_lessons(), |d: &DaySchedule| d.date),
        move |day_slot| {
            let date = day_slot.key();
            day_card_with(state, get_lessons, date).any()
        },
    )
}

fn day_card_with(
    state: AppState,
    get_lessons: impl Fn() -> Vec<DaySchedule> + Copy + 'static,
    date: u64,
) -> impl Piece {
    let get_slots = move || {
        get_lessons()
            .into_iter()
            .find(|d| d.date == date)
            .map(|d| d.slots)
            .unwrap_or_default()
    };
    column((
        label(move || {
            get_lessons()
                .iter()
                .find(|d| d.date == date)
                .map(|d| utils::format_date_header(d.day_of_week, d.date))
                .unwrap_or_default()
        })
        .font(Font::Headline)
        .color(move || Color::hex(state.accent_color.get()))
        .padding(Insets { top: 16.0, leading: PAD, bottom: 6.0, trailing: PAD }).align(TextAlign::Leading),
        each(
            items(get_slots, |s: &LessonSlot| s.number),
            move |slot| {
                let num = slot.key();
                widgets::lesson_row::render(get_lessons, date, num).any()
            },
        ),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
}

// ── Summary view ───────────────────────────────────────────────────────



fn summary_view(state: AppState, page_width: Signal<f64>, strip_tx: Signal<f64>) -> impl Piece {
    let probe_width = page_width;
    let probe_tx = strip_tx;
    let strip_state = state;
    column((
        canvas(move |_draw, size| {
            let w = size.width;
            if w > 1.0 && (probe_width.get() - w).abs() > 0.5 {
                let old_w = probe_width.get();
                let tx = probe_tx.get();
                probe_width.set(w);
                if (tx + old_w).abs() < 2.0 {
                    probe_tx.set(-w);
                }
            }
        })
        .height(0.0),

        when(
            move || {
                page_width.get();
                true
            },
            move || {
                let w = page_width.get();
                row((
                    row((
                        summary_page(strip_state, -1, w),
                        summary_page(strip_state, 0, w),
                        summary_page(strip_state, 1, w),
                    ))
                    .translation(strip_tx, 0.0),
                ))
                .on_drag(summary_drag(strip_tx.clone(), page_width.clone(), state.clone()))
            },
        ),
    ))
    .spacing(0.0)
}

fn summary_page(state: AppState, offset: i32, w: f64) -> impl Piece {
    let q = (state.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
    let title_state = state;
    let header_state = state;
    let qs_state = state;
    let ys_state = state;
    column((
        label(move || {
            let q = (title_state.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
            if q == 4 { "Итоги года" } else { "Итоги четверти" }
        })
        .font(Font::Headline)
        .color(move || Color::hex(state.accent_color.get()))
        .align(TextAlign::Center)
        .padding(Insets { top: 16.0, leading: PAD, bottom: 2.0, trailing: PAD }),
        
        {
            let s = state;
            label(move || {
                let q = (s.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                let off: std::collections::HashMap<String, Vec<OfficialMark>> = if offset == 0 {
                    s.official_marks.get()
                } else {
                    s.quarter_official_marks.get().get(q).cloned().unwrap_or_default()
                };
                let mut sum = 0.0;
                let mut count = 0;
                for marks in off.values() {
                    for m in marks {
                        sum += m.value;
                        count += 1;
                    }
                }
                if count > 0 {
                    format!("Средний балл: {:.2}", sum / count as f64)
                } else {
                    "Средний балл: —".to_string()
                }
            })
            .font(Font::Subheadline)
            .secondary()
            .align(TextAlign::Center)
            .padding(Insets { top: 0.0, leading: PAD, bottom: 10.0, trailing: PAD })
        },
        
        summary_header_for(header_state, offset),
        divider().padding(Insets { top: 0.0, leading: PAD, bottom: 0.0, trailing: PAD }),
        when(
            move || {
                let q = (title_state.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                q != 4
            },
            move || quarter_summary_at(qs_state, q, offset),
        ),
        when(
            move || {
                let q = (title_state.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                q == 4
            },
            move || year_summary_at(ys_state),
        ),
    ))
    .spacing(0.0)
    .width(w)
}

fn summary_header_for(state: AppState, offset: i32) -> impl Piece {
    row((
        label("Предмет").font(Font::Caption).secondary().grow(),
        label("Выст.").font(Font::Caption).secondary().frame(50.0, 0.0).align(TextAlign::Center),
        label("Вых.").font(Font::Caption).secondary().frame(50.0, 0.0).align(TextAlign::Center),
        when(
            move || {
                let q = (state.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                q == 4
            },
            || row((
                label("I").font(Font::Caption).secondary().frame(38.0, 0.0).align(TextAlign::Center),
                label("II").font(Font::Caption).secondary().frame(38.0, 0.0).align(TextAlign::Center),
                label("III").font(Font::Caption).secondary().frame(38.0, 0.0).align(TextAlign::Center),
                label("IV").font(Font::Caption).secondary().frame(38.0, 0.0).align(TextAlign::Center),
            ))
            .spacing(4.0),
        ),
    ))
    .spacing(8.0)
    .padding(Insets { top: 4.0, leading: PAD, bottom: 4.0, trailing: PAD })
}

fn quarter_summary_at(state: AppState, q: usize, offset: i32) -> impl Piece {
    let marks_state = state;
    let off_state = state;
    each(
        items(
            move || {
                let teachers = state.subjects_teachers.get();
                let marks: std::collections::HashMap<String, Vec<f64>> = if offset == 0 {
                    state.quarter_marks.get()
                } else {
                    state.quarter_all_marks.get().get(q).cloned().unwrap_or_default()
                };
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
                    let off: std::collections::HashMap<String, Vec<OfficialMark>> = if offset == 0 {
                        off_state.official_marks.get()
                    } else {
                        off_state.quarter_official_marks.get().get(q).cloned().unwrap_or_default()
                    };
                    let (text, ok) = mark_label(official_avg(&off, &sj));
                    label(text).font(Font::Headline).frame(50.0, 0.0).align(TextAlign::Center)
                        .color(if ok { colors::SUCCESS } else { colors::SECONDARY })
                },
                {
                    let marks: std::collections::HashMap<String, Vec<f64>> = if offset == 0 {
                        marks_state.quarter_marks.get()
                    } else {
                        marks_state.quarter_all_marks.get().get(q).cloned().unwrap_or_default()
                    };
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

fn year_summary_at(state: AppState) -> impl Piece {
    let marks_state = state;
    let off_state = state;
    let cell_state = state;
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
                        let off = off_state.official_marks.get();
                        let (text, ok) = mark_label(official_avg(&off, &sj));
                        label(text).font(Font::Headline).frame(50.0, 0.0).align(TextAlign::Center)
                            .color(if ok { colors::SUCCESS } else { colors::SECONDARY })
                    },
                    {
                        let marks = marks_state.quarter_marks.get();
                        let (text, ok) = mark_label(marks_avg(&marks, &sj2));
                        let c = if ok { colors::SUCCESS } else if text != "—" { colors::WARNING } else { colors::SECONDARY };
                        label(text).font(Font::Headline).frame(50.0, 0.0).align(TextAlign::Center).color(c)
                    },
                    year_quarter_cells(cell_state, sj3),
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
