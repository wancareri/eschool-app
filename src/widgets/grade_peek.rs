//! Итоги peek — a tap or long-press on a subject row slides a side panel out
//! from the screen's right edge (the page dims and stays visible to its left):
//! the quarter's marks wrapped into lines (date over mark), the average at the
//! end of the strip, and a «+» that opens the two planning modes — predicted
//! marks tapped straight into the strip (each one highlighted among the real
//! marks), and a target-grade plan that answers with ONE whole mark to keep
//! reaching, rounded up.
//!
//! Signals live here (thread-local), not in `AppState`: the peek is one screen
//! concern and the row handler reaches it through [`open`].

use crate::app::AppState;
use crate::shared::{colors, utils};
use day::prelude::*;

/// Bounds for pickable/target marks. The school reports on the ten-point scale
/// (see `utils::avg_grade_color`'s bands); allowing the full 1..10 range keeps
/// the peek correct even if a subject is graded differently.
const PEEK_MIN: f64 = 1.0;
const PEEK_MAX: f64 = 10.0;
/// The floor a "suffices" scenario counts on — двойка, not единичка.
const PEEK_FLOOR: f64 = 2.0;

#[derive(Clone, Copy)]
struct Peek {
    subject: Signal<Option<String>>,
    quarter: Signal<usize>,
    /// Predicted marks appended after the real ones (tap a chip to remove).
    preds: Signal<Vec<f64>>,
    /// 0 = closed, 1 = predict controls, 2 = goal controls.
    panel: Signal<u8>,
    /// Target quarter average for the goal mode — always a whole mark.
    target: Signal<f64>,
    /// How many marks are still to come (future lessons aren't in the cache).
    k: Signal<usize>,
    /// Pop animation: 0 while mounting, animated to 1; back to 0 on the way out.
    shown: Signal<f64>,
    /// A close is in flight — its unmount is pending; opening again cancels it.
    closing: Signal<bool>,
    /// Generation counter: every open/close bumps it, so a stale timer thread
    /// sees the mismatch and stays out of the way.
    epoch: Signal<u64>,
}

impl Peek {
    fn new() -> Self {
        Self {
            subject: Signal::new(None),
            quarter: Signal::new(0),
            preds: Signal::new(Vec::new()),
            panel: Signal::new(0),
            target: Signal::new(5.0),
            k: Signal::new(5),
            shown: Signal::new(0.0),
            closing: Signal::new(false),
            epoch: Signal::new(0),
        }
    }

    fn open(&self, subject: String, quarter: usize) {
        // Idempotent: a long-press release also fires the row's tap, and the
        // second call must not restart the pop. Re-opening while a close is
        // still fading cancels that close and pops again instead.
        if self.subject.get().is_some() && !self.closing.get() {
            return;
        }
        // Whether the panel's subtree is already on screen — a re-open during
        // the close slide keeps it (the `when` condition never went false), so
        // the fresh-mount build closure won't run again and the pop has to be
        // summoned from here.
        let was_mounted = self.subject.get().is_some();
        self.quarter.set(quarter);
        self.preds.set(Vec::new());
        self.panel.set(0);
        self.target.set(5.0);
        self.k.set(5);
        self.shown.set(0.0);
        self.closing.set(false);
        self.epoch.set(self.epoch.get() + 1);
        let g = self.epoch.get();
        // Whom the overlay's build closure should pop for: a thread-local, not
        // a Signal — reading `epoch` inside that closure reactively would make
        // this very bump re-run the build (and, worse, spawn a pop in the
        // middle of a close, whose epoch still matches).
        PENDING.with(|c| c.set(g));
        self.subject.set(Some(subject));
        if was_mounted {
            spawn_pop(g);
        }
    }

    fn close(&self) {
        if self.subject.get().is_none() || self.closing.get() {
            return;
        }
        // Slide the panel back off the right edge first; the subtree unmounts
        // a tick after the animation lands, and only if nothing re-opened the
        // peek meanwhile.
        self.closing.set(true);
        self.epoch.set(self.epoch.get() + 1);
        let g = self.epoch.get();
        let set = self.shown.setter();
        with_animation(AnimSpec::ease_out(200), move || set.set(0.0));
        let subj = self.subject.setter();
        let cls = self.closing.setter();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(230));
            day::reactive::on_main(move || {
                if peek().epoch.get() == g {
                    subj.set(None);
                    cls.set(false);
                }
            });
        });
    }
}

