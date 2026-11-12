use super::{PAD, SWIPE_GEN, SWIPE_PENDING};
use crate::app::AppState;
use crate::app::preferences::haptics;
use day::prelude::*;
use std::sync::atomic::Ordering;

pub(super) fn child_switcher(state: AppState) -> impl Piece {
    when(
        move || state.is_parent.get(),
        move || {
            let children = state.parent_children.get();
            if children.len() > 1 {
                let pills: Vec<AnyPiece> = children
                    .into_iter()
                    .enumerate()
                    .map(|(this_idx, ch)| {
                        let name = ch.full_name.clone();
                        button(move || {
                            if state.active_child_index.get() == this_idx {
                                format!("● {}", name)
                            } else {
                                name.clone()
                            }
                        })
                        .action(move || {
                            haptics::tick();
                            crate::pages::diary::data::switch_child(state, this_idx);
                        })
                        .padding(Insets {
                            top: 4.0,
                            leading: 10.0,
                            bottom: 4.0,
                            trailing: 10.0,
                        })
                        .any()
                    })
                    .collect();

                row(pills)
                    .spacing(8.0)
                    .align(VAlign::Center)
                    .padding(Insets {
                        top: 2.0,
                        leading: PAD,
                        bottom: 6.0,
                        trailing: PAD,
                    })
                    .any()
            } else if let Some(ch) = children.first() {
                let name = ch.full_name.clone();
                label(move || format!("Обучающийся: {}", name))
                    .font(Font::Subheadline)
                    .secondary()
                    .align(TextAlign::Center)
                    .padding(Insets {
                        top: 0.0,
                        leading: PAD,
                        bottom: 4.0,
                        trailing: PAD,
                    })
                    .any()
            } else {
                spacer().height(0.0).any()
            }
        },
    )
}

// ── Quarter / Year tabs ────────────────────────────────────────────────

fn choose_quarter(state: AppState, show_summary: Signal<bool>, revision: Signal<u64>, q: usize) {
    haptics::tick();
    cancel_period_swipe(revision);
    if q == 4 {
        show_summary.set(true);
    }
    if show_summary.get() {
        select_quarter(state, q);
    } else {
        state.current_quarter.set(q);
        select_week_in_quarter(state, q);
    }
}

pub(super) fn quarter_tabs(
    state: AppState,
    show_summary: Signal<bool>,
    period_revision: Signal<u64>,
) -> impl Piece {
    row((
        quarter_btn(state, "I", 0, show_summary, period_revision),
        quarter_btn(state, "II", 1, show_summary, period_revision),
        quarter_btn(state, "III", 2, show_summary, period_revision),
        quarter_btn(state, "IV", 3, show_summary, period_revision),
        year_btn(state, show_summary, period_revision),
    ))
    .spacing(6.0)
    .padding(Insets {
        top: 0.0,
        leading: 12.0,
        bottom: 8.0,
        trailing: 12.0,
    })
}

fn quarter_btn(
    state: AppState,
    title: &'static str,
    q: usize,
    show_summary: Signal<bool>,
    revision: Signal<u64>,
) -> impl Piece {
    button(move || {
        if state.current_quarter.get() == q && !state.marks_loading.get() {
            format!("[{title}]")
        } else {
            title.to_string()
        }
    })
    .action(move || choose_quarter(state, show_summary, revision, q))
    .id(day::id!(quarter, title))
}

fn year_btn(state: AppState, show_summary: Signal<bool>, revision: Signal<u64>) -> impl Piece {
    button(move || {
        if state.current_quarter.get() == 4 && !state.marks_loading.get() {
            "[Год]"
        } else {
            "Год"
        }
    })
    .action(move || choose_quarter(state, show_summary, revision, 4))
    .id(day::id!(q_year))
}

fn cancel_period_swipe(revision: Signal<u64>) {
    SWIPE_GEN.fetch_add(1, Ordering::AcqRel);
    SWIPE_PENDING.store(-1, Ordering::Release);
    revision.update(|value| *value += 1);
}

pub(super) fn selected_quarter_has_current_week(state: AppState) -> bool {
    let index = state.current_week_index.get();
    index >= 0
        && (index as usize) < state.all_weeks.get().len()
        && state.week_quarters.with(|quarters|
            // Some schools provide weeks without enough period metadata to
            // infer quarters. Keep those real lessons and arrows accessible.
            quarters.get(index as usize).copied().flatten()
                .is_none_or(|quarter| quarter == state.current_quarter.get()))
}

fn select_week_in_quarter(state: AppState, q: usize) {
    let range = crate::pages::diary::data::quarter_week_indices(state, q);
    let current = state.current_week_index.get();
    if current < 0 || !range.contains(&(current as usize)) {
        if let Some(&first) = range.first().filter(|&&i| i < state.all_weeks.get().len()) {
            // A header choice supersedes an in-flight horizontal swipe.
            SWIPE_GEN.fetch_add(1, Ordering::AcqRel);
            SWIPE_PENDING.store(-1, Ordering::Release);
            crate::pages::diary::data::load_week(state, first as i32);
        }
    }
}

pub(super) fn select_quarter(state: AppState, q: usize) {
    if q == 4 {
        crate::pages::diary::data::load_year(state);
        return;
    }
    // Show existing per-quarter data immediately
    let q_all = state.quarter_all_marks.get();
    if let Some(marks) = q_all.get(q) {
        if !marks.is_empty() {
            state.quarter_marks.set(marks.clone());
        }
    }
    state.current_quarter.set(q);
    crate::pages::diary::data::load_quarter(state, q);
}

pub(super) fn sub_tabs(
    state: AppState,
    show_summary: Signal<bool>,
    period_revision: Signal<u64>,
) -> impl Piece {
    let s1 = state;
    let s2 = state;
    let s3 = state;
    row((
        button(move || {
            if !show_summary.get() {
                "● Недели"
            } else {
                "○ Недели"
            }
        })
        .action(move || {
            if show_summary.get() {
                haptics::tick();
                cancel_period_swipe(period_revision);
            }
            show_summary.set(false);
            let q = s1.current_quarter.get();
            if q < 4 {
                select_week_in_quarter(s2, q);
            } else {
                // Return from year totals to the current week's actual quarter.
                crate::pages::diary::data::load_week(s2, s2.current_week_index.get());
            }
        })
        .id(day::id!(sub_weeks)),
        button(move || {
            if show_summary.get() {
                "● Итоги"
            } else {
                "○ Итоги"
            }
        })
        .action(move || {
            if !show_summary.get() {
                haptics::tick();
                cancel_period_swipe(period_revision);
            }
            show_summary.set(true);
            let q = s3.current_quarter.get();
            select_quarter(s3, q);
        })
        .id(day::id!(sub_summary)),
    ))
    .spacing(12.0)
    .padding(Insets {
        top: 0.0,
        leading: 16.0,
        bottom: 8.0,
        trailing: 16.0,
    })
}

// ── Week view ──────────────────────────────────────────────────────────
