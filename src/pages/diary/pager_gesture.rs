use super::super::period::select_quarter;
use super::super::{SWIPE_AXIS_LOCK, SWIPE_EDGE_DAMP, SWIPE_GEN, SWIPE_PENDING, SWIPE_THRESHOLD};
use crate::app::AppState;
use crate::app::preferences::haptics;
use day::prelude::*;
use eschool_api::entities::DaySchedule;

use std::sync::atomic::Ordering;

pub(super) fn lessons_at(state: AppState, offset: i32) -> Vec<DaySchedule> {
    if offset == 0 {
        state.lessons.get()
    } else {
        let idx = state.current_week_index.get() + offset;
        state
            .week_cache
            .with(|cache| cache.get(&idx).cloned().unwrap_or_default())
    }
}

pub(super) fn pager_go(state: AppState, page_width: Signal<f64>, drag_x: Signal<f64>, dir: i32) {
    start_week_swipe(state, drag_x, page_width, dir, 180);
}

fn start_week_swipe(
    state: AppState,
    drag_x: Signal<f64>,
    page_width: Signal<f64>,
    dir: i32,
    dur_ms: u32,
) {
    let pending = SWIPE_PENDING.load(Ordering::Relaxed);
    let base = if pending >= 0 {
        pending
    } else {
        state.current_week_index.get()
    };
    let total = state.all_weeks.get().len() as i32;
    let new_idx = base + dir;
    if new_idx < 0 || new_idx >= total {
        // Out of range: never touch a committed flight — only spring a resting drag.
        if pending < 0 {
            with_animation(AnimSpec::ease_out(180), || {
                drag_x.set(0.0);
            });
        }
        return;
    }
    SWIPE_PENDING.store(new_idx, Ordering::Relaxed);
    haptics::bump();
    let my_gen = SWIPE_GEN.fetch_add(1, Ordering::Relaxed) + 1;
    crate::pages::diary::data::prefetch_week(state, new_idx);
    let w = page_width.get();
    let target = if dir > 0 { -w } else { w };
    with_animation(AnimSpec::ease_out(dur_ms), || {
        drag_x.set(target);
    });
    spawn_week_slide(drag_x.setter(), dur_ms, new_idx, my_gen);
}

fn spawn_week_slide(set_drag: day::reactive::Setter<f64>, dur_ms: u32, new_idx: i32, my_gen: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(dur_ms as u64));
        day::reactive::on_main(move || {
            if SWIPE_GEN.load(Ordering::Relaxed) != my_gen {
                // Superseded by a newer swipe: it owns drag/index now.
                return;
            }
            day::reactive::batch(|| {
                if let Some(state) = AppState::get_main() {
                    crate::pages::diary::data::load_week(state, new_idx);
                }
                set_drag.set(0.0);
            });
            SWIPE_PENDING.store(-1, Ordering::Relaxed);
        });
    });
}

pub(in crate::pages::diary) fn pager_drag(
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
                    horiz = Some(dx.abs() > dy.abs());
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
                let mut was_horiz = axis.get() == Some(true);
                let mut actual_dx = drag_x.get();
                // A fast fling can arrive as Began→Ended with no Changed large
                // enough to resolve the axis; honor it from the raw delta.
                if !was_horiz
                    && axis.get().is_none()
                    && dx.abs() >= SWIPE_THRESHOLD
                    && dx.abs() > dy.abs()
                {
                    was_horiz = true;
                    actual_dx = dx;
                    drag_x.set(dx);
                }
                axis.set(None);
                if was_horiz && actual_dx.abs() >= SWIPE_THRESHOLD {
                    let dir = if actual_dx < 0.0 { 1 } else { -1 };
                    start_week_swipe(state, drag_x, page_width, dir, 140);
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

pub(in crate::pages::diary) fn summary_drag(
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
                    horiz = Some(dx.abs() > dy.abs());
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
                        haptics::tick();
                        with_animation(AnimSpec::ease_out(220), move || {
                            drag_x.set(-w);
                        });
                        let set_drag = drag_x.setter();
                        let my_gen = SWIPE_GEN.fetch_add(1, Ordering::AcqRel) + 1;
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(220));
                            day::reactive::on_main(move || {
                                if SWIPE_GEN.load(Ordering::Acquire) != my_gen {
                                    return;
                                }
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
                        haptics::tick();
                        with_animation(AnimSpec::ease_out(220), move || {
                            drag_x.set(w);
                        });
                        let set_drag = drag_x.setter();
                        let my_gen = SWIPE_GEN.fetch_add(1, Ordering::AcqRel) + 1;
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(220));
                            day::reactive::on_main(move || {
                                if SWIPE_GEN.load(Ordering::Acquire) != my_gen {
                                    return;
                                }
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