thread_local! {
    static PEEK: std::cell::Cell<Option<Peek>> = const { std::cell::Cell::new(None) };
    /// Which generation the overlay's build closure owes a pop to — written by
    /// `open` (same thread as the build), consumed on the first build after it.
    static PENDING: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn peek() -> Peek {
    PEEK.with(|c| match c.get() {
        Some(p) => p,
        None => {
            let p = Peek::new();
            c.set(Some(p));
            p
        }
    })
}

/// One entrance slide: a tick AFTER the panel's subtree is built, animate
/// `shown` to 1. Called from the overlay's build closure (fresh mount — the
/// subtree has to exist before the animated write lands on it) or from `open`
/// (re-open during a close slide — no rebuild happens there). The epoch check
/// runs inside `on_main` on the main thread (Signals are not Send): a newer
/// open/close invalidates `g` and this timer stays out of the way.
fn spawn_pop(g: u64) {
    let set = peek().shown.setter();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(48));
        day::reactive::on_main(move || {
            if peek().epoch.get() == g {
                with_animation(AnimSpec::ease_out(260), move || set.set(1.0));
            }
        });
    });
}

/// Summon the peek — called from the Итоги row's tap / long-press.
pub fn open(subject: String, quarter: usize) {
    peek().open(subject, quarter);
}

fn subject_marks(state: AppState, subject: &str, quarter: usize) -> Vec<f64> {
    state.week_cache.with(|c| {
        let dated = crate::features::diary::extract_marks_dated(c, quarter, subject);
        dated.iter().flat_map(|(_, _, p)| p.iter().copied()).collect()
    })
}

fn avg_of(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        None
    } else {
        Some(v.iter().sum::<f64>() / v.len() as f64)
    }
}

/// The accent at a given alpha — one place for the hex → rgba dance.
fn accent_tint(hex: u32, alpha: f64) -> Color {
    Color::rgba(
        ((hex >> 16) & 0xff) as f64 / 255.0,
        ((hex >> 8) & 0xff) as f64 / 255.0,
        (hex & 0xff) as f64 / 255.0,
        alpha,
    )
}

/// The goal-mode verdict, split for the layout: a big whole number and the
/// line under it.
struct GoalReport {
    big: String,
    hint: String,
    color: Color,
}

/// The verdict math: how the k remaining marks have to be graded. The school's
/// FINAL quarter mark is the raw average rounded to a whole number by the
/// teacher, so the plan works off the threshold `target - 0.5` — any average at
/// or above it rounds up onto the target. What the student has to HOLD is still
/// reported as a whole mark (rounded up, because any lower could fall short).
fn goal_report(state: AppState, pk: Peek) -> GoalReport {
    let q = pk.quarter.get();
    let subj = pk.subject.get().unwrap_or_default();
    let marks = subject_marks(state, &subj, q);
    let n = marks.len() as f64;
    let sum: f64 = marks.iter().sum();
    let target = pk.target.get();
    // Rounding the final average to a whole mark: avg ≥ thr ⟺ round(avg) ≥ target.
    let thr = target - 0.5;
    let k = pk.k.get() as f64;

    if marks.is_empty() {
        return GoalReport {
            big: "—".into(),
            hint: "Нет оценок за четверть — считать не от чего.".into(),
            color: colors::SECONDARY,
        };
    }
    if k == 0.0 {
        let cur = sum / n;
        return if cur >= thr - 1e-9 {
            GoalReport {
                big: format!("{cur:.1}"),
                hint: format!(
                    "Оценок не осталось — итог {cur:.1} округляется до {:.0}, цель {target:.0} держится.",
                    cur.round()
                ),
                color: colors::SUCCESS,
            }
        } else {
            GoalReport {
                big: format!("{cur:.1}"),
                hint: format!(
                    "Оценок не осталось — итог {cur:.1} округлится до {:.0}, цели {target:.0} не хватит.",
                    cur.round()
                ),
                color: colors::ERROR,
            }
        };
    }

    // The average each of the k remaining marks has to deliver.
    let per = (thr * (n + k) - sum) / k;
    if per <= PEEK_FLOOR + 1e-9 {
        let with_floor = (sum + PEEK_FLOOR * k) / (n + k);
        return GoalReport {
            big: "2".into(),
            hint: format!(
                "Достаточно любых двоек — итог {with_floor:.2} округляется до {:.0} ≥ цели {target:.0}.",
                with_floor.round()
            ),
            color: colors::SUCCESS,
        };
    }
    if per > PEEK_MAX + 1e-9 {
        let max = (sum + PEEK_MAX * k) / (n + k);
        return GoalReport {
            big: "10+".into(),
            hint: format!(
                "Невозможно: даже все десятки дадут итог {max:.2} → {:.0} — цели {target:.0} не хватит.",
                max.round()
            ),
            color: colors::ERROR,
        };
    }

    let need = per.ceil().clamp(PEEK_MIN, PEEK_MAX);
    let hint = format!(
        "На каждую из {} оценок — не ниже {need} (среднее {per:.2}; итог округляется учителем до целого).",
        k as u64
    );
    let color = if need <= 6.0 {
        colors::SUCCESS
    } else if need <= 8.0 {
        colors::SECONDARY
    } else {
        colors::ERROR
    };
    GoalReport {
        big: format!("{need:.0}"),
        hint,
        color,
    }
}

