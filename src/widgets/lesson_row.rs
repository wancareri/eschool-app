use eschool_api::entities::*;
use crate::shared::{colors, utils};
use day::prelude::*;

const RAIL_GAP: f64 = 10.0;
const DETAIL_LEFT: f64 = 16.0;

fn mark_of(s: &LessonSlot) -> Option<&str> {
    s.lesson_mark
        .as_ref()
        .and_then(|m| m.mark.as_deref())
        .filter(|m| !m.trim().is_empty())
}

pub fn render(slot: ItemSlot<LessonSlot, u32>) -> impl Piece {
    let s_num = slot;
    let s_time = slot;
    let s_title = slot;
    let s_topic = slot;
    let s_mark = slot;
    let s_has_comment = slot;
    let s_comment = slot;
    let s_has_hw = slot;
    let s_hw = slot;
    let s_has_msg = slot;
    let s_msg = slot;

    column((
        row((
            column((
                label(move || s_num.with(|s| s.number.to_string()))
                    .font(Font::Title3)
                    .bold()
                    .tabular(),
                label(move || s_time.with(|s| utils::strip_seconds(&s.start_time)))
                    .font(Font::Caption)
                    .secondary()
                    .tabular(),
            ))
            .spacing(3.0)
            .align(HAlign::Center),
            column((
                label(move || s_title.with(|s| s.subject_title.clone()))
                    .font(Font::Body)
                    .weight(FontWeight::Semibold),
                when(
                    move || s_topic.with(|s| {
                        s.topic.as_deref().map(|t| !t.is_empty()).unwrap_or(false)
                    }),
                    move || label(move || s_topic.with(|s| s.topic.clone().unwrap_or_default()))
                        .font(Font::Caption)
                        .secondary(),
                ),
            ))
            .spacing(1.0)
            .align(HAlign::Leading)
            .grow(),
            when(
                move || s_mark.with(|s| mark_of(s).is_some()),
                move || {
                    let s_pill = s_mark;
                    label(move || s_pill.with(|s| mark_of(s).unwrap_or("").to_string()))
                        .font(Font::Title3)
                        .bold()
                        .tabular()
                        .color(move || s_pill.with(|s| utils::grade_color(mark_of(s).unwrap_or(""))))
                        .background(move || {
                            s_pill
                                .with(|s| utils::grade_color(mark_of(s).unwrap_or("")))
                                .with_alpha(0.16)
                        })
                        .corner_radius(8.0)
                        .padding(Insets { top: 1.0, leading: 8.0, bottom: 1.0, trailing: 8.0 })
                },
            ),
            when(
                move || s_mark.with(|s| mark_of(s).is_none()),
                move || label("—")
                    .font(Font::Title3)
                    .tabular()
                    .color(colors::GRAY_400),
            ),
        ))
        .spacing(RAIL_GAP)
        .align(VAlign::Center)
        .grow()
        .padding(Insets { top: 8.0, leading: 16.0, bottom: 2.0, trailing: 16.0 }),

        when(
            move || s_has_comment.with(|s| {
                s.lesson_mark
                    .as_ref()
                    .and_then(|m| m.comment.as_ref())
                    .map(|c| !c.is_empty())
                    .unwrap_or(false)
            }),
            move || label(move || {
                s_comment
                    .with(|s| s.lesson_mark.as_ref().and_then(|m| m.comment.clone()).unwrap_or_default())
            })
            .font(Font::Caption)
            .italic()
            .secondary()
            .padding(Insets { top: 1.0, leading: DETAIL_LEFT, bottom: 1.0, trailing: 16.0 }),
        ),

        when(
            move || s_has_hw.with(|s| s.homework.as_ref().map(|h| !h.is_empty()).unwrap_or(false)),
            move || label(move || {
                s_hw.with(|s| format!("✎ {}", s.homework.clone().unwrap_or_default()))
            })
            .font(Font::Caption)
            .weight(FontWeight::Semibold)
            .color(colors::ACCENT)
            .padding(Insets { top: 2.0, leading: DETAIL_LEFT, bottom: 2.0, trailing: 16.0 }),
        ),

        when(
            move || s_has_msg.with(|s| s.message.as_ref().map(|m| !m.is_empty()).unwrap_or(false)),
            move || label(move || {
                s_msg.with(|s| format!("ℹ {}", s.message.as_ref().map(|m| m.as_str()).unwrap_or("")))
            })
            .font(Font::Caption)
            .secondary()
            .padding(Insets { top: 1.0, leading: DETAIL_LEFT, bottom: 2.0, trailing: 16.0 }),
        ),

        divider().padding(Insets { top: 4.0, leading: 16.0, bottom: 0.0, trailing: 0.0 }),
    ))
    .spacing(0.0)
    .align(HAlign::Leading)
}
