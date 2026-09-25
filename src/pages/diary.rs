use crate::app::{AppState, OfficialMark};
use crate::features;
use eschool_api::entities::*;
use crate::shared::{colors, utils};
use crate::widgets;
use crate::res;
use day::prelude::*;
use day_piece_pullrefresh::pull_to_refresh;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

const PAD: f64 = 20.0;
const SWIPE_THRESHOLD: f64 = 36.0;
const SWIPE_AXIS_LOCK: f64 = 8.0;
const SWIPE_EDGE_DAMP: f64 = 0.3;

// A swipe's `load_week` lands only after its slide thread sleeps, so a swipe
// inside that window reads the stale index and targets the same week. One
// flight at a time; extra swipes queue here and chain after the load.
static WEEK_SWIPE_BUSY: AtomicBool = AtomicBool::new(false);
static WEEK_SWIPE_QUEUE: AtomicI32 = AtomicI32::new(0);

pub fn get_screen_width() -> f64 {
    #[cfg(target_os = "ios")]
    {
        use objc2::MainThreadMarker;
        use objc2_ui_kit::UIScreen;
        if let Some(mtm) = MainThreadMarker::new() {
            let screen = UIScreen::mainScreen(mtm);
            let bounds = screen.bounds();
            if bounds.size.width > 50.0 {
                return bounds.size.width as f64;
            }
        }
    }
    390.0
}

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let show_summary = Signal::new(false);
    let refreshing = Signal::new(false);
    let initial_w = get_screen_width();
    let page_width = Signal::new(initial_w);
    let drag_x = Signal::new(0.0);

    zstack((
        pull_to_refresh(refreshing, scroll(column((
                column((
                    label(move || res::str::diary_title().format())
                        .font(Font::LargeTitle)
                        .align(TextAlign::Center),
                ))
                .spacing(6.0)
                .padding(Insets { top: 8.0, leading: PAD, bottom: 4.0, trailing: PAD }),

                quarter_tabs(state),
                sub_tabs(state, show_summary),

                when(
                    move || !show_summary.get(),
                    move || week_view(state, page_width, drag_x),
                ),
                when(
                    move || show_summary.get(),
                    move || summary_view(state, page_width, drag_x),
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
            .grow(),

        widgets::conn_status::render()
            .padding(Insets { top: 16.0, leading: 16.0, bottom: 0.0, trailing: 0.0 }),
    ))
    .align(Alignment::TopLeading)
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
        state.week_cache.with(|cache| cache.get(&idx).cloned().unwrap_or_default())
    }
}

fn pager_go(state: AppState, page_width: Signal<f64>, drag_x: Signal<f64>, dir: i32) {
    start_week_swipe(state, drag_x, page_width, dir, 280);
}

fn start_week_swipe(
    state: AppState,
    drag_x: Signal<f64>,
    page_width: Signal<f64>,
    dir: i32,
    dur_ms: u32,
) {
    if WEEK_SWIPE_BUSY.load(Ordering::Relaxed) {
        WEEK_SWIPE_QUEUE.fetch_add(dir, Ordering::Relaxed);
        return;
    }
    let idx = state.current_week_index.get();
    let total = state.all_weeks.get().len() as i32;
    let new_idx = idx + dir;
    if new_idx < 0 || new_idx >= total {
        with_animation(AnimSpec::ease_out(180), || {
            drag_x.set(0.0);
        });
        return;
    }
    WEEK_SWIPE_BUSY.store(true, Ordering::Relaxed);
    let w = page_width.get();
    let target = if dir > 0 { -w } else { w };
    with_animation(AnimSpec::ease_out(dur_ms), || {
        drag_x.set(target);
    });
    spawn_week_slide(drag_x.setter(), w, dur_ms, new_idx);
}

fn spawn_week_slide(set_drag: day::reactive::Setter<f64>, w: f64, dur_ms: u32, new_idx: i32) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(dur_ms as u64));
        day::reactive::on_main(move || {
            day::reactive::batch(|| {
                if let Some(state) = AppState::get_main() {
                    features::diary::load_week(state, new_idx);
                }
                set_drag.set(0.0);
            });
            let q = WEEK_SWIPE_QUEUE.swap(0, Ordering::Relaxed);
            if q == 0 {
                WEEK_SWIPE_BUSY.store(false, Ordering::Relaxed);
                return;
            }
            let dir = q.signum();
            if q.abs() > 1 {
                WEEK_SWIPE_QUEUE.store(q - dir, Ordering::Relaxed);
            }
            let Some(state) = AppState::get_main() else {
                WEEK_SWIPE_BUSY.store(false, Ordering::Relaxed);
                return;
            };
            let idx = state.current_week_index.get();
            let total = state.all_weeks.get().len() as i32;
            if idx + dir < 0 || idx + dir >= total {
                WEEK_SWIPE_QUEUE.store(0, Ordering::Relaxed);
                WEEK_SWIPE_BUSY.store(false, Ordering::Relaxed);
                return;
            }
            let target = if dir > 0 { -w } else { w };
            with_animation(AnimSpec::ease_out(dur_ms), || {
                set_drag.set(target);
            });
            spawn_week_slide(set_drag, w, dur_ms, idx + dir);
        });
    });
}