/// The fullscreen overlay: dim (tap to close) + the side panel.
///
/// The panel is an opaque full-height slab that slides out of the RIGHT edge —
/// the page stays visible, dimmed, in a strip to its left, instead of being
/// covered by a centered card. Nothing inside scrolls (the strip wraps
/// instead), so there is no gesture that could slide content and expose the
/// dim behind the panel. The entrance's animated write is spawned from the
/// `when` build closure below — only there is the subtree guaranteed to exist
/// when the timer lands.
pub fn overlay(state: AppState) -> impl Piece {
    let pk = peek();
    when(
        move || pk.subject.get().is_some(),
        move || {
            // The subtree just (re)built for this open — start its slide. The
            // generation comes from a thread-local (see `PENDING`), not a
            // tracked read: the build must not re-run when `epoch` bumps.
            let g = PENDING.with(|c| {
                let v = c.get();
                c.set(0);
                v
            });
            if g != 0 {
                spawn_pop(g);
            }
            let w = crate::pages::diary::get_screen_width();
            // Keep a strip of the dimmed page visible to the panel's left.
            let panel_w = (w - 52.0).max(240.0);
            zstack((
                button("")
                    .action(move || peek().close())
                    .background(move || Color::rgba(0.0, 0.0, 0.0, 0.34 * pk.shown.get()))
                    .grow(),
                build_card(state, pk, panel_w)
                    // Off-screen right while hidden, flush to the edge when shown.
                    .translation(move || panel_w * (1.0 - pk.shown.get()), 0.0),
            ))
            .align(Alignment::TopTrailing)
            .grow()
        },
    )
    .grow()
}

/// A mode chip for the Предикт/Цель switch: accent-tinted when its mode is
/// the live one.
fn seg_chip(state: AppState, pk: Peek, mode: u8, text: &'static str) -> AnyPiece {
    column((
        label(text).font(Font::Subheadline).color(move || {
            if pk.panel.get() == mode {
                Color::hex(state.accent_color.get())
            } else {
                colors::SECONDARY
            }
        }),
    ))
    .padding(Insets {
        top: 5.0,
        leading: 12.0,
        bottom: 5.0,
        trailing: 12.0,
    })
    .corner_radius(8.0)
    .background(move || {
        if pk.panel.get() == mode {
            accent_tint(state.accent_color.get(), 0.14)
        } else {
            Color::rgba(0.0, 0.0, 0.0, 0.06)
        }
    })
    .on_tap(move || {
        if mode == 2 {
            // Seed the goal from the current average, as a whole mark.
            let q = pk.quarter.get();
            let subj = pk.subject.get().unwrap_or_default();
            if let Some(a) = avg_of(&subject_marks(state, &subj, q)) {
                pk.target.set(a.round().clamp(PEEK_MIN, PEEK_MAX));
            }
        }
        pk.panel.set(mode);
    })
    .any()
}

