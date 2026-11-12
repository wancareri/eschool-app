use crate::app::AppState;
use crate::app::preferences::haptics;
use day::prelude::*;
use day::pullrefresh::pull_to_refresh;
#[cfg(test)]
use eschool_api::entities::WeekActivity;

use std::sync::atomic::{AtomicI32, AtomicU64};

const PAD: f64 = 20.0;
const SWIPE_THRESHOLD: f64 = 36.0;
const SWIPE_AXIS_LOCK: f64 = 8.0;
const SWIPE_EDGE_DAMP: f64 = 0.3;

// Every swipe bumps SWIPE_GEN and records its landing target in SWIPE_PENDING.
// The landing of a flight whose generation is stale returns without touching
// drag/index (its prefetch already warmed the cache), so a gesture during a
// flight can never be stomped by the old flight's load; basing the next swipe
// on PENDING (not the still-unchanged index) makes N rapid swipes land N pages.
static SWIPE_GEN: AtomicU64 = AtomicU64::new(0);
static SWIPE_PENDING: AtomicI32 = AtomicI32::new(-1);

/// Rebuild `build` whenever the window crosses a size-class breakpoint
/// (rotation on iOS: portrait width < 600pt, landscape ≥ 600pt).
///
/// The pagers pin the screen width they capture at build time, and day's piece
/// builds are one-shot — a plain build-time `size_class()` read is not tracked.
/// A keyed `each` whose key IS the class is the structural rebuild: its source
/// reads the class inside the reaction (docs/size-classes.md), and a changed
/// key remounts the one row, re-capturing the width.
pub fn rebuild_on_class_change<P: Piece + 'static>(build: impl Fn() -> P + 'static) -> impl Piece {
    each(
        items(
            move || vec![size_class().map(|c| (c.width as u8, c.height as u8))],
            |key| *key,
        ),
        move |_| build(),
    )
}

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let show_summary = Signal::new(false);
    let refreshing = Signal::new(false);
    let period_revision = Signal::new(0u64);

    diary_scroll(
        state,
        refreshing,
        column((
            child_switcher(state),
            sub_tabs(state, show_summary, period_revision),
            quarter_tabs(state, show_summary, period_revision),
            when(
                move || !show_summary.get(),
                move || rebuild_on_class_change(move || week_view(state, period_revision)),
            ),
            when(
                move || show_summary.get(),
                move || rebuild_on_class_change(move || summary_view(state, period_revision)),
            ),
        ))
        .spacing(0.0)
        .padding(Insets {
            top: 8.0,
            leading: 0.0,
            bottom: 0.0,
            trailing: 0.0,
        })
        .grow(Dimension::Width),
    )
}

fn diary_scroll(state: AppState, refreshing: Signal<bool>, content: impl Piece) -> impl Piece {
    pull_to_refresh(
        refreshing,
        scroll(
            column((content, spacer().height(16.0)))
                .spacing(0.0)
                .grow(Dimension::Width),
        ),
    )
    .on_refresh(move || {
        haptics::tick();
        if state.is_authenticated.get() {
            crate::pages::diary::data::load_all(state);
        }
    })
    .grow(Dimension::Both)
}

#[path = "pager.rs"]
mod pager;
#[path = "period.rs"]
mod period;
#[path = "summary.rs"]
mod summary;

use pager::week_view;
#[cfg(test)]
use pager::{pager_drag, summary_drag};
use period::{child_switcher, quarter_tabs, sub_tabs};
use summary::summary_view;

// ── Structs ────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "mock"))]
mod header_tests {
    use super::*;

    #[test]
    fn diary_has_one_page_level_scroll_before_any_branch() {
        day_core::uninstall_tree();
        let (mock, probe) = day_mock::MockToolkit::new();
        day_core::launch_with(mock, day_spec::WindowOptions::default(), || {
            with_environment(AppState::create(), render).any()
        });
        let state = probe.state.borrow();
        let scrolls: Vec<_> = state
            .widgets
            .values()
            .filter(|w| w.kind == day_spec::kinds::SCROLL)
            .collect();
        assert_eq!(
            scrolls.len(),
            1,
            "pull-to-refresh must use one page-level scroll"
        );
        let mut current = state
            .widgets
            .iter()
            .find(|(id, _)| !state.widgets.values().any(|w| w.children.contains(id)))
            .map(|(_, w)| w)
            .expect("page root");
        for _ in 0..6 {
            if current.kind == day_spec::kinds::SCROLL {
                break;
            }
            assert_eq!(
                current.children.len(),
                1,
                "UIKit must reach the scroll before any branching content"
            );
            current = &state.widgets[&current.children[0]];
        }
        assert_eq!(current.kind, day_spec::kinds::SCROLL);
        drop(state);
        day_core::uninstall_tree();
    }