fn pager_drag(
    drag_x: Signal<f64>,
    page_width: Signal<f64>,
    state: AppState,
) -> impl Fn(Drag) + 'static {
    let axis: Signal<Option<bool>> = Signal::new(None);
    move |drag: Drag| {
        let dx = drag.translation.x;
        let dy = drag.translation.y;
        match drag.phase {
            DragPhase::Began => {
                axis.set(None);
                drag_x.set(0.0);
            }
            DragPhase::Changed => {
                let mut horiz = axis.get();
                if horiz.is_none() && (dx.abs() > SWIPE_AXIS_LOCK || dy.abs() > SWIPE_AXIS_LOCK) {
                    horiz = Some(dx.abs() >= dy.abs() * 0.5);
                    axis.set(horiz);
                }
                if horiz == Some(false) && dx.abs() > 20.0 && dx.abs() > dy.abs() * 1.0 {
                    horiz = Some(true);
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
                    drag_x.set(x);
                }
            }
            DragPhase::Ended => {
                let was_horiz = axis.get() == Some(true);
                axis.set(None);
                let actual_dx = drag_x.get();
                if was_horiz && actual_dx.abs() >= SWIPE_THRESHOLD {
                    let dir = if actual_dx < 0.0 { 1 } else { -1 };
                    start_week_swipe(state, drag_x, page_width, dir, 220);
                    return;
                }
                if was_horiz {
                    with_animation(AnimSpec::ease_out(180), move || {
                        drag_x.set(0.0);
                    });
                }
            }
        }
    }
}

fn summary_drag(
    drag_x: Signal<f64>,
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
                axis.set(None);
                drag_x.set(0.0);
            }
            DragPhase::Changed => {
                let mut horiz = axis.get();
                if horiz.is_none() && (dx.abs() > SWIPE_AXIS_LOCK || dy.abs() > SWIPE_AXIS_LOCK) {
                    horiz = Some(dx.abs() >= dy.abs() * 0.6);
                    axis.set(horiz);
                }
                if horiz == Some(false) && dx.abs() > 24.0 && dx.abs() > dy.abs() * 1.2 {
                    horiz = Some(true);
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
                    drag_x.set(x);
                }
            }
            DragPhase::Ended => {
                let was_horiz = axis.get() == Some(true);
                axis.set(None);
                let q = state.current_quarter.get();
                let actual_dx = drag_x.get();
                if was_horiz && actual_dx.abs() >= SWIPE_THRESHOLD {
                    if actual_dx < 0.0 && q + 1 <= 4 {
                        with_animation(AnimSpec::ease_out(220), move || {
                            drag_x.set(-w);
                        });
                        let set_drag = drag_x.setter();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(220));
                            day::reactive::on_main(move || {
                                day::reactive::batch(|| {
                                    if let Some(state) = AppState::get_main() {
                                        select_quarter(state, q + 1);
                                    }
                                    set_drag.set(0.0);
                                });
                            });
                        });
                        return;
                    }
                    if actual_dx > 0.0 && q > 0 {
                        with_animation(AnimSpec::ease_out(220), move || {
                            drag_x.set(w);
                        });
                        let set_drag = drag_x.setter();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(220));
                            day::reactive::on_main(move || {
                                day::reactive::batch(|| {
                                    if let Some(state) = AppState::get_main() {
                                        select_quarter(state, q - 1);
                                    }
                                    set_drag.set(0.0);
                                });
                            });
                        });
                        return;
                    }
                }
                if was_horiz {
                    with_animation(AnimSpec::ease_out(180), move || {
                        drag_x.set(0.0);
                    });
                }
            }
        }
    }
}

