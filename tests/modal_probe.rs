#![cfg(feature = "mock")]

use day::prelude::*;
use day_mock::{MockProbe, MockToolkit};
use day_spec::{Event, NodeId, WindowOptions};

fn boot() -> (MockProbe, Signal<Option<String>>) {
    day_core::uninstall_tree();
    dayapp::ios::keyboard::set_keyboard_height(0.0);
    let open = Signal::global(None::<String>);
    let (mock, probe) = MockToolkit::new();
    day_core::launch_with(mock, WindowOptions::default(), move || {
        dayapp::components::modal::modal_window(open, 100.0, |_| label("Sheet body")).any()
    });
    open.set(Some("test".into()));
    day::reactive::flush_sync();
    (probe, open)
}

// A native recognizer must be on a card ancestor, excluding the dim and the
// fullscreen overlay. Injecting events alone would hide that regression.
fn card_drag(probe: &MockProbe) -> NodeId {
    let state = probe.state.borrow();
    let handle = state
        .log
        .iter()
        .rev()
        .find_map(|line| {
            line.strip_prefix("enable_gesture #")
                .and_then(|s| s.strip_suffix(" Drag"))
                .and_then(|s| s.parse::<u64>().ok())
        })
        .expect("modal must install a native card drag recognizer");
    fn contains_body(state: &day_mock::MockState, handle: u64) -> bool {
        let w = &state.widgets[&handle];
        w.text == "Sheet body" || w.children.iter().any(|h| contains_body(state, *h))
    }
    assert!(
        contains_body(&state, handle),
        "drag must cover the card body"
    );
    fn contains_dim(state: &day_mock::MockState, handle: u64) -> bool {
        let w = &state.widgets[&handle];
        (w.kind == day_spec::kinds::BUTTON && w.text.is_empty())
            || w.children.iter().any(|h| contains_dim(state, *h))
    }
    assert!(
        !contains_dim(&state, handle),
        "drag must attach to the card, not the fullscreen overlay"
    );
    assert_ne!(state.widgets[&handle].kind, day_spec::kinds::BUTTON);
    NodeId(state.widgets[&handle].node)
}

fn drag(probe: &MockProbe, node: NodeId, phase: DragPhase, y: f64) {
    probe.emit(
        node,
        Event::Drag {
            phase,
            location: Point::new(40.0, y),
            translation: Point::new(0.0, y),
        },
    );
    day::reactive::flush_sync();
}

fn card_translation(probe: &MockProbe, node: NodeId) -> f64 {
    let state = probe.state.borrow();
    let mut handle = *state
        .widgets
        .iter()
        .find(|(_, w)| w.node == node.0)
        .unwrap()
        .0;
    loop {
        if let Some(transform) = state.widgets[&handle].transform {
            return transform.ty;
        }
        handle = *state
            .widgets
            .iter()
            .find(|(_, w)| w.children.contains(&handle))
            .expect("card must have a translated ancestor")
            .0;
    }
}

#[test]
fn card_swipe_closes_only_past_threshold_and_can_reopen() {
    let (probe, open) = boot();
    let node = card_drag(&probe);
    drag(&probe, node, DragPhase::Began, 0.0);
    drag(&probe, node, DragPhase::Changed, 60.0);
    assert_eq!(
        card_translation(&probe, node),
        60.0,
        "card must follow the finger"
    );
    drag(&probe, node, DragPhase::Ended, 60.0);
    assert!(open.get().is_some());
    assert_eq!(
        card_translation(&probe, node),
        0.0,
        "short pull returns to rest"
    );
    drag(&probe, node, DragPhase::Began, 0.0);
    drag(&probe, node, DragPhase::Changed, -140.0);
    assert!((card_translation(&probe, node) + 46.66666667).abs() < 0.001);
    drag(&probe, node, DragPhase::Ended, -140.0);
    assert!(open.get().is_some(), "upward pull must not dismiss");
    drag(&probe, node, DragPhase::Began, 0.0);
    drag(&probe, node, DragPhase::Changed, 90.0);
    drag(&probe, node, DragPhase::Ended, 90.0);
    assert!(open.get().is_none());

    let cover = probe.find_by_kind(day_spec::kinds::COVER)[0].1.node;
    probe.emit(NodeId(cover), Event::CoverHidden);
    open.set(Some("again".into()));
    day::reactive::flush_sync();
    let node = card_drag(&probe);
    drag(&probe, node, DragPhase::Began, 0.0);
    drag(&probe, node, DragPhase::Changed, 90.0);
    drag(&probe, node, DragPhase::Ended, 90.0);
    assert!(open.get().is_none());
}

#[test]
fn keyboard_takes_priority_over_sheet_dismissal() {
    let (probe, open) = boot();
    let node = card_drag(&probe);
    dayapp::ios::keyboard::set_keyboard_height(300.0);
    drag(&probe, node, DragPhase::Began, 0.0);
    drag(&probe, node, DragPhase::Changed, 100.0);
    drag(&probe, node, DragPhase::Ended, 100.0);
    assert!(open.get().is_some(), "first swipe must keep the sheet open");
    dayapp::ios::keyboard::set_keyboard_height(0.0);
    drag(&probe, node, DragPhase::Began, 0.0);
    drag(&probe, node, DragPhase::Changed, 100.0);
    drag(&probe, node, DragPhase::Ended, 100.0);
    assert!(open.get().is_none());
}

#[test]
fn dim_tap_closes_sheet() {
    let (probe, open) = boot();
    let (handle, _) = probe
        .find_by_kind(day_spec::kinds::BUTTON)
        .into_iter()
        .find(|(_, w)| w.text.is_empty())
        .expect("dim button");
    let node = probe.widget(handle).node;
    dayapp::ios::keyboard::set_keyboard_height(300.0);
    probe.emit(NodeId(node), Event::Pressed);
    day::reactive::flush_sync();
    assert!(
        open.get().is_some(),
        "dim tap with keyboard must keep the sheet open"
    );
    dayapp::ios::keyboard::set_keyboard_height(0.0);
    probe.emit(NodeId(node), Event::Pressed);
    day::reactive::flush_sync();
    assert!(open.get().is_none());
}
