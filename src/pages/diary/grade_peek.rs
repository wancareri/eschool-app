
use crate::app::AppState;
use crate::app::preferences::colors;
use crate::app::preferences::haptics;
use crate::app::format;
use day::prelude::*;

const PEEK_MIN: f64 = 1.0;
const PEEK_MAX: f64 = 10.0;
const PEEK_FLOOR: f64 = 2.0;

#[derive(Clone, Copy)]
struct Peek {
    subject: Signal<Option<String>>,
    quarter: Signal<usize>,
    preds: Signal<Vec<f64>>,
    panel: Signal<u8>,
    target: Signal<f64>,
    k: Signal<usize>,
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
        }
    }

    fn open(&self, subject: String, quarter: usize) {
        crate::ios::nslog::nslog(&format!("[GradePeek] open: subject='{subject}', quarter={quarter}"));
        self.quarter.set(quarter);
        self.preds.set(Vec::new());
        self.panel.set(0);
        self.target.set(5.0);
        self.k.set(5);
        self.subject.set(Some(subject));
    }

    fn close(&self) {
        crate::ios::nslog::nslog("[GradePeek] close: setting subject=None");
        self.subject.set(None);
    }
}

thread_local! {
    static PEEK: std::cell::Cell<Option<Peek>> = const { std::cell::Cell::new(None) };
}

fn peek() -> Peek {
    PEEK.with(|c| match c.get() {
        Some(p) => p,
        None => {
            let p = Scope::root().enter(Peek::new);
            c.set(Some(p));
            p
        }
    })
}

pub fn open(subject: String, quarter: usize) {
    haptics::pop();
    peek().open(subject, quarter);
}