fn week_view(state: AppState, page_width: Signal<f64>, drag_x: Signal<f64>) -> impl Piece {
    let s_prev = state;
    let s_cur = state;
    let s_next = state;
    let header_state = state;
    let header_width = page_width;
    let header_dx = drag_x;
    let w = page_width.get();
    column((
        week_header(header_state, header_width, header_dx),
        zstack((
            // Previous week page (at drag_x - w)
            when(
                move || s_prev.current_week_index.get() > 0,
                move || week_page(s_prev, -1, w).translation(move || drag_x.get() - w, 0.0),
            ),
            // Next week page (at drag_x + w)
            when(
                move || s_next.current_week_index.get() + 1 < s_next.all_weeks.get().len() as i32,
                move || week_page(s_next, 1, w).translation(move || drag_x.get() + w, 0.0),
            ),
            // Current week page (at drag_x)
            week_page(s_cur, 0, w).translation(move || drag_x.get(), 0.0),
        ))
        .align(Alignment::TopLeading)
        .width(w)
        .on_drag(pager_drag(drag_x.clone(), page_width.clone(), state.clone())),
    ))
    .spacing(0.0)
    .width(w)
}

fn week_page(state: AppState, offset: i32, w: f64) -> impl Piece {
    let get_lessons = move || lessons_at(state, offset);
    let target_idx = move || state.current_week_index.get() + offset;
    let is_valid_week = move || {
        let ti = target_idx();
        let total = state.all_weeks.get().len() as i32;
        ti >= 0 && ti < total
    };
    let is_loaded = move || {
        let ti = target_idx();
        let total = state.all_weeks.get().len() as i32;
        if ti < 0 || ti >= total {
            return true;
        }
        if offset == 0 {
            !state.lessons.get().is_empty() || !state.lessons_loading.get()
        } else {
            state.week_cache.with(|c| c.contains_key(&ti))
        }
    };

    column((
        when(
            move || true,
            move || widgets::week_summary::render(state, get_lessons),
        ),
        when(
            move || is_valid_week() && !is_loaded(),
            move || column((
                widgets::spinner::render(state, 11.0),
            ))
            .align(HAlign::Center)
            .padding(Insets { top: 80.0, leading: 0.0, bottom: 80.0, trailing: 0.0 }),
        ),
        when(
            move || is_loaded() && get_lessons().is_empty(),
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

fn week_header(state: AppState, page_width: Signal<f64>, drag_x: Signal<f64>) -> impl Piece {
    let s1 = state;
    let pw1 = page_width;
    let dx1 = drag_x;
    let s2 = state;
    let pw2 = page_width;
    let dx2 = drag_x;
    row((
        button("<")
            .action(move || pager_go(s1, pw1, dx1, -1))
            .id("wk-prev")
            .frame(44.0, 36.0),
        spacer().grow(),
        label(move || strip_week_summary(&state.current_week.get()))
            .font(Font::Headline)
            .align(TextAlign::Center),
        spacer().grow(),
        button(">")
            .action(move || pager_go(s2, pw2, dx2, 1))
            .id("wk-next")
            .frame(44.0, 36.0),
    ))
    .spacing(8.0)
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
        items(get_lessons, |d: &DaySchedule| d.date),
        move |day_slot| {
            day_card_with(state, day_slot).any()
        },
    )
}

fn day_card_with(
    state: AppState,
    day_slot: ItemSlot<DaySchedule, u64>,
) -> impl Piece {
    let s_header = day_slot;
    let s_slots = day_slot;
    let s_empty = day_slot;
    let get_slots = move || s_slots.with(|d| d.slots.clone());

    column((
        label(move || {
            s_header.with(|d| utils::format_date_header(d.day_of_week, d.date))
        })
        .font(Font::Headline)
        .color(move || Color::hex(state.accent_color.get()))
        .padding(Insets { top: 16.0, leading: PAD, bottom: 6.0, trailing: PAD }).align(TextAlign::Leading),
        when(
            move || s_empty.with(|d| d.slots.is_empty()),
            move || {
                let s_dow = s_empty;
                label(move || {
                    let dow = s_dow.with(|d| d.day_of_week);
                    if dow >= 6 {
                        "Выходной день"
                    } else {
                        "Нет уроков"
                    }
                })
                .font(Font::Subheadline)
                .secondary()
                .padding(Insets { top: 2.0, leading: PAD, bottom: 8.0, trailing: PAD })
                .align(TextAlign::Leading)
            },
        ),
        each(
            items(get_slots, |s: &LessonSlot| s.number),
            move |slot| {
                widgets::lesson_row::render(slot).any()
            },
        ),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
}

// ── Summary view ───────────────────────────────────────────────────────



fn summary_view(state: AppState, page_width: Signal<f64>, drag_x: Signal<f64>) -> impl Piece {
    let s_prev = state;
    let s_cur = state;
    let s_next = state;
    let w = page_width.get();
    column((
        zstack((
            when(
                move || s_prev.current_quarter.get() > 0,
                move || summary_page(s_prev, -1, w).translation(move || drag_x.get() - w, 0.0),
            ),
            when(
                move || s_next.current_quarter.get() < 4,
                move || summary_page(s_next, 1, w).translation(move || drag_x.get() + w, 0.0),
            ),
            summary_page(s_cur, 0, w).translation(move || drag_x.get(), 0.0),
        ))
        .align(Alignment::TopLeading)
        .width(w)
        .on_drag(summary_drag(drag_x.clone(), page_width.clone(), state.clone())),
    ))
    .spacing(0.0)
    .width(w)
}

fn summary_page(state: AppState, offset: i32, w: f64) -> impl Piece {
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
                if q == 4 {
                    let mut sum = 0.0;
                    let mut count = 0;
                    for q_idx in 0..4 {
                        let m = get_quarter_marks_map(s, q_idx);
                        for vals in m.values() {
                            for &v in vals {
                                sum += v;
                                count += 1;
                            }
                        }
                    }
                    if count > 0 {
                        format!("Средний балл за год: {:.2}", sum / count as f64)
                    } else {
                        "Средний балл за год: —".to_string()
                    }
                } else {
                    let marks = get_quarter_marks_map(s, q);
                    let mut sum = 0.0;
                    let mut count = 0;
                    for vals in marks.values() {
                        for &v in vals {
                            sum += v;
                            count += 1;
                        }
                    }
                    if count > 0 {
                        format!("Средний балл: {:.2}", sum / count as f64)
                    } else {
                        "Средний балл: —".to_string()
                    }
                }
            })
            .font(Font::Subheadline)
            .secondary()
            .align(TextAlign::Center)
            .padding(Insets { top: 0.0, leading: PAD, bottom: 10.0, trailing: PAD })
        },
        
        when(
            move || (header_state.current_quarter.get() as i32 + offset).clamp(0, 4) != 4,
            move || quarter_summary_header(),
        ),
        when(
            move || (header_state.current_quarter.get() as i32 + offset).clamp(0, 4) == 4,
            move || year_summary_header(),
        ),
        divider().padding(Insets { top: 0.0, leading: PAD, bottom: 0.0, trailing: PAD }),
        when(
            move || (title_state.current_quarter.get() as i32 + offset).clamp(0, 4) != 4,
            move || quarter_summary_at(qs_state, offset),
        ),
        when(
            move || (title_state.current_quarter.get() as i32 + offset).clamp(0, 4) == 4,
            move || year_summary_at(ys_state),
        ),
    ))
    .spacing(0.0)
    .width(w)
}

