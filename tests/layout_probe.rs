//! Layout regression for the Итоги card (mock toolkit, 8pt/char × 16pt metrics).
//!
//! The UIKit backend never applies `TextAlign` — a label that fills its parent
//! renders its text leading. Centering therefore has to come from the layout:
//! hug labels inside a column that fills the space under its padding (the
//! column's default `CrossAlign::Center` then has slack to center within).

#![cfg(feature = "mock")]

use day::prelude::*;
use day_mock::{MockProbe, MockToolkit};
use day_spec::WindowOptions;
use std::collections::HashMap;

const WINDOW_W: f64 = 400.0;

fn boot(root: impl FnOnce() -> AnyPiece + 'static) -> MockProbe {
    day_core::uninstall_tree();
    let (mock, probe) = MockToolkit::new();
    day_core::launch_with(
        mock,
        WindowOptions {
            title: "probe".into(),
            size: Size::new(WINDOW_W, 600.0),
            ..Default::default()
        },
        root,
    );
    probe
}

/// Parent links from children lists; roots = widgets nobody lists as a child.
/// The mock pump never registers windows, so `probe.windows()` stays empty.
fn parents(probe: &MockProbe) -> HashMap<u64, u64> {
    let state = probe.state.borrow();
    let mut parent: HashMap<u64, u64> = HashMap::new();
    for (h, w) in &state.widgets {
        for c in &w.children {
            parent.insert(*c, *h);
        }
    }
    parent
}

fn handles_with_text(probe: &MockProbe, text: &str) -> Vec<u64> {
    let state = probe.state.borrow();
    let mut hs: Vec<u64> = state
        .widgets
        .iter()
        .filter(|(_, w)| w.text == text)
        .map(|(h, _)| *h)
        .collect();
    hs.sort();
    hs
}

/// The label must sit centered inside its parent column, with slack to prove
/// the centering is the layout's and not a coincidence of hug widths.
fn assert_centered(probe: &MockProbe, parent: &HashMap<u64, u64>, h: u64) {
    let state = probe.state.borrow();
    let w = state.widgets.get(&h).expect("label");
    let p = state.widgets.get(&parent[&h]).expect("parent");
    // Child frames are parent-relative: centered means label center = parent center.
    let lc = w.frame.origin.x + w.frame.size.width / 2.0;
    let pc = p.frame.size.width / 2.0;
    let (lw, cw) = (w.frame.size.width, p.frame.size.width);
    assert!(
        (lc - pc).abs() <= 0.6,
        "label {:?} center {lc} not centered in parent w={cw}",
        w.text
    );
    assert!(
        cw > lw,
        "label {:?} w={lw} fills its parent w={cw} — nothing to center within",
        w.text
    );
}

/// Mark chips: hug labels centered in a column that fills under the padding.
#[test]
fn mark_chips_center_text() {
    let probe = boot(|| {
        column((
            row((
                chip("12.09", "5"),
                chip("13.09", "4"),
                chip("14.09", "5"),
            ))
            .spacing(6.0)
            .fit(RowFit::WrapColumns { run_spacing: 6.0 })
            .grow_w(),
        ))
        .spacing(10.0)
        .padding(14.0)
        .any()
    });
    let parent = parents(&probe);
    for text in ["12.09", "13.09", "14.09", "5", "4"] {
        for h in handles_with_text(&probe, text) {
            assert_centered(&probe, &parent, h);
        }
    }
    // The wrap grid is uniform: every chip's rounded shell has the same width.
    // Radius and background sit on separate nodes (ops compose outward), so the
    // shell is found by its corner_radius alone.
    let state = probe.state.borrow();
    let mut shells: Vec<f64> = state
        .widgets
        .values()
        .filter(|w| w.corner_radius > 0.0)
        .map(|w| w.frame.size.width)
        .collect();
    shells.sort_by(|a, b| a.partial_cmp(b).unwrap());
    shells.dedup_by(|a, b| (*a - *b).abs() <= 0.6);
    assert_eq!(shells.len(), 1, "chips are not one uniform width: {shells:?}");
}

/// The «Цель» bar spans the card's full width with its label centered.
#[test]
fn goal_bar_full_width_centered() {
    let probe = boot(|| {
        column((goal_bar(),))
            .spacing(10.0)
            .padding(14.0)
            .any()
    });
    let parent = parents(&probe);
    for h in handles_with_text(&probe, "Цель") {
        assert_centered(&probe, &parent, h);
    }
    let state = probe.state.borrow();
    let bar = state
        .widgets
        .values()
        .find(|w| w.corner_radius > 0.0)
        .expect("goal bar shell");
    assert!(
        (bar.frame.size.width - (WINDOW_W - 28.0)).abs() <= 0.6,
        "goal bar is {}pt wide, want {}pt",
        bar.frame.size.width,
        WINDOW_W - 28.0
    );
}

/// The goal stepper's value digits center inside their fixed-width box.
#[test]
fn goal_value_box_centered() {
    let probe = boot(|| {
        column((
            row((
                label("Цель по итогу:").grow_w(),
                column((label("42").font(Font::Title3),)).width(44.0),
            ))
            .spacing(8.0)
            .align(VAlign::Center)
            .grow_w(),
        ))
        .spacing(10.0)
        .padding(14.0)
        .any()
    });
    let parent = parents(&probe);
    for h in handles_with_text(&probe, "42") {
        let state = probe.state.borrow();
        let p = state.widgets.get(&parent[&h]).expect("value box");
        assert!(
            (p.frame.size.width - 44.0).abs() <= 0.6,
            "value box is {}pt, want 44pt",
            p.frame.size.width
        );
        drop(state);
        assert_centered(&probe, &parent, h);
    }
}

/// A mark chip: hug labels, column fills under the padding, shell on top —
/// the chain grade_peek uses for every cell of the strip.
fn chip(date: &'static str, mark: &'static str) -> AnyPiece {
    column((
        label(date).font(Font::Caption2).color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
        label(mark).font(Font::Subheadline),
    ))
    .spacing(1.0)
    .align(HAlign::Center)
    .grow_w()
    .padding(Insets {
        top: 4.0,
        leading: 6.0,
        bottom: 4.0,
        trailing: 6.0,
    })
    .background(Color::rgba(0.0, 0.0, 0.0, 0.07))
    .corner_radius(8.0)
    .grow_w()
    .any()
}

/// The «Цель» row: hug label, column fills under the padding, tinted shell.
fn goal_bar() -> AnyPiece {
    column((
        label("Цель")
            .font(Font::Subheadline)
            .color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
    ))
    .grow_w()
    .padding(Insets {
        top: 8.0,
        leading: 12.0,
        bottom: 8.0,
        trailing: 12.0,
    })
    .background(Color::rgba(0.0, 0.0, 0.0, 0.06))
    .corner_radius(14.0)
    .grow_w()
    .any()
}
