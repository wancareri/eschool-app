#[path = "pager_gesture.rs"]
mod gesture;
pub(in crate::pages::diary) use gesture::{pager_drag, summary_drag};

use self::gesture::{lessons_at, pager_go};
use super::PAD;
use super::period::selected_quarter_has_current_week;
use super::{SWIPE_GEN, SWIPE_PENDING};
use crate::app::AppState;
use crate::{components, app::format};
use day::prelude::*;
use eschool_api::entities::{DaySchedule, LessonSlot};
use std::sync::atomic::Ordering;

pub(super) fn week_view(state: AppState, period_revision: Signal<u64>) -> impl Piece {
    // Pager state lives inside the view so a size-class rebuild re-captures the
    // screen width — the captured one is the previous orientation's after a rotation.
    Scope::current().on_cleanup(|| {
        SWIPE_GEN.fetch_add(1, Ordering::AcqRel);
        SWIPE_PENDING.store(-1, Ordering::Release);
    });
    let page_width = Signal::new(crate::ios::viewport::get_screen_width());
    let drag_x = Signal::new(0.0);
    // Header choices and data refreshes can replace the week during a slide.
    day::reactive::watch(
        move || {
            (
                state.current_week_index.get(),
                state.current_quarter.get(),
                period_revision.get(),
            )
        },
        move |_, _| drag_x.set(0.0),
    );
    let s_prev = state;
    let s_cur = state;
    let s_next = state;
    let header_state = state;
    let header_width = page_width;
    let header_dx = drag_x;
    let w = page_width.get();
    column((
        when(
            move || selected_quarter_has_current_week(state),
            move || week_header(header_state, header_width, header_dx),
        ),
        when(
            move || selected_quarter_has_current_week(state),
            move || {
                crate::ios::drag::horizontal_pager(
                    zstack((
                        // Previous week page (at drag_x - w)
                        when(
                            move || s_prev.current_week_index.get() > 0,
                            move || {
                                week_page(s_prev, -1, w).translation(move || drag_x.get() - w, 0.0)
                            },
                        ),
                        // Next week page (at drag_x + w)
                        when(
                            move || {
                                s_next.current_week_index.get() + 1
                                    < s_next.all_weeks.get().len() as i32
                            },
                            move || {
                                week_page(s_next, 1, w).translation(move || drag_x.get() + w, 0.0)
                            },
                        ),
                        // Current week page (at drag_x)
                        week_page(s_cur, 0, w).translation(move || drag_x.get(), 0.0),
                    ))
                    .align(Alignment::TopLeading),
                    pager_drag(drag_x, page_width, state),
                )
                .width(w)
            },
        )
        .otherwise(|| {
            label("Нет данных за этот период")
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center)
                .padding(Insets {
                    top: 80.0,
                    leading: PAD,
                    bottom: 80.0,
                    trailing: PAD,
                })
        }),
    ))
    .spacing(0.0)
    .width(w)
    .grow(Dimension::Height)
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
            move || crate::pages::diary::week_summary::render(state, get_lessons),
        ),
        when(
            move || is_valid_week() && !is_loaded(),
            move || {
                column((components::spinner::render(state, 11.0),))
                    .align(HAlign::Center)
                    .padding(Insets {
                        top: 80.0,
                        leading: 0.0,
                        bottom: 80.0,
                        trailing: 0.0,
                    })
            },
        ),
        when(
            move || is_loaded() && get_lessons().is_empty(),
            || {
                label("Нет данных за этот период")
                    .font(Font::Body)
                    .secondary()
                    .align(TextAlign::Center)
                    .padding(Insets {
                        top: 80.0,
                        leading: PAD,
                        bottom: 80.0,
                        trailing: PAD,
                    })
            },
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
        button("‹")
            .enabled(move || state.current_week_index.get() > 0)
            .action(move || pager_go(s1, pw1, dx1, -1))
            .id(day::id!(wk_prev))
            .frame(44.0, 36.0),
        spacer().grow(Dimension::Both),
        label(move || strip_week_summary(&state.current_week.get()))
            .font(Font::Headline)
            .align(TextAlign::Center),
        spacer().grow(Dimension::Both),
        button("›")
            .enabled(move || {
                state.current_week_index.get() + 1 < state.all_weeks.get().len() as i32
            })
            .action(move || pager_go(s2, pw2, dx2, 1))
            .id(day::id!(wk_next))
            .frame(44.0, 36.0),
    ))
    .spacing(8.0)
    .padding(Insets {
        top: 0.0,
        leading: 16.0,
        bottom: 6.0,
        trailing: 16.0,
    })
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
        move |day_slot| day_card_with(state, day_slot).any(),
    )
}

fn day_card_with(state: AppState, day_slot: ItemSlot<DaySchedule, u64>) -> impl Piece {
    let s_header = day_slot;
    let s_slots = day_slot;
    let s_empty = day_slot;
    let s_date_for_row = day_slot;
    let get_slots = move || s_slots.with(|d| d.slots.clone());

    column((
        label(move || s_header.with(|d| format::format_date_header(d.day_of_week, d.date)))
            .font(Font::Headline)
            .color(move || Color::hex(state.accent_color.get()))
            .padding(Insets {
                top: 16.0,
                leading: PAD,
                bottom: 6.0,
                trailing: PAD,
            })
            .align(TextAlign::Leading),
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
                .padding(Insets {
                    top: 2.0,
                    leading: PAD,
                    bottom: 8.0,
                    trailing: PAD,
                })
                .align(TextAlign::Leading)
            },
        ),
        each(items(get_slots, |s: &LessonSlot| s.number), move |slot| {
            let date_str = s_date_for_row.with(|d| format::format_date_full(d.day_of_week, d.date));
            crate::pages::diary::lesson_row::render(slot, date_str).any()
        }),
    ))
    .spacing(0.0)
    .padding(Insets {
        top: 0.0,
        leading: 0.0,
        bottom: 8.0,
        trailing: 0.0,
    })
}

// ── Summary view ───────────────────────────────────────────────────────
