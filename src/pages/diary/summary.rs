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
        when(
            move || (title_state.current_quarter.get() as i32 + offset).clamp(0, 4) == 4,
            move || year_summary_average(state),
        ),
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
            move || (title_state.current_quarter.get() as i32 + offset).clamp(0, 4) != 4,
            move || quarter_summary_footer(state, offset),
        ),
        when(
            move || (title_state.current_quarter.get() as i32 + offset).clamp(0, 4) == 4,
            move || year_summary_at(ys_state),
        ),
    ))
    .spacing(0.0)
    .width(w)
}

fn year_summary_average(state: AppState) -> impl Piece {
    label(move || {
        let (sum, count) = state.final_marks.with(|marks| {
            marks
                .values()
                .filter_map(|periods| periods.get("YEAR"))
                .filter_map(|mark| mark.parse::<f64>().ok())
                .fold((0.0, 0usize), |(sum, count), mark| (sum + mark, count + 1))
        });
        if count > 0 {
            format!("Средний балл за год: {:.2}", sum / count as f64)
        } else {
            "Средний балл за год: —".to_string()
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
}

fn quarter_summary_footer(state: AppState, offset: i32) -> impl Piece {
    let footer_state = state;
    column((
        divider().padding(Insets {
            top: 0.0,
            leading: PAD,
            bottom: 0.0,
            trailing: PAD,
        }),
        label(move || {
            let q = (footer_state.current_quarter.get() as i32 + offset).clamp(0, 3) as usize;
            match rounded_quarter_final_average(footer_state, q) {
                Some(avg) => format!("Средний балл: {avg}"),
                None => "Средний балл: —".to_string(),
            }
        })
        .font(Font::Headline)
        .grow(Dimension::Width)
        .align(TextAlign::Trailing)
        .padding(Insets {
            top: 12.0,
            leading: PAD,
            bottom: 16.0,
            trailing: PAD + 68.0,
        }),
    ))
    .spacing(0.0)
    .width(crate::ios::viewport::get_screen_width())
}

fn quarter_summary_header() -> impl Piece {
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
