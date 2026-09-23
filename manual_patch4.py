import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

# 1. Add rect() inside day::prelude
content = content.replace("use day::prelude::*;", "use day::prelude::*;\nuse day::widgets::rect;")

# 2. Add global_drag implementation if it's missing
if "fn global_drag" not in content:
    content = content.replace("""fn pager_drag(""", """fn global_drag(
    strip_tx: Signal<f64>,
    page_width: Signal<f64>,
    state: AppState,
    show_summary: Signal<bool>,
) -> impl Fn(Drag) + 'static {
    let axis: Signal<Option<bool>> = Signal::new(None);
    move |drag: Drag| {
        let dx = drag.translation.x;
        let dy = drag.translation.y;
        let w = page_width.get();
        match drag.phase {
            DragPhase::Began => {
                axis.set(None);
                strip_tx.set(-w);
            }
            DragPhase::Changed => {
                let mut horiz = axis.get();
                if horiz.is_none() && (dx.abs() > SWIPE_AXIS_LOCK || dy.abs() > SWIPE_AXIS_LOCK) {
                    horiz = Some(dx.abs() >= dy.abs());
                    axis.set(horiz);
                }
                if horiz == Some(true) {
                    let mut x = dx;
                    if show_summary.get() {
                        let q = state.current_quarter.get();
                        if q == 0 && x > 0.0 { x *= SWIPE_EDGE_DAMP; }
                        if q >= 4 && x < 0.0 { x *= SWIPE_EDGE_DAMP; }
                    } else {
                        let idx = state.current_week_index.get();
                        let total = state.all_weeks.get().len() as i32;
                        if idx <= 0 && x > 0.0 { x *= SWIPE_EDGE_DAMP; }
                        if idx + 1 >= total && x < 0.0 { x *= SWIPE_EDGE_DAMP; }
                    }
                    strip_tx.set(-w + x);
                }
            }
            DragPhase::Ended => {
                let was_horiz = axis.get() == Some(true);
                axis.set(None);
                let actual_dx = strip_tx.get() + w;
                if was_horiz && actual_dx.abs() >= SWIPE_THRESHOLD {
                    if actual_dx < 0.0 {
                        // Swipe left -> next
                        if show_summary.get() {
                            let q = state.current_quarter.get();
                            if q < 4 {
                                state.current_quarter.set(q + 1);
                                strip_tx.set(strip_tx.get() + w);
                                day::reactive::with_animation(AnimSpec::ease_out(200), || {
                                    strip_tx.set(-w);
                                });
                                return;
                            }
                        } else {
                            let idx = state.current_week_index.get();
                            let total = state.all_weeks.get().len() as i32;
                            if idx + 1 < total {
                                features::diary::load_week(state, idx + 1);
                                strip_tx.set(strip_tx.get() + w);
                                day::reactive::with_animation(AnimSpec::ease_out(200), || {
                                    strip_tx.set(-w);
                                });
                                return;
                            }
                        }
                    }
                    if actual_dx > 0.0 {
                        // Swipe right -> prev
                        if show_summary.get() {
                            let q = state.current_quarter.get();
                            if q > 0 {
                                state.current_quarter.set(q - 1);
                                strip_tx.set(strip_tx.get() - w);
                                day::reactive::with_animation(AnimSpec::ease_out(200), || {
                                    strip_tx.set(-w);
                                });
                                return;
                            }
                        } else {
                            let idx = state.current_week_index.get();
                            if idx > 0 {
                                features::diary::load_week(state, idx - 1);
                                strip_tx.set(strip_tx.get() - w);
                                day::reactive::with_animation(AnimSpec::ease_out(200), || {
                                    strip_tx.set(-w);
                                });
                                return;
                            }
                        }
                    }
                }
                if was_horiz {
                    day::reactive::with_animation(AnimSpec::ease_out(200), || {
                        strip_tx.set(-w);
                    });
                }
            }
        }
    }
}

fn pager_drag(""")

content = content.replace("day::reactive::with_animation", "with_animation")

# 3. Replace render wrapper
old_render = """    pull_to_refresh(refreshing, scroll(column((
        zstack((
            column((
                label(move || res::str::diary_title().format())
                    .font(Font::LargeTitle)
                    .align(TextAlign::Center),
            ))
            .spacing(6.0)
            .padding(Insets { top: 16.0, leading: PAD, bottom: 4.0, trailing: PAD }),
            
            // Connection indicator overlay in top-left
            widgets::conn_status::render(),
        )),

        when(
            move || state.is_authenticated.get(),
            move || quarter_tabs(state),
        ),

        when(
            move || state.is_authenticated.get(),
            move || sub_tabs(state, show_summary),
        ),

        when(
            move || !state.is_authenticated.get(),
            || column((
                spacer(),
                label("Войдите для просмотра дневника")
                    .font(Font::Body).secondary().align(TextAlign::Center),
                spacer(),
            )).grow(),
        ),

        when(
            move || state.is_authenticated.get() && !show_summary.get(),
            move || week_view(state, page_width, strip_tx),
        ),
        when(
            move || state.is_authenticated.get() && show_summary.get(),
            move || summary_view(state, page_width, strip_tx),
        ),
    ))
    .spacing(0.0)
    .background(Color::CLEAR)
    .grow())
    .grow())
    .on_refresh(move || {
        let state = AppState::ambient();
        if state.is_authenticated.get() {
            features::diary::load_all(state);
        }
    })
}"""

new_render = """    zstack((
        rect().color(Color::rgba(255.0, 255.0, 255.0, 0.01)).grow(),
        pull_to_refresh(refreshing, scroll(column((
            zstack((
                column((
                    label(move || res::str::diary_title().format())
                        .font(Font::LargeTitle)
                        .align(TextAlign::Center),
                ))
                .spacing(6.0)
                .padding(Insets { top: 16.0, leading: PAD, bottom: 4.0, trailing: PAD }),
                
                // Connection indicator overlay in top-left
                widgets::conn_status::render(),
            )),

            when(
                move || state.is_authenticated.get(),
                move || quarter_tabs(state),
            ),

            when(
                move || state.is_authenticated.get(),
                move || sub_tabs(state, show_summary),
            ),

            when(
                move || !state.is_authenticated.get(),
                || column((
                    spacer(),
                    label("Войдите для просмотра дневника")
                        .font(Font::Body).secondary().align(TextAlign::Center),
                    spacer(),
                )).grow(),
            ),

            when(
                move || state.is_authenticated.get() && !show_summary.get(),
                move || week_view(state, page_width, strip_tx),
            ),
            when(
                move || state.is_authenticated.get() && show_summary.get(),
                move || summary_view(state, page_width, strip_tx),
            ),
        ))
        .spacing(0.0)
        .background(Color::CLEAR)
        .grow())
        .grow())
        .on_refresh(move || {
            let state = AppState::ambient();
            if state.is_authenticated.get() {
                features::diary::load_all(state);
            }
        })
    ))
    .on_drag(global_drag(strip_tx.clone(), page_width.clone(), state.clone(), show_summary.clone()))
    .grow()
}"""

content = content.replace(old_render, new_render)

# 4. Remove pager_drag and summary_drag from week_view and summary_view
content = re.sub(r"""\s*\.on_drag\(pager_drag\([^)]*\)\)""", "", content)
content = re.sub(r"""\s*\.on_drag\(summary_drag\([^)]*\)\)""", "", content)

with open('src/pages/diary.rs', 'w') as f:
    f.write(content)