/// One digit of the predict keypad: accent-tinted chip, tap appends that mark
/// to the prediction strip. Built once per panel open as a direct child of the
/// wrap row — exactly how the settings keyboard lays out its letter keys, the
/// pattern whose taps are known to land. Ten predictions cap the keypad: the
/// strip renders ten slots, and a fuller one would grow past the panel.
fn digit_chip(state: AppState, pk: Peek, v: u8) -> AnyPiece {
    column((
        label(format!("{v}"))
            .font(Font::Subheadline)
            .color(move || Color::hex(state.accent_color.get())),
    ))
    .padding(Insets {
        top: 5.0,
        leading: 9.0,
        bottom: 5.0,
        trailing: 9.0,
    })
    .corner_radius(8.0)
    .background(move || accent_tint(state.accent_color.get(), 0.12))
    .on_tap(move || {
        if pk.preds.get().len() >= 10 {
            return;
        }
        let mut p = pk.preds.get();
        p.push(v as f64);
        pk.preds.set(p);
    })
    .any()
}

/// One predicted chip's slot: a stable child that shows while there are at
/// least `idx + 1` predictions and hides past that. The label reads
/// `preds[idx]` reactively, so removing a chip slides every later chip's
/// value up a slot by itself — no keyed diffing between the finger and the
/// strip (the pattern whose taps were known to land: plain children).
fn pred_slot(state: AppState, pk: Peek, idx: usize) -> AnyPiece {
    when(
        move || pk.preds.get().len() > idx,
        move || {
            column((
                label("пред")
                    .font(Font::Caption2)
                    .color(move || Color::hex(state.accent_color.get())),
                label(move || {
                    pk.preds
                        .get()
                        .get(idx)
                        .map(|v| format!("~{}", v.round()))
                        .unwrap_or_default()
                })
                .font(Font::Subheadline)
                .color(move || Color::hex(state.accent_color.get())),
            ))
            .spacing(1.0)
            .padding(Insets {
                top: 4.0,
                leading: 6.0,
                bottom: 4.0,
                trailing: 6.0,
            })
            .background(move || accent_tint(state.accent_color.get(), 0.16))
            .corner_radius(8.0)
            .on_tap(move || {
                let mut v = pk.preds.get();
                if idx < v.len() {
                    v.remove(idx);
                    pk.preds.set(v);
                }
            })
            .any()
        },
    )
    .any()
}

