//! Итоги peek — a long-press on a subject row summons a full-width preview
//! card at a fixed spot on screen: the quarter's marks wrapped into lines
//! (date over mark), the average at the end of the strip, and a «+» that opens
//! the two prediction modes — a predicted mark with a live projected average,
//! and a target-grade plan (which marks would still be needed).
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
    /// The mark the predict stepper is parked on.
    pred_value: Signal<f64>,
    /// Target quarter average for the goal mode.
    target: Signal<f64>,
    /// How many marks are still to come (future lessons aren't in the cache).
    k: Signal<usize>,
    /// Fade-in opacity: 0 on open, animated to 1 one tick later.
    shown: Signal<f64>,
}

impl Peek {
    fn new() -> Self {
        Self {
            subject: Signal::new(None),
            quarter: Signal::new(0),
            preds: Signal::new(Vec::new()),
            panel: Signal::new(0),
            pred_value: Signal::new(8.0),
            target: Signal::new(8.0),
            k: Signal::new(5),
            shown: Signal::new(0.0),
        }
    }

    fn open(&self, subject: String, quarter: usize) {
        self.subject.set(Some(subject));
        self.quarter.set(quarter);
        self.preds.set(Vec::new());
        self.panel.set(0);
        self.pred_value.set(8.0);
        self.target.set(8.0);
        self.k.set(5);
        // Mount at opacity 0, then one animated write a tick later — the delay
        // lets the fresh subtree build before the fade lands on it.
        self.shown.set(0.0);
        let set = self.shown.setter();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(16));
            day::reactive::on_main(move || {
                with_animation(AnimSpec::ease_out(150), move || set.set(1.0));
            });
        });
    }

    fn close(&self) {
        self.subject.set(None);
        self.preds.set(Vec::new());
        self.panel.set(0);
    }
}

