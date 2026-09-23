import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

content = content.replace(""".translation(strip_tx, 0.0),
                ))
                .on_drag(summary_drag(strip_tx.clone(), page_width.clone(), state.clone()))
            },
        ),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
}""",
""".translation(strip_tx, 0.0),
                ))
                .on_drag(summary_drag(strip_tx.clone(), page_width.clone(), state.clone()))
                .grow()
            },
        ),
    ))
    .spacing(0.0)
    .padding(Insets { top: 0.0, leading: 0.0, bottom: 8.0, trailing: 0.0 })
    .grow()
}""")

content = content.replace("""fn summary_page(state: AppState, offset: i32, w: f64) -> impl Piece {
    let q = (state.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
    let title_state = state;
    let header_state = state;
    let qs_state = state;
    let ys_state = state;
    column((
        label(move || {
            let q = (title_state.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
            if q == 4 { "Итоги года" } else { "Итоги четверти" }
        })
        .font(Font::Headline)
        .color(move || Color::hex(state.accent_color.get()))
        .align(TextAlign::Center)
        .padding(Insets { top: 16.0, leading: PAD, bottom: 2.0, trailing: PAD }),
        
        {
            let s = state;
            label(move || {
                let q = (s.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                let off: std::collections::HashMap<String, Vec<OfficialMark>> = if offset == 0 {
                    s.official_marks.get()
                } else {
                    s.quarter_official_marks.get().get(q).cloned().unwrap_or_default()
                };
                let mut sum = 0.0;
                let mut count = 0;
                for marks in off.values() {
                    for m in marks {
                        sum += m.value;
                        count += 1;
                    }
                }
                if count > 0 {
                    format!("Средний балл: {:.2}", sum / count as f64)
                } else {
                    "".to_string()
                }
            })
            .font(Font::Subheadline)
            .secondary()
            .align(TextAlign::Center)
            .padding(Insets { top: 0.0, leading: PAD, bottom: 12.0, trailing: PAD })
        },

        when(
            move || q < 4,
            move || widgets::quarter_stats::render(qs_state.clone(), q),
        ),
        when(
            move || q == 4,
            move || widgets::quarter_stats::render_year(ys_state.clone()),
        ),
    ))
    .spacing(0.0)
    .width(w)
}""",
"""fn summary_page(state: AppState, offset: i32, w: f64) -> impl Piece {
    let q = (state.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
    let title_state = state;
    let header_state = state;
    let qs_state = state;
    let ys_state = state;
    column((
        label(move || {
            let q = (title_state.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
            if q == 4 { "Итоги года" } else { "Итоги четверти" }
        })
        .font(Font::Headline)
        .color(move || Color::hex(state.accent_color.get()))
        .align(TextAlign::Center)
        .padding(Insets { top: 16.0, leading: PAD, bottom: 2.0, trailing: PAD }),
        
        {
            let s = state;
            label(move || {
                let q = (s.current_quarter.get() as i32 + offset).clamp(0, 4) as usize;
                let off: std::collections::HashMap<String, Vec<OfficialMark>> = if offset == 0 {
                    s.official_marks.get()
                } else {
                    s.quarter_official_marks.get().get(q).cloned().unwrap_or_default()
                };
                let mut sum = 0.0;
                let mut count = 0;
                for marks in off.values() {
                    for m in marks {
                        sum += m.value;
                        count += 1;
                    }
                }
                if count > 0 {
                    format!("Средний балл: {:.2}", sum / count as f64)
                } else {
                    "".to_string()
                }
            })
            .font(Font::Subheadline)
            .secondary()
            .align(TextAlign::Center)
            .padding(Insets { top: 0.0, leading: PAD, bottom: 12.0, trailing: PAD })
        },

        when(
            move || q < 4,
            move || widgets::quarter_stats::render(qs_state.clone(), q),
        ),
        when(
            move || q == 4,
            move || widgets::quarter_stats::render_year(ys_state.clone()),
        ),
    ))
    .spacing(0.0)
    .width(w)
    .grow()
}""")

with open('src/pages/diary.rs', 'w') as f:
    f.write(content)
