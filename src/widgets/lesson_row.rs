use eschool_api::entities::*;
use crate::shared::{colors, utils};
use day::prelude::*;

const NUM_WIDTH: f64 = 28.0;
const HW_LEFT: f64 = 16.0 + NUM_WIDTH + 12.0;

pub fn render(slot: ItemSlot<LessonSlot, u32>) -> impl Piece {
    let s_num = slot;
    let s_title = slot;
    let s_time = slot;
    let s_mark = slot;
    let s_color = slot;
    let s_has_comment = slot;
    let s_comment = slot;
    let s_has_hw = slot;
    let s_hw = slot;
    let s_has_msg = slot;
    let s_msg = slot;

    column((
        row((
            label(move || s_num.with(|s| s.number.to_string()))
                .font(Font::Caption)
                .secondary()
                .frame(NUM_WIDTH, 20.0),
            column((
                label(move || s_title.with(|s| s.subject_title.clone()))
                    .font(Font::Body),
                label(move || {
                    s_time.with(|s| {
                        let t = &s.start_time;
                        let time = t.get(..5).unwrap_or(t);
                        match &s.topic {
                            Some(topic) if !topic.is_empty() => format!("{time} · {topic}"),
                            _ => time.to_string(),
                        }
                    })
                })
                .font(Font::Caption)
                .secondary(),
            ))
            .spacing(2.0)
            .align(HAlign::Leading)
            .grow(),
            label(move || {
                s_mark.with(|s| {
                    s.lesson_mark.as_ref()
                        .and_then(|m| m.mark.clone())
                        .unwrap_or_else(|| "—".into())
                })
            })
            .font(Font::Title3)
            .color(move || {
                s_color.with(|s| {
                    let mark = s.lesson_mark.as_ref()
                        .and_then(|m| m.mark.as_deref())
                        .unwrap_or("—");
                    utils::grade_color(mark)
                })
            }),
        ))
        .spacing(12.0)
        .padding(Insets { top: 8.0, leading: 16.0, bottom: 0.0, trailing: 20.0 }),

        when(
            move || s_has_comment.with(|s| s.lesson_mark.as_ref().and_then(|m| m.comment.as_ref()).map(|c| !c.is_empty()).unwrap_or(false)),
            move || label(move || s_comment.with(|s| s.lesson_mark.as_ref().and_then(|m| m.comment.clone()).unwrap_or_default()))
                .font(Font::Caption)
                .color(colors::SECONDARY)
                .padding(Insets { top: 2.0, leading: HW_LEFT, bottom: 2.0, trailing: 20.0 }),
        ),

        when(
            move || s_has_hw.with(|s| s.homework.as_ref().map(|h| !h.is_empty()).unwrap_or(false)),
            move || label(move || s_hw.with(|s| s.homework.clone().unwrap_or_default()))
                .font(Font::Caption)
                .color(colors::ACCENT)
                .padding(Insets { top: 4.0, leading: HW_LEFT, bottom: 4.0, trailing: 20.0 }),
        ),

        when(
            move || s_has_msg.with(|s| s.message.as_ref().map(|m| !m.is_empty()).unwrap_or(false)),
            move || label(move || s_msg.with(|s| s.message.as_ref().map(|m| format!("ℹ {m}")).unwrap_or_default()))
                .font(Font::Caption)
                .color(colors::SECONDARY)
                .padding(Insets { top: 2.0, leading: HW_LEFT, bottom: 8.0, trailing: 20.0 }),
        ),
    ))
    .spacing(0.0)
}