fn subject_marks(state: AppState, subject: &str, quarter: usize) -> Vec<f64> {
    state.week_cache.with(|c| {
        let dated = crate::pages::diary::data::extract_marks_dated(c, &state.week_quarters.get(), quarter, subject);
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

fn accent_tint(hex: u32, alpha: f64) -> Color {
    Color::rgba(
        ((hex >> 16) & 0xff) as f64 / 255.0,
        ((hex >> 8) & 0xff) as f64 / 255.0,
        (hex & 0xff) as f64 / 255.0,
        alpha,
    )
}

struct GoalReport {
    big: String,
    hint: String,
    color: Color,
}

fn goal_report(state: AppState, pk: Peek) -> GoalReport {
    let q = pk.quarter.get();
    let subj = pk.subject.get().unwrap_or_default();
    let marks = subject_marks(state, &subj, q);
    let n = marks.len() as f64;
    let sum: f64 = marks.iter().sum();
    let target = pk.target.get();
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

pub fn modal(state: AppState) -> impl Piece {
    let pk = peek();
    let min_h = crate::ios::viewport::get_screen_height() * 0.5;
    crate::components::modal::modal_window(pk.subject, min_h, move |subj| {
        crate::ios::nslog::nslog(&format!("[GradePeek] build_card invoked, subj='{subj}'"));
        build_card(state, pk)
    })
}

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
    .background(move || accent_tint(state.accent_color.get(), 0.12))
    .corner_radius(14.0)
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
            .align(HAlign::Center)
            .grow(Dimension::Width)
            .padding(Insets {
                top: 4.0,
                leading: 6.0,
                bottom: 4.0,
                trailing: 6.0,
            })
            .background(move || accent_tint(state.accent_color.get(), 0.16))
            .corner_radius(8.0)
            .grow(Dimension::Width)
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

fn build_card(state: AppState, pk: Peek) -> impl Piece {
    let st_avg = state;
    let st_avg_col = state;
    let st_sum = state;

    let header = row((
        label(move || pk.subject.get().unwrap_or_default())
            .font(Font::Title3)
            .grow(Dimension::Width),
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

    let real_chips: Vec<AnyPiece> = {
        let q = pk.quarter.get();
        let subj = pk.subject.get().unwrap_or_default();
        state
            .week_cache
            .with(|c| crate::pages::diary::data::extract_marks_dated(c, &state.week_quarters.get(), q, &subj))
            .into_iter()
            .map(|(date, raw, parsed)| {
                let pavg = parsed.iter().sum::<f64>() / parsed.len().max(1) as f64;
                column((
                    label(format::format_date_short(date))
                        .font(Font::Caption2)
                        .color(colors::SECONDARY),
                    label(raw)
                        .font(Font::Subheadline)
                        .color(format::avg_grade_color(pavg)),
                ))
                .spacing(1.0)
                .align(HAlign::Center)
                .grow(Dimension::Width)
                .padding(Insets {
                    top: 4.0,
                    leading: 6.0,
                    bottom: 4.0,
                    trailing: 6.0,
                })
                .background(Color::rgba(0.0, 0.0, 0.0, 0.07))
                .corner_radius(8.0)
                .grow(Dimension::Width)
                .any()
            })
            .collect()
    };

    let pred_slots: Vec<AnyPiece> = (0..10).map(|i| pred_slot(state, pk, i)).collect();

    let avg_chip = column((
        label("средний")
            .font(Font::Caption2)
            .color(colors::SECONDARY),
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
            avg_of(&marks).map(format::avg_grade_color).unwrap_or(colors::SECONDARY)
        })
        .align(TextAlign::Center),
    ))
    .spacing(1.0)
    .align(HAlign::Center)
    .grow(Dimension::Width)
    .padding(Insets {
        top: 4.0,
        leading: 8.0,
        bottom: 4.0,
        trailing: 8.0,
    })
    .background(Color::rgba(0.0, 0.0, 0.0, 0.13))
    .corner_radius(8.0)
    .grow(Dimension::Width);

    let plus_chip = column((
        zstack((
            column((
                label(" ")
                    .font(Font::Caption2)
                    .color(colors::SECONDARY),
                label(" ")
                    .font(Font::Subheadline),
            ))
            .spacing(1.0),
            label("+")
                .font(Font::Subheadline),
        )),
    ))
    .spacing(1.0)
    .align(HAlign::Center)
    .grow(Dimension::Width)
    .padding(Insets {
        top: 4.0,
        leading: 6.0,
        bottom: 4.0,
        trailing: 6.0,
    })
    .background(move || {
        if pk.panel.get() == 1 {
            accent_tint(state.accent_color.get(), 0.14)
        } else {
            Color::rgba(0.0, 0.0, 0.0, 0.07)
        }
    })
    .corner_radius(8.0)
    .grow(Dimension::Width)
    .on_tap(move || {
        haptics::pop();
        pk.panel.set(if pk.panel.get() == 1 { 0 } else { 1 });
    });

    let mut cells = real_chips;
    cells.extend(pred_slots);
    cells.push(avg_chip.any());
    cells.push(plus_chip.any());
    let strip = row(cells)
        .spacing(6.0)
        .fit(RowFit::WrapColumns { run_spacing: 6.0 });

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
            .grow(Dimension::Width)
        },
    );

    let goal_btn = column((
        label("Цель")
            .font(Font::Subheadline)
            .color(move || {
                if pk.panel.get() == 2 {
                    Color::hex(state.accent_color.get())
                } else {
                    colors::SECONDARY
                }
            }),
    ))
    .grow(Dimension::Width)
    .padding(Insets {
        top: 8.0,
        leading: 12.0,
        bottom: 8.0,
        trailing: 12.0,
    })
    .background(move || {
        if pk.panel.get() == 2 {
            accent_tint(state.accent_color.get(), 0.14)
        } else {
            Color::rgba(0.0, 0.0, 0.0, 0.06)
        }
    })
    .corner_radius(14.0)
    .grow(Dimension::Width)
    .on_tap(move || {
        if pk.panel.get() == 2 {
            pk.panel.set(0);
        } else {
            let q = pk.quarter.get();
            let subj = pk.subject.get().unwrap_or_default();
            if let Some(a) = avg_of(&subject_marks(state, &subj, q)) {
                pk.target.set(a.round().clamp(PEEK_MIN, PEEK_MAX));
            }
            pk.panel.set(2);
        }
    });

    let predict_ui = when(
        move || pk.panel.get() == 1,
        move || {
            let chips: Vec<AnyPiece> = (1..=10u8).map(|v| digit_chip(state, pk, v)).collect();
            crate::components::centered_wrap::centered_wrap(chips, 6.0, 6.0)
                .grow(Dimension::Width)
        },
    );

    let goal_ui = when(
        move || pk.panel.get() == 2,
        move || {
            column((
                row((
                    label("Цель по итогу:").grow(Dimension::Width),
                    button("−").action(move || {
                        let v = pk.target.get() - 1.0;
                        pk.target.set(v.max(PEEK_MIN));
                    }),
                    column((
                        label(move || format!("{:.0}", pk.target.get())).font(Font::Title3),
                    ))
                    .width(44.0),
                    button("+").action(move || {
                        let v = pk.target.get() + 1.0;
                        pk.target.set(v.min(PEEK_MAX));
                    }),
                ))
                .spacing(8.0)
                .align(VAlign::Center)
                .grow(Dimension::Width),
                row((
                    label("Оценок останется:").grow(Dimension::Width),
                    button("−").action(move || {
                        let k = pk.k.get();
                        if k > 0 {
                            pk.k.set(k - 1);
                        }
                    }),
                    column((
                        label(move || format!("{}", pk.k.get())).font(Font::Headline),
                    ))
                    .width(32.0),
                    button("+").action(move || {
                        let k = pk.k.get();
                        if k < 40 {
                            pk.k.set(k + 1);
                        }
                    }),
                ))
                .spacing(8.0)
                .align(VAlign::Center)
                .grow(Dimension::Width),
            ))
            .spacing(10.0)
            .grow(Dimension::Width)
        },
    );

    let verdict = when(
        move || pk.panel.get() == 2,
        move || {
            let st_big = state;
            let st_big_col = state;
            let st_hint = state;
            column((
                label(move || goal_report(st_big, pk).big)
                    .font(Font::Title2)
                    .color(move || goal_report(st_big_col, pk).color),
                label(move || goal_report(st_hint, pk).hint)
                    .font(Font::Footnote)
                    .color(colors::SECONDARY),
            ))
            .spacing(4.0)
            .grow(Dimension::Width)
        },
    );

    column((
        header,
        strip,
        summary,
        goal_btn,
        predict_ui,
        goal_ui,
        verdict,
    ))
    .spacing(10.0)
    .grow(Dimension::Width)
}

pub fn close() { peek().close(); }
