import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

# Remove global_drag from pull_to_refresh
content = re.sub(r'\.on_drag\(global_drag\(drag_tx, drag_width, drag_state, show_summary_drag\)\)\n\s*\.grow\(\)', '.grow()', content)

# Add pager_drag and summary_drag back
drag_code = """
fn pager_drag(
    strip_tx: Signal<f64>,
    page_width: Signal<f64>,
    state: AppState,
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
                    let idx = state.current_week_index.get();
                    let total = state.all_weeks.get().len() as i32;
                    let mut x = dx;
                    if idx <= 0 && x > 0.0 {
                        x *= SWIPE_EDGE_DAMP;
                    }
                    if idx + 1 >= total && x < 0.0 {
                        x *= SWIPE_EDGE_DAMP;
                    }
                    strip_tx.set(-w + x);
                }
            }
            DragPhase::Ended => {
                let was_horiz = axis.get() == Some(true);
                axis.set(None);
                let idx = state.current_week_index.get();
                let total = state.all_weeks.get().len() as i32;
                let actual_dx = strip_tx.get() + w;
                if was_horiz && actual_dx.abs() >= SWIPE_THRESHOLD {
                    if actual_dx < 0.0 && idx + 1 < total {
                        features::diary::load_week(state, idx + 1);
                        strip_tx.set(strip_tx.get() + w);
                        with_animation(AnimSpec::ease_out(200), || {
                            strip_tx.set(-w);
                        });
                        return;
                    }
                    if actual_dx > 0.0 && idx > 0 {
                        features::diary::load_week(state, idx - 1);
                        strip_tx.set(strip_tx.get() - w);
                        with_animation(AnimSpec::ease_out(200), || {
                            strip_tx.set(-w);
                        });
                        return;
                    }
                }
                if was_horiz {
                    with_animation(AnimSpec::ease_out(200), || {
                        strip_tx.set(-w);
                    });
                }
            }
        }
    }
}

fn summary_drag(
    strip_tx: Signal<f64>,
    page_width: Signal<f64>,
    state: AppState,
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
                    let q = state.current_quarter.get();
                    let mut x = dx;
                    if q <= 0 && x > 0.0 {
                        x *= SWIPE_EDGE_DAMP;
                    }
                    if q >= 4 && x < 0.0 {
                        x *= SWIPE_EDGE_DAMP;
                    }
                    strip_tx.set(-w + x);
                }
            }
            DragPhase::Ended => {
                let was_horiz = axis.get() == Some(true);
                axis.set(None);
                let q = state.current_quarter.get();
                let actual_dx = strip_tx.get() + w;
                if was_horiz && actual_dx.abs() >= SWIPE_THRESHOLD {
                    if actual_dx < 0.0 && q + 1 <= 4 {
                        select_quarter(state, q + 1);
                        strip_tx.set(strip_tx.get() + w);
                        with_animation(AnimSpec::ease_out(200), || {
                            strip_tx.set(-w);
                        });
                        return;
                    }
                    if actual_dx > 0.0 && q > 0 {
                        select_quarter(state, q - 1);
                        strip_tx.set(strip_tx.get() - w);
                        with_animation(AnimSpec::ease_out(200), || {
                            strip_tx.set(-w);
                        });
                        return;
                    }
                }
                if was_horiz {
                    with_animation(AnimSpec::ease_out(200), || {
                        strip_tx.set(-w);
                    });
                }
            }
        }
    }
}
"""

content = content.replace("fn week_view(state: AppState, page_width: Signal<f64>, strip_tx: Signal<f64>) -> impl Piece {", drag_code + "\nfn week_view(state: AppState, page_width: Signal<f64>, strip_tx: Signal<f64>) -> impl Piece {")

# Add on_drag back to the inner rows
content = content.replace(""".translation(strip_tx, 0.0),
                ))
            },""",
""".translation(strip_tx, 0.0),
                ))
                .on_drag(pager_drag(strip_tx.clone(), page_width.clone(), state.clone()))
                .grow()
            },""")

content = content.replace(""".translation(strip_tx, 0.0),
                ))
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
}""")

with open('src/pages/diary.rs', 'w') as f:
    f.write(content)