thread_local! {
    static PEEK: std::cell::Cell<Option<Peek>> = const { std::cell::Cell::new(None) };
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

/// Summon the peek — called from the Итоги row's `on_long_press`.
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

/// The goal-mode verdict: (text, color) — recomputed by both the label and its
/// tint (pure and cheap, and keeps the piece tree flat).
fn goal_report(state: AppState, pk: Peek) -> (String, Color) {
    let q = pk.quarter.get();
    let subj = pk.subject.get().unwrap_or_default();
    let marks = subject_marks(state, &subj, q);
    let n = marks.len() as f64;
    let sum: f64 = marks.iter().sum();
    let target = pk.target.get();
    let k = pk.k.get() as f64;

    if marks.is_empty() {
        return (
            "Нет оценок за четверть — считать не от чего.".into(),
            colors::SECONDARY,
        );
    }
    if k == 0.0 {
        let cur = sum / n;
        return if cur >= target {
            (
                format!("Оценок не осталось: цель {target:.1} уже держится (сейчас {cur:.2})."),
                colors::SUCCESS,
            )
        } else {
            (
                format!("Оценок не осталось: сейчас {cur:.2} — цели {target:.1} не хватит."),
                colors::ERROR,
            )
        };
    }

    // The sum the k remaining marks have to deliver.
    let need_total = target * (n + k) - sum;
    if need_total <= PEEK_FLOOR * k + 1e-9 {
        let with_floor = (sum + PEEK_FLOOR * k) / (n + k);
        return (
            format!(
                "Достаточно любых двоек — итог при всех двойках {with_floor:.2} ≥ цели {target:.1}."
            ),
            colors::SUCCESS,
        );
    }
    if need_total > PEEK_MAX * k + 1e-9 {
        let max = (sum + PEEK_MAX * k) / (n + k);
        return (
            format!(
                "Невозможно: даже все десятки дадут {max:.2} — цели {target:.1} не хватит."
            ),
            colors::ERROR,
        );
    }

    // Exact minimal sequence: need spread as evenly as integers allow —
    // `r` marks of ⌈need/k⌉ and the rest of ⌊need/k⌋, summing to exactly `need`.
    let need = need_total.ceil().min(PEEK_MAX * k);
    let ki = k as i64;
    let ni = need as i64;
    let x = ni / ki;
    let r = ni % ki;
    let mut list: Vec<i64> = Vec::with_capacity(ki as usize);
    for _ in 0..r {
        list.push(x + 1);
    }
    for _ in 0..(ki - r) {
        list.push(x);
    }
    list.sort_unstable_by(|a, b| b.cmp(a));
    let vals: Vec<String> = list.iter().map(|v| v.to_string()).collect();
    let shown = if vals.len() > 12 {
        format!("{}, …", vals[..12].join(", "))
    } else {
        vals.join(", ")
    };
    (
        format!(
            "Нужно набрать {need} за {ki} оценок (в среднем {:.2}). Например: {shown}.",
            need / k
        ),
        colors::SECONDARY,
    )
}

/// The fullscreen overlay: dim (tap to close) + the card.
///
/// The card is full-width and sits at a FIXED spot — the press point only picks
/// the subject, so where you long-press never moves the preview. Nothing inside
/// scrolls (the strip wraps instead), so there is no gesture that could slide
/// content and expose the dim behind the card.
pub fn overlay(state: AppState) -> impl Piece {
    let pk = peek();
    when(
        move || pk.subject.get().is_some(),
        move || {
            let w = crate::pages::diary::get_screen_width();
            zstack((
                button("")
                    .action(move || peek().close())
                    .background(Color::rgba(0.0, 0.0, 0.0, 0.42))
                    .grow(),
                build_card(state, pk, w),
            ))
            .align(Alignment::Center)
            .opacity(move || pk.shown.get())
            .grow()
        },
    )
    .grow()
}

fn build_card(state: AppState, pk: Peek, width: f64) -> impl Piece {
    let st_sum = state;
    let st_avg = state;
    let st_avg_col = state;
    let st_seed = state;
    let st_report = state;

    // Real marks: one chip per lesson slot (date over the mark as spelled).
    let real_chips = each(
        items(
            move || {
                let q = pk.quarter.get();
                let subj = pk.subject.get().unwrap_or_default();
                state
                    .week_cache
                    .with(|c| crate::features::diary::extract_marks_dated(c, q, &subj))
                    .into_iter()
                    .enumerate()
                    .collect::<Vec<_>>()
            },
            |t: &(usize, (u64, String, Vec<f64>))| t.0,
        ),
        move |item| {
            let (_, (date, raw, parsed)) = item.get();
            let pavg = parsed.iter().sum::<f64>() / parsed.len().max(1) as f64;
            column((
                label(utils::format_date_short(date))
                    .font(Font::Caption2)
                    .color(colors::SECONDARY),
                label(raw)
                    .font(Font::Headline)
                    .color(utils::avg_grade_color(pavg)),
            ))
            .spacing(2.0)
            .padding(Insets {
                top: 6.0,
                leading: 8.0,
                bottom: 6.0,
                trailing: 8.0,
            })
            .background(Color::rgba(0.0, 0.0, 0.0, 0.08))
            .corner_radius(8.0)
            .any()
        },
    );

    // Predicted chips: accent-tinted, prefixed with «~», tap removes.
    let pred_chips = each(
        items(
            move || {
                pk.preds
                    .get()
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (i, *v))
                    .collect::<Vec<_>>()
            },
            |t: &(usize, f64)| t.0,
        ),
        move |item| {
            let (idx, val) = item.get();
            let st = state;
            let pk_rm = pk;
            column((
                label("пред")
                    .font(Font::Caption2)
                    .color(move || Color::hex(st.accent_color.get())),
                label(format!("~{}", val.round()))
                    .font(Font::Headline)
                    .color(move || Color::hex(st.accent_color.get())),
            ))
            .spacing(2.0)
            .padding(Insets {
                top: 6.0,
                leading: 8.0,
                bottom: 6.0,
                trailing: 8.0,
            })
            .background(move || {
                let h = st.accent_color.get();
                Color::rgba(
                    ((h >> 16) & 0xff) as f64 / 255.0,
                    ((h >> 8) & 0xff) as f64 / 255.0,
                    (h & 0xff) as f64 / 255.0,
                    0.16,
                )
            })
            .corner_radius(8.0)
            .on_tap(move || {
                let mut v = pk_rm.preds.get();
                if idx < v.len() {
                    v.remove(idx);
                    pk_rm.preds.set(v);
                }
            })
            .any()
        },
    );

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
    .spacing(2.0)
    .padding(Insets {
        top: 6.0,
        leading: 8.0,
        bottom: 6.0,
        trailing: 8.0,
    })
    .background(Color::rgba(0.0, 0.0, 0.0, 0.14))
    .corner_radius(8.0);

    // «+» — opens the panel (predict mode first; seeded from the current avg).
    let plus_chip = column((
        label("+").font(Font::Title3),
        label("новая").font(Font::Caption2).color(colors::SECONDARY),
    ))
    .spacing(2.0)
    .padding(Insets {
        top: 6.0,
        leading: 8.0,
        bottom: 6.0,
        trailing: 8.0,
    })
    .background(Color::rgba(0.0, 0.0, 0.0, 0.08))
    .corner_radius(8.0)
    .on_tap(move || {
        if pk.panel.get() == 0 {
            let q = pk.quarter.get();
            let subj = pk.subject.get().unwrap_or_default();
            let m = subject_marks(st_seed, &subj, q);
            let v = avg_of(&m)
                .map(|a| a.round().clamp(PEEK_MIN, PEEK_MAX))
                .unwrap_or(8.0);
            pk.pred_value.set(v);
            pk.panel.set(1);
        } else {
            pk.panel.set(0);
        }
    });

    // Wrapped into lines at the card's full width — every chip on screen at
    // once, nothing to swipe: no scroll view, so no bounce that could expose
    // the dim behind the card.
    let strip = row((real_chips, pred_chips, avg_chip, plus_chip))
        .spacing(8.0)
        .fit(RowFit::Wrap { run_spacing: 8.0 });

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
            let st = state;
            row((
                button("Предикт").action(move || pk.panel.set(1)),
                button("Цель").action(move || {
                    let q = pk.quarter.get();
                    let subj = pk.subject.get().unwrap_or_default();
                    let m = subject_marks(st, &subj, q);
                    if let Some(a) = avg_of(&m) {
                        pk.target.set((a * 2.0).round() / 2.0);
                    }
                    pk.panel.set(2);
                }),
                button("Отмена").action(move || pk.panel.set(0)),
            ))
            .spacing(8.0)
            .grow()
        },
    );

    // Predict controls: stepper + projection + append.
    let predict_ui = when(
        move || pk.panel.get() == 1,
        move || {
            let st = state;
            column((
                row((
                    button("−").action(move || {
                        let v = pk.pred_value.get() - 1.0;
                        pk.pred_value.set(v.max(PEEK_MIN));
                    }),
                    label(move || format!("{}", pk.pred_value.get()))
                        .font(Font::Title2)
                        .width(44.0)
                        .align(TextAlign::Center),
                    button("+").action(move || {
                        let v = pk.pred_value.get() + 1.0;
                        pk.pred_value.set(v.min(PEEK_MAX));
                    }),
                    label("оценка").font(Font::Subheadline).color(colors::SECONDARY),
                ))
                .spacing(10.0)
                .align(VAlign::Center)
                .grow(),
                label(move || {
                    let q = pk.quarter.get();
                    let subj = pk.subject.get().unwrap_or_default();
                    let marks = subject_marks(st, &subj, q);
                    let base = avg_of(&marks).unwrap_or(0.0);
                    let n = marks.len() as f64 + 1.0;
                    let new_avg = (marks.iter().sum::<f64>() + pk.pred_value.get()) / n;
                    if marks.is_empty() {
                        format!("Первая оценка в четверти: станет {new_avg:.2}")
                    } else {
                        format!("Средний станет: {base:.2} → {new_avg:.2}")
                    }
                })
                .font(Font::Footnote)
                .color(colors::SECONDARY)
                .grow(),
                button("Добавить оценку")
                    .prominent()
                    .action(move || {
                        let mut p = pk.preds.get();
                        p.push(pk.pred_value.get());
                        pk.preds.set(p);
                    })
                    .grow(),
            ))
            .spacing(8.0)
            .grow()
        },
    );

    // Goal controls: target + remaining-count steppers.
    let goal_ui = when(
        move || pk.panel.get() == 2,
        move || {
            column((
                row((
                    label("Цель:").grow(),
                    button("−").action(move || {
                        let v = pk.target.get() - 0.5;
                        pk.target.set(v.max(PEEK_MIN));
                    }),
                    label(move || format!("{:.1}", pk.target.get()))
                        .font(Font::Headline)
                        .width(44.0)
                        .align(TextAlign::Center),
                    button("+").action(move || {
                        let v = pk.target.get() + 0.5;
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
            .spacing(8.0)
            .grow()
        },
    );

    // The verdict under the goal controls.
    let verdict = when(
        move || pk.panel.get() == 2,
        move || {
            label(move || goal_report(st_report, pk).0)
                .font(Font::Footnote)
                .color(move || goal_report(st_report, pk).1)
                .grow()
        },
    );

    section((
        label(move || pk.subject.get().unwrap_or_default())
            .font(Font::Title3)
            .grow(),
        strip,
        summary,
        mode_row,
        predict_ui,
        goal_ui,
        verdict,
    ))
    .width(width)
}