fn quarter_summary_header() -> impl Piece {
    row((
        label("Предмет").font(Font::Caption).secondary().grow(),
        label("Ср. балл").font(Font::Caption).secondary().frame(72.0, 0.0).align(TextAlign::Center),
        label("Выставл.").font(Font::Caption).secondary().frame(68.0, 0.0).align(TextAlign::Center),
    ))
    .spacing(8.0)
    .padding(Insets { top: 6.0, leading: PAD, bottom: 6.0, trailing: PAD })
}

fn year_summary_header() -> impl Piece {
    row((
        label("Предмет").font(Font::Caption).secondary().grow(),
        label("I").font(Font::Caption).secondary().frame(28.0, 0.0).align(TextAlign::Center),
        label("II").font(Font::Caption).secondary().frame(28.0, 0.0).align(TextAlign::Center),
        label("III").font(Font::Caption).secondary().frame(28.0, 0.0).align(TextAlign::Center),
        label("IV").font(Font::Caption).secondary().frame(28.0, 0.0).align(TextAlign::Center),
        label("Ср.").font(Font::Caption).secondary().frame(42.0, 0.0).align(TextAlign::Center),
        label("Год").font(Font::Caption).secondary().frame(42.0, 0.0).align(TextAlign::Center),
    ))
    .spacing(4.0)
    .padding(Insets { top: 6.0, leading: 12.0, bottom: 6.0, trailing: 12.0 })
}

