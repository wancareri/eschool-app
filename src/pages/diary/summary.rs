#[path = "summary_data.rs"]
mod data;
#[path = "summary_quarter.rs"]
mod quarter;
#[path = "summary_year.rs"]
mod year;

use self::data::*;
use self::{quarter::quarter_summary_at, year::year_summary_at};
use super::pager::summary_drag;
use super::{PAD, SWIPE_GEN, SWIPE_PENDING};
use crate::app::AppState;
use day::prelude::*;
use std::sync::atomic::Ordering;

pub(super) fn summary_view(state: AppState, period_revision: Signal<u64>) -> impl Piece {
    Scope::current().on_cleanup(|| {
        SWIPE_GEN.fetch_add(1, Ordering::AcqRel);
        SWIPE_PENDING.store(-1, Ordering::Release);
    });
    let page_width = Signal::new(crate::ios::viewport::get_screen_width());
    let drag_x = Signal::new(0.0);
    day::reactive::watch(
        move || (state.current_quarter.get(), period_revision.get()),
        move |_, _| drag_x.set(0.0),
    );
    let s_prev = state;
    let s_cur = state;
    let s_next = state;
    let w = page_width.get();
    column((crate::ios::drag::horizontal_pager(
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
        .align(Alignment::TopLeading),
        summary_drag(drag_x, page_width, state),
    )
    .width(w),))
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
            if q == 4 {
                "Итоги года"
            } else {
                "Итоги четверти"
            }
        })
        .font(Font::Headline)
        .color(move || Color::hex(state.accent_color.get()))
        .align(TextAlign::Center)
        .padding(Insets {
            top: 16.0,
            leading: PAD,
            bottom: 2.0,
            trailing: PAD,
        }),
        {
            let s = state;
            label(move || {
                let q = (s.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                if q == 4 {
                    let mut year_sum = 0.0;
                    let mut year_count = 0;
                    s.final_marks.with(|fm| {
                        for periods in fm.values() {
                            if let Some(m) = periods.get("YEAR").and_then(|v| v.parse::<f64>().ok())
                            {
                                year_sum += m;
                                year_count += 1;
                            }
                        }
                    });
                    if year_count > 0 {
                        format!("Средний балл за год: {:.2}", year_sum / year_count as f64)
                    } else {
                        let mut q_sum = 0.0;
                        let mut q_count = 0;
                        s.final_marks.with(|fm| {
                            for periods in fm.values() {
                                for q_idx in 0..4 {
                                    if let Some(m) = periods
                                        .get(quarter_period(q_idx))
                                        .and_then(|v| v.parse::<f64>().ok())
                                    {
                                        q_sum += m;
                                        q_count += 1;
                                    }
                                }
                            }
                        });
                        if q_count > 0 {
                            format!("Средний балл за год: {:.2}", q_sum / q_count as f64)
                        } else {
                            "Средний балл за год: —".to_string()
                        }
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
            .padding(Insets {
                top: 0.0,
                leading: PAD,
                bottom: 10.0,
                trailing: PAD,
            })
        },
        when(
            move || (header_state.current_quarter.get() as i32 + offset).clamp(0, 4) != 4,
            move || quarter_summary_header(),
        ),
        when(
            move || (header_state.current_quarter.get() as i32 + offset).clamp(0, 4) == 4,
            move || year_summary_header(),
        ),
        divider().padding(Insets {
            top: 0.0,
            leading: PAD,
            bottom: 0.0,
            trailing: PAD,
        }),
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
    // frame(w, 0.0) forced the cell node's height to zero (FrameLayout reports
    // Some(0.0) as-is) and a stretched label sits flush-left on UIKit; the
    // zstack keeps hug height while width() pins the column and OverlayLayout
    // centers the text.
    row((
        label("Предмет")
            .font(Font::Caption)
            .secondary()
            .grow(Dimension::Both),
        zstack((label("Ср. балл")
            .font(Font::Caption)
            .secondary()
            .align(TextAlign::Center),))
        .width(72.0),
        zstack((label("Выставл.")
            .font(Font::Caption)
            .secondary()
            .align(TextAlign::Center),))
        .width(68.0),
    ))
    .spacing(8.0)
    .padding(Insets {
        top: 6.0,
        leading: PAD,
        bottom: 6.0,
        trailing: PAD,
    })
}

fn year_summary_header() -> impl Piece {
    row((
        label("Предмет")
            .font(Font::Caption)
            .secondary()
            .grow(Dimension::Both),
        zstack((label("I")
            .font(Font::Caption)
            .secondary()
            .align(TextAlign::Center),))
        .width(28.0),
        zstack((label("II")
            .font(Font::Caption)
            .secondary()
            .align(TextAlign::Center),))
        .width(28.0),
        zstack((label("III")
            .font(Font::Caption)
            .secondary()
            .align(TextAlign::Center),))
        .width(28.0),
        zstack((label("IV")
            .font(Font::Caption)
            .secondary()
            .align(TextAlign::Center),))
        .width(28.0),
        zstack((label("Ср.")
            .font(Font::Caption)
            .secondary()
            .align(TextAlign::Center),))
        .width(42.0),
        zstack((label("Год")
            .font(Font::Caption)
            .secondary()
            .align(TextAlign::Center),))
        .width(42.0),
    ))
    .spacing(4.0)
    .padding(Insets {
        top: 6.0,
        leading: 12.0,
        bottom: 6.0,
        trailing: 12.0,
    })
}