    #[test]
    fn quarter_header_moves_the_visible_week_into_selected_quarter() {
        day_core::uninstall_tree();
        let (mock, probe) = day_mock::MockToolkit::new();
        let saved = std::rc::Rc::new(std::cell::Cell::new(None));
        let mounted = saved.clone();
        day_core::launch_with(mock, day_spec::WindowOptions::default(), move || {
            let state = AppState::create();
            state.all_weeks.set(
                (0..3)
                    .map(|i| WeekActivity {
                        uuid: format!("week-{i}"),
                        summary: format!("week {i}"),
                        time_activity_id: String::new(),
                        start_ts: 0,
                        end_ts: 0,
                    })
                    .collect(),
            );
            state.week_quarters.set(vec![Some(0), Some(1), Some(1)]);
            let show_summary = Signal::new(false);
            let revision = Signal::new(0);
            mounted.set(Some((state, show_summary, revision)));
            column((
                quarter_tabs(state, show_summary, revision),
                week_view(state, revision),
            ))
            .any()
        });
        let (state, show_summary, revision) = saved.get().unwrap();
        let click = |title: &str| {
            let target = probe
                .state
                .borrow()
                .widgets
                .values()
                .find(|w| w.kind == day_spec::kinds::BUTTON && w.text == title)
                .unwrap_or_else(|| panic!("missing quarter button {title}"))
                .node;
            probe.emit(day_spec::NodeId(target), day_spec::Event::Pressed);
        };
        click("II");
        day::reactive::flush_sync();
        assert_eq!(state.current_quarter.get(), 1);
        assert_eq!(
            state.current_week_index.get(),
            1,
            "the header must not show the old quarter's week after selection"
        );
        assert_eq!(state.current_week.get(), "week 1");
        click("III");
        day::reactive::flush_sync();
        assert_eq!(state.current_quarter.get(), 2);
        assert!(
            !probe
                .state
                .borrow()
                .widgets
                .values()
                .any(|w| w.text == "week 1"),
            "an empty quarter must not display the old week's header"
        );
        assert!(
            probe
                .state
                .borrow()
                .widgets
                .values()
                .any(|w| w.text == "Нет данных за этот период")
        );
        // Missing calendar metadata must not make otherwise loaded weeks inaccessible.
        state.week_quarters.set(vec![None; 3]);
        day::reactive::flush_sync();
        assert!(
            probe
                .state
                .borrow()
                .widgets
                .values()
                .any(|w| w.text == "week 1")
        );
        click("Год");
        day::reactive::flush_sync();
        assert!(show_summary.get(), "year selection must open totals");
        assert_eq!(state.current_quarter.get(), 4);
        assert_eq!(revision.get(), 3);
        day_core::uninstall_tree();
    }
}

#[cfg(test)]
mod gesture_tests {
    use super::*;

    #[test]
    fn vertical_and_diagonal_scrolls_never_turn_into_page_drags() {
        for summary in [false, true] {
            let state = AppState::create();
            state.current_week_index.set(1);
            state.current_quarter.set(1);
            let x = Signal::new(0.0);
            let width = Signal::new(390.0);
            let handler: Box<dyn Fn(Drag)> = if summary {
                Box::new(summary_drag(x, width, state))
            } else {
                Box::new(pager_drag(x, width, state))
            };
            for (phase, dx, dy) in [
                (DragPhase::Began, 0.0, 0.0),
                (DragPhase::Changed, 15.0, 25.0),
                (DragPhase::Changed, 35.0, 28.0),
                (DragPhase::Ended, 35.0, 28.0),
            ] {
                handler(Drag {
                    phase,
                    location: Point::ZERO,
                    translation: Point::new(dx, dy),
                });
                assert_eq!(
                    x.get(),
                    0.0,
                    "vertical scroll must keep its axis until release"
                );
            }
            handler(Drag {
                phase: DragPhase::Began,
                location: Point::ZERO,
                translation: Point::ZERO,
            });
            handler(Drag {
                phase: DragPhase::Changed,
                location: Point::ZERO,
                translation: Point::new(24.0, 5.0),
            });
            assert_eq!(
                x.get(),
                24.0,
                "horizontal movement must still follow the finger"
            );
        }
    }
}