fn get_all_subject_names(state: AppState) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for t in state.subjects_teachers.get() {
        if !t.subject_title.is_empty() && !names.contains(&t.subject_title) {
            names.push(t.subject_title);
        }
    }
    for day in state.lessons.get() {
        for slot in &day.slots {
            if !slot.subject_title.is_empty() && !names.contains(&slot.subject_title) {
                names.push(slot.subject_title.clone());
            }
        }
    }
    state.week_cache.with(|cache| {
        for days in cache.values() {
            for day in days {
                for slot in &day.slots {
                    if !slot.subject_title.is_empty() && !names.contains(&slot.subject_title) {
                        names.push(slot.subject_title.clone());
                    }
                }
            }
        }
    });
    for q_map in state.quarter_all_marks.get() {
        for name in q_map.keys() {
            if !name.is_empty() && !names.contains(name) {
                names.push(name.clone());
            }
        }
    }
    for name in state.quarter_marks.get().keys() {
        if !name.is_empty() && !names.contains(name) {
            names.push(name.clone());
        }
    }
    names.sort();
    names.dedup();
    names
}

fn get_quarter_marks_map(state: AppState, q: usize) -> std::collections::HashMap<String, Vec<f64>> {
    let q_all = state.quarter_all_marks.get();
    if let Some(m) = q_all.get(q) {
        if !m.is_empty() {
            return m.clone();
        }
    }
    state.week_cache.with(|c| {
        let (m, _) = features::diary::extract_marks_for_quarter(c, q);
        m
    })
}

fn get_quarter_official_map(state: AppState, q: usize) -> std::collections::HashMap<String, Vec<OfficialMark>> {
    let q_off = state.quarter_official_marks.get();
    if let Some(m) = q_off.get(q) {
        if !m.is_empty() {
            return m.clone();
        }
    }
    state.week_cache.with(|c| {
        let (_, off) = features::diary::extract_marks_for_quarter(c, q);
        off
    })
}

