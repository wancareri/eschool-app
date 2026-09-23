import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

content = content.replace(""".translation(strip_tx, 0.0),
                ))
                .on_drag(pager_drag(strip_tx.clone(), page_width.clone(), state.clone()))
            },
        ),
    ))
    .spacing(0.0)
    .background(Color::CLEAR)
}""",
""".translation(strip_tx, 0.0),
                ))
                .on_drag(pager_drag(strip_tx.clone(), page_width.clone(), state.clone()))
                .grow()
            },
        ),
    ))
    .spacing(0.0)
    .background(Color::CLEAR)
    .grow()
}""")

content = content.replace("""fn week_page(state: AppState, offset: i32, w: f64) -> impl Piece {
    let get_lessons = move || lessons_at(state, offset);
    column((
        when(
            move || true,
            move || widgets::week_summary::render(state, get_lessons),
        ),
        when(
            move || get_lessons().is_empty() && offset == 0 && state.lessons_loading.get(),
            || column((
                spacer(),
                spinner(),
                spacer(),
            )).height(300.0),
        ),
        when(
            move || get_lessons().is_empty() && (!state.lessons_loading.get() || offset != 0),
            || label("Нет данных за этот период")
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center)
                .padding(Insets { top: 24.0, leading: PAD, bottom: 24.0, trailing: PAD }),
        ),
        diary_list_with(state, get_lessons),
    ))
    .spacing(0.0)
    .width(w)
}""",
"""fn week_page(state: AppState, offset: i32, w: f64) -> impl Piece {
    let get_lessons = move || lessons_at(state, offset);
    column((
        when(
            move || true,
            move || widgets::week_summary::render(state, get_lessons),
        ),
        when(
            move || get_lessons().is_empty() && offset == 0 && state.lessons_loading.get(),
            || column((
                spacer(),
                spinner(),
                spacer(),
            )).height(300.0),
        ),
        when(
            move || get_lessons().is_empty() && (!state.lessons_loading.get() || offset != 0),
            || label("Нет данных за этот период")
                .font(Font::Body)
                .secondary()
                .align(TextAlign::Center)
                .padding(Insets { top: 24.0, leading: PAD, bottom: 24.0, trailing: PAD }),
        ),
        diary_list_with(state, get_lessons),
    ))
    .spacing(0.0)
    .width(w)
    .grow()
}""")

with open('src/pages/diary.rs', 'w') as f:
    f.write(content)
