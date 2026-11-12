use super::super::PAD;
use super::data::*;
use crate::app::AppState;
use crate::app::preferences::colors;
use crate::app::format;
use day::prelude::*;

fn peek_row<D: Decorate + 'static>(base: D, sj: String, st: AppState, offset: i32) -> AnyPiece {
    let sj_tap = sj.clone();
    crate::ios::long_press::attach(
        base.on_tap(move || {
            let cur_q = (st.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
            crate::ios::nslog::nslog(&format!(
                "[Diary] on_tap grade row: subject='{sj_tap}', quarter={cur_q}"
            ));
            crate::pages::diary::grade_peek::open(sj_tap.clone(), cur_q);
        }),
        move || {
            let cur_q = (st.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
            crate::pages::diary::grade_peek::open(sj.clone(), cur_q);
        },
    )
    .any()
}

pub(super) fn quarter_summary_at(state: AppState, offset: i32) -> impl Piece {
    let s_subj = state;
    let s_rows = state;
    each(
        items(
            move || get_all_subject_names(s_subj),
            |s: &String| s.clone(),
        ),
        move |item| {
            let subj_name = item.get();
            let sj1 = subj_name.clone();
            let sj2 = subj_name.clone();
            let sj3 = subj_name.clone();
            let sj4 = subj_name.clone();
            let sj5 = subj_name.clone();
            let st1 = s_rows;
            let st2 = s_rows;
            let st3 = s_rows;
            let st4 = s_rows;
            let st5 = s_rows;
            let base = column((
                row((
                    label(subj_name).font(Font::Body).grow(Dimension::Both),
                    // 1. Расчётный средний балл по оценкам четверти (реактивно!)
                    zstack((label(move || {
                        let cur_q =
                            (st1.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                        let marks = get_quarter_marks_map(st1, cur_q);
                        let avg = marks.get(&sj1).and_then(|v| {
                            if v.is_empty() {
                                None
                            } else {
                                Some(v.iter().sum::<f64>() / v.len() as f64)
                            }
                        });
                        match avg {
                            Some(v) => format!("{:.2}", v),
                            None => "—".into(),
                        }
                    })
                    .font(Font::Headline)
                    .color(move || {
                        let cur_q =
                            (st2.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                        let marks = get_quarter_marks_map(st2, cur_q);
                        let avg = marks.get(&sj2).and_then(|v| {
                            if v.is_empty() {
                                None
                            } else {
                                Some(v.iter().sum::<f64>() / v.len() as f64)
                            }
                        });
                        match avg {
                            Some(v) => format::avg_grade_color(v),
                            None => colors::SECONDARY,
                        }
                    })
                    .align(TextAlign::Center),))
                    .width(72.0),
                    // 2. Итоговая оценка четверти, выставленная учителем
                    // (/final/whole); без неё — прочерк, среднее не подставляется.
                    zstack((label(move || {
                        let cur_q =
                            (st3.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                        final_mark_text(final_mark(st3, &sj3, quarter_period(cur_q)))
                    })
                    .font(Font::Headline)
                    .color(move || {
                        let cur_q =
                            (st4.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                        match final_mark(st4, &sj4, quarter_period(cur_q))
                            .and_then(|v| v.parse::<f64>().ok())
                        {
                            Some(v) => format::avg_grade_color(v),
                            None => colors::SECONDARY,
                        }
                    })
                    .align(TextAlign::Center),))
                    .width(68.0),
                ))
                .spacing(8.0)
                .padding(Insets {
                    top: 8.0,
                    leading: PAD,
                    bottom: 8.0,
                    trailing: PAD,
                }),
                divider().padding(Insets {
                    top: 0.0,
                    leading: PAD,
                    bottom: 0.0,
                    trailing: PAD,
                }),
            ))
            .spacing(0.0);
            // Tap or long-press pops the grade peek right under this row — the
            // row's own view gives the window-space Y. The second event (a
            // long-press release also fires the tap) is absorbed by the peek's
            // idempotent open.
            peek_row(base, sj5, st5, offset)
        },
    )
}