fn quarter_summary_at(state: AppState, offset: i32) -> impl Piece {
    let s_subj = state;
    let s_rows = state;
    each(
        items(
            move || get_all_subject_names(s_subj),
            |s: &String| s.clone(),
        ),
        move |item| {
            let subj_name = item.get();
            let sj1 = subj_name.clone();
            let sj2 = subj_name.clone();
            let sj3 = subj_name.clone();
            let sj4 = subj_name.clone();
            let st1 = s_rows;
            let st2 = s_rows;
            let st3 = s_rows;
            let st4 = s_rows;
            column((
                row((
                    label(subj_name).font(Font::Body).grow(),
                    // 1. Выходящая: средний балл по оценкам (реактивно!)
                    label(move || {
                        let cur_q = (st1.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                        let marks = get_quarter_marks_map(st1, cur_q);
                        let avg = marks.get(&sj1).and_then(|v| {
                            if v.is_empty() { None } else { Some(v.iter().sum::<f64>() / v.len() as f64) }
                        });
                        match avg {
                            Some(v) => format!("{:.2}", v),
                            None => "—".into(),
                        }
                    })
                    .font(Font::Headline)
                    .color(move || {
                        let cur_q = (st2.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                        let marks = get_quarter_marks_map(st2, cur_q);
                        let avg = marks.get(&sj2).and_then(|v| {
                            if v.is_empty() { None } else { Some(v.iter().sum::<f64>() / v.len() as f64) }
                        });
                        match avg {
                            Some(v) => utils::avg_grade_color(v),
                            None => colors::SECONDARY,
                        }
                    })
                    .frame(72.0, 0.0)
                    .align(TextAlign::Center),

                    // 2. Выставленная: выставленная итоговая оценка (реактивно!)
                    label(move || {
                        let cur_q = (st3.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                        let off = get_quarter_official_map(st3, cur_q);
                        let marks = get_quarter_marks_map(st3, cur_q);
                        let final_val = off.get(&sj3).and_then(|v| v.last()).map(|m| m.value).or_else(|| {
                            marks.get(&sj3).and_then(|v| {
                                if v.is_empty() { None } else { Some((v.iter().sum::<f64>() / v.len() as f64).round()) }
                            })
                        });
                        match final_val {
                            Some(v) => format!("{:.0}", v),
                            None => "—".into(),
                        }
                    })
                    .font(Font::Headline)
                    .color(move || {
                        let cur_q = (st4.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                        let off = get_quarter_official_map(st4, cur_q);
                        let marks = get_quarter_marks_map(st4, cur_q);
                        let final_val = off.get(&sj4).and_then(|v| v.last()).map(|m| m.value).or_else(|| {
                            marks.get(&sj4).and_then(|v| {
                                if v.is_empty() { None } else { Some((v.iter().sum::<f64>() / v.len() as f64).round()) }
                            })
                        });
                        match final_val {
                            Some(v) => utils::avg_grade_color(v),
                            None => colors::SECONDARY,
                        }
                    })
                    .frame(68.0, 0.0)
                    .align(TextAlign::Center),
                ))
                .spacing(8.0)
                .padding(Insets { top: 8.0, leading: PAD, bottom: 8.0, trailing: PAD }),
                divider().padding(Insets { top: 0.0, leading: PAD, bottom: 0.0, trailing: PAD }),
            ))
            .spacing(0.0)
            .any()
        },
    )
}

fn year_summary_at(state: AppState) -> impl Piece {
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
                    label(subj_name).font(Font::Subheadline).grow(),
                    year_q_cell(cell_state, sj0, 0),
                    year_q_cell(cell_state, sj1, 1),
                    year_q_cell(cell_state, sj2, 2),
                    year_q_cell(cell_state, sj3, 3),
                    year_avg_cell(cell_state, sja),
                    year_final_cell(cell_state, sjf),
                ))
                .spacing(4.0)
                .padding(Insets { top: 8.0, leading: 12.0, bottom: 8.0, trailing: 12.0 }),
                divider().padding(Insets { top: 0.0, leading: 12.0, bottom: 0.0, trailing: 12.0 }),
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
    label(move || {
        let marks = get_quarter_marks_map(s1, q);
        if let Some(v) = marks.get(&*sj1) {
            if !v.is_empty() {
                return format!("{:.0}", (v.iter().sum::<f64>() / v.len() as f64).round());
            }
        }
        let yqd = s1.year_quarter_data.get();
        if let Some((_, m)) = yqd.get(q) {
            if let Some(v) = m.get(&*sj1) {
                if !v.is_empty() {
                    return format!("{:.0}", (v.iter().sum::<f64>() / v.len() as f64).round());
                }
            }
        }
        "—".into()
    })
    .font(Font::Subheadline)
    .color(move || {
        let marks = get_quarter_marks_map(s2, q);
        let avg = marks.get(&*sj2).and_then(|v| {
            if v.is_empty() { None } else { Some(v.iter().sum::<f64>() / v.len() as f64) }
        });
        match avg {
            Some(v) => utils::avg_grade_color(v),
            None => colors::SECONDARY,
        }
    })
    .align(TextAlign::Center)
    .frame(28.0, 0.0)
}

fn calc_year_avg(state: AppState, subject: &str) -> Option<f64> {
    let mut all_marks: Vec<f64> = Vec::new();
    for q in 0..4 {
        let marks = get_quarter_marks_map(state, q);
        if let Some(vals) = marks.get(subject) {
            all_marks.extend_from_slice(vals);
        }
    }
    if all_marks.is_empty() {
        None
    } else {
        Some(all_marks.iter().sum::<f64>() / all_marks.len() as f64)
    }
}

fn year_avg_cell(state: AppState, subject: String) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let sj1 = subject.clone();
    let sj2 = subject;
    label(move || {
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
            Some(v) => utils::avg_grade_color(v),
            None => colors::SECONDARY,
        }
    })
    .align(TextAlign::Center)
    .frame(42.0, 0.0)
}

fn year_final_cell(state: AppState, subject: String) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let sj1 = subject.clone();
    let sj2 = subject;
    label(move || {
        let off = s1.official_marks.get();
        if let Some(om) = off.get(&*sj1).and_then(|v| v.last()) {
            format!("{:.0}", om.value)
        } else {
            let avg = calc_year_avg(s1, &sj1);
            match avg {
                Some(v) => format!("{:.0}", v.round()),
                None => "—".into(),
            }
        }
    })
    .font(Font::Headline)
    .color(move || {
        let off = s2.official_marks.get();
        let val = if let Some(om) = off.get(&*sj2).and_then(|v| v.last()) {
            Some(om.value)
        } else {
            calc_year_avg(s2, &sj2).map(|v| v.round())
        };
        match val {
            Some(v) => utils::avg_grade_color(v),
            None => colors::SECONDARY,
        }
    })
    .align(TextAlign::Center)
    .frame(42.0, 0.0)
}

// ── Structs ────────────────────────────────────────────────────────────