fn build_card(state: AppState, pk: Peek, width: f64) -> impl Piece {
    let st_avg = state;
    let st_avg_col = state;
    let st_sum = state;

    // Header: the subject on the left, a compact ✕ on the right.
    let header = row((
        label(move || pk.subject.get().unwrap_or_default())
            .font(Font::Title3)
            .grow(),
        column((label("✕").font(Font::Subheadline),))
            .padding(Insets {
                top: 6.0,
                leading: 9.0,
                bottom: 6.0,
                trailing: 9.0,
            })
            .background(Color::rgba(0.0, 0.0, 0.0, 0.07))
            .corner_radius(8.0)
            .on_tap(move || peek().close()),
    ))
    .spacing(8.0)
    .align(VAlign::Center);

    // Real marks: one chip per lesson slot (date over the mark as spelled),
    // built as plain children when the panel mounts — the panel unmounts
    // between opens, so the list is fixed for its lifetime, and a finger has
    // no keyed diff to miss (the settings-keyboard pattern).
    let real_chips: Vec<AnyPiece> = {
        let q = pk.quarter.get();
        let subj = pk.subject.get().unwrap_or_default();
        state
            .week_cache
            .with(|c| crate::features::diary::extract_marks_dated(c, q, &subj))
            .into_iter()
            .map(|(date, raw, parsed)| {
                let pavg = parsed.iter().sum::<f64>() / parsed.len().max(1) as f64;
                column((
                    label(utils::format_date_short(date))
                        .font(Font::Caption2)
                        .color(colors::SECONDARY),
                    label(raw)
                        .font(Font::Subheadline)
                        .color(utils::avg_grade_color(pavg)),
                ))
                .spacing(1.0)
                .padding(Insets {
                    top: 4.0,
                    leading: 6.0,
                    bottom: 4.0,
                    trailing: 6.0,
                })
                .background(Color::rgba(0.0, 0.0, 0.0, 0.07))
                .corner_radius(8.0)
                .any()
            })
            .collect()
    };

    // Predicted chips: accent-tinted, prefixed with «~», tap removes — they sit
    // IN the strip among the real marks, highlighted. Ten stable slots, each
    // alive only while its index has a prediction behind it.
    let pred_slots: Vec<AnyPiece> = (0..10).map(|i| pred_slot(state, pk, i)).collect();

    // The average chip — the strip's last cell, projecting the predictions in.
    let avg_chip = column((
        label("средний").font(Font::Caption2).color(colors::SECONDARY),
        label(move || {
            let q = pk.quarter.get();
            let subj = pk.subject.get().unwrap_or_default();
            let marks = subject_marks(st_avg, &subj, q);
            let preds = pk.preds.get();
            if marks.is_empty() && preds.is_empty() {
                "—".into()
            } else {
                let n = (marks.len() + preds.len()) as f64;
                let sum: f64 = marks.iter().sum::<f64>() + preds.iter().sum::<f64>();
                format!("{:.2}", sum / n)
            }
        })
        .font(Font::Headline)
        .color(move || {
            if !pk.preds.get().is_empty() {
                return Color::hex(st_avg_col.accent_color.get());
            }
            let q = pk.quarter.get();
            let subj = pk.subject.get().unwrap_or_default();
            let marks = subject_marks(st_avg_col, &subj, q);
            avg_of(&marks).map(utils::avg_grade_color).unwrap_or(colors::SECONDARY)
        })
        .align(TextAlign::Center),
    ))
    .spacing(1.0)
    .padding(Insets {
        top: 4.0,
        leading: 8.0,
        bottom: 4.0,
        trailing: 8.0,
    })
    .background(Color::rgba(0.0, 0.0, 0.0, 0.13))
    .corner_radius(8.0);

    // «+» — opens the planning panel (predict mode first).
    let plus_chip = column((
        label("+").font(Font::Headline),
        label("что если").font(Font::Caption2).color(colors::SECONDARY),
    ))
    .spacing(1.0)
    .padding(Insets {
        top: 4.0,
        leading: 6.0,
        bottom: 4.0,
        trailing: 6.0,
    })
    .background(Color::rgba(0.0, 0.0, 0.0, 0.07))
    .corner_radius(8.0)
    .on_tap(move || {
        if pk.panel.get() == 0 {
            pk.panel.set(1);
        } else {
            pk.panel.set(0);
        }
    });

    // Wrapped into lines at the card's full width — every chip on screen at
    // once, nothing to swipe: no scroll view, so no bounce that could expose
    // the dim behind the card. One PieceVec of plain children — real marks,
    // prediction slots, the average, the «+» — the settings-keyboard layout.
    let mut cells = real_chips;
    cells.extend(pred_slots);
    cells.push(avg_chip.any());
    cells.push(plus_chip.any());
    let strip = row(PieceVec(cells))
        .spacing(6.0)
        .fit(RowFit::Wrap { run_spacing: 6.0 });

    // «Сейчас X → с предиктами Y» — only once something is predicted.
    let summary = when(
        move || !pk.preds.get().is_empty(),
        move || {
            label(move || {
                let q = pk.quarter.get();
                let subj = pk.subject.get().unwrap_or_default();
                let marks = subject_marks(st_sum, &subj, q);
                let preds = pk.preds.get();
                let cur = avg_of(&marks);
                let n = (marks.len() + preds.len()) as f64;
                let sum: f64 = marks.iter().sum::<f64>() + preds.iter().sum::<f64>();
                format!(
                    "Сейчас {} → с предиктами {:.2}",
                    cur.map(|v| format!("{v:.2}")).unwrap_or_else(|| "—".into()),
                    sum / n
                )
            })
            .font(Font::Footnote)
            .color(move || Color::hex(st_sum.accent_color.get()))
            .grow()
        },
    );

    // Mode switch — visible only while the panel is open.
    let mode_row = when(
        move || pk.panel.get() > 0,
        move || {
            row((
                seg_chip(state, pk, 1, "Предикт"),
                seg_chip(state, pk, 2, "Цель"),
                column((
                    label("Отмена")
                        .font(Font::Subheadline)
                        .color(colors::SECONDARY),
                ))
                .padding(Insets {
                    top: 5.0,
                    leading: 12.0,
                    bottom: 5.0,
                    trailing: 12.0,
                })
                .corner_radius(8.0)
                .background(Color::rgba(0.0, 0.0, 0.0, 0.06))
                .on_tap(move || pk.panel.set(0))
                .any(),
            ))
            .spacing(6.0)
            .grow()
        },
    );

    // Predict controls: tap any digit to append that predicted mark — no add
    // button, taps stack up, every append lands highlighted in the strip. The
    // digits sit DIRECTLY in the wrap row (PieceVec, the settings-keyboard
    // pattern) — no `each` anchor between the finger and the chip's tap.
    let predict_ui = when(
        move || pk.panel.get() == 1,
        move || {
            let chips: Vec<AnyPiece> = (1..=10u8).map(|v| digit_chip(state, pk, v)).collect();
            row(PieceVec(chips))
                .spacing(6.0)
                .fit(RowFit::Wrap { run_spacing: 6.0 })
                .grow()
        },
    );

    // Goal controls: whole-mark target + remaining-count steppers.
    let goal_ui = when(
        move || pk.panel.get() == 2,
        move || {
            column((
                row((
                    label("Цель по итогу:").grow(),
                    button("−").action(move || {
                        let v = pk.target.get() - 1.0;
                        pk.target.set(v.max(PEEK_MIN));
                    }),
                    label(move || format!("{:.0}", pk.target.get()))
                        .font(Font::Title3)
                        .width(44.0)
                        .align(TextAlign::Center),
                    button("+").action(move || {
                        let v = pk.target.get() + 1.0;
                        pk.target.set(v.min(PEEK_MAX));
                    }),
                ))
                .spacing(8.0)
                .align(VAlign::Center)
                .grow(),
                row((
                    label("Оценок останется:").grow(),
                    button("−").action(move || {
                        let k = pk.k.get();
                        if k > 0 {
                            pk.k.set(k - 1);
                        }
                    }),
                    label(move || format!("{}", pk.k.get()))
                        .font(Font::Headline)
                        .width(32.0)
                        .align(TextAlign::Center),
                    button("+").action(move || {
                        let k = pk.k.get();
                        if k < 40 {
                            pk.k.set(k + 1);
                        }
                    }),
                ))
                .spacing(8.0)
                .align(VAlign::Center)
                .grow(),
            ))
            .spacing(10.0)
            .grow()
        },
    );

    // The verdict: the whole mark needed, big and centered, then the math line.
    let verdict = when(
        move || pk.panel.get() == 2,
        move || {
            let st_big = state;
            let st_big_col = state;
            let st_hint = state;
            column((
                label(move || goal_report(st_big, pk).big)
                    .font(Font::Title2)
                    .color(move || goal_report(st_big_col, pk).color)
                    .align(TextAlign::Center)
                    .grow(),
                label(move || goal_report(st_hint, pk).hint)
                    .font(Font::Footnote)
                    .color(colors::SECONDARY)
                    .align(TextAlign::Center)
                    .grow(),
            ))
            .spacing(4.0)
            .grow()
        },
    );

    // The panel: an opaque slab pinned to the right edge, full height — the
    // section inside keeps its own card material, the padding keeps the
    // content clear of the status bar and the home indicator.
    column((
        section((
            header,
            strip,
            summary,
            mode_row,
            predict_ui,
            goal_ui,
            verdict,
        )),
    ))
    .padding(Insets {
        top: day::safe_area().top + 8.0,
        leading: 14.0,
        bottom: day::safe_area().bottom + 14.0,
        trailing: 16.0,
    })
    .background(colors::CARD)
    .frame(width, crate::pages::diary::get_screen_height())
}
