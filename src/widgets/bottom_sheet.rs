use crate::app::{AppState, ConnStatus};
use crate::res;
use day::prelude::*;

/// Close is just the signal flip: the cover piece runs the native dismissal
/// itself and keeps its content mounted until the backend reports it hidden.
fn close_sheet(state: AppState) {
    state.show_network_modal.set(None);
}

/// The network status modal as a fullscreen cover. day never attaches a cover's
/// view to the page it sits in — it lives in its own modal VC and its node
/// measures 0×0 in the tree — so presenting it leaves the page's subview walk
/// untouched: the full-bleed chain holds and the tab bar never moves. The cover
/// is presented OverFullScreen (patched in the wancareri/day fork: upstream's
/// FullScreen drops the presenting view once the transition lands, which showed
/// black behind this dim instead of the live page), so the page stays visible
/// beneath the dim; the fork presents with a fade-and-slide-in and dismisses
/// with a plain fade, so the dim never sweeps across the page like a window.
///
/// The sheet is draggable: pulling UP resists (a rubber band with ~70pt of
/// travel), pulling DOWN follows the finger and, past the threshold, closes it.
pub fn network_error_sheet(state: AppState) -> impl Piece {
    let s_tap = state;
    let s_retry = state;
    let s_ok = state;

    cover(
        state.show_network_modal,
        move |_| {
            // Sheet drag. Vertical only (horizontal travel is simply ignored).
            // Both directions start from the finger; only the release animates.
            let drag_y = Signal::new(0.0);
            let s_drag = state;
            let sheet_drag = move |d: Drag| match d.phase {
                DragPhase::Began => drag_y.set(0.0),
                DragPhase::Changed => {
                    let dy = d.translation.y;
                    let off = if dy > 0.0 {
                        // Down: follows the finger 1:1 — this is the dismiss direction.
                        dy
                    } else {
                        // Up: rubber band — asymptotes at MAX_UP instead of refusing,
                        // so the sheet yields a little and then visibly resists.
                        const MAX_UP: f64 = 70.0;
                        -MAX_UP * (1.0 - 1.0 / (1.0 + (-dy) / MAX_UP))
                    };
                    drag_y.set(off);
                }
                DragPhase::Ended => {
                    if drag_y.get() > 110.0 {
                        // Past the threshold: close with the cover's fade dismissal.
                        // The offset stays put through the fade; reset it once the
                        // dismissal (0.24s) has finished so the next open starts at 0.
                        let reset = drag_y.setter();
                        close_sheet(s_drag);
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(320));
                            day::reactive::on_main(move || reset.set(0.0));
                        });
                    } else {
                        with_animation(AnimSpec::ease_out(240), move || drag_y.set(0.0));
                    }
                }
            };

            // The status glyph — the same lucide vectors the island carries
            // (wifi-off / alert-circle / check), the accent spinner while connecting.
            // One shared 22pt slot that swaps on every status change.
            let status_icon = when(
                move || matches!(state.conn_status.get(), ConnStatus::Offline | ConnStatus::Error),
                move || {
                    if state.conn_status.get() == ConnStatus::Offline {
                        vector(res::vectors::status_offline)
                            .frame(22.0, 22.0)
                            .tint(Color::hex(0x8E8E93))
                            .any()
                    } else {
                        vector(res::vectors::status_error)
                            .frame(22.0, 22.0)
                            .tint(Color::hex(0xEF4444))
                            .any()
                    }
                },
            )
            .otherwise(move || {
                when(
                    move || state.conn_status.get() == ConnStatus::Connecting,
                    move || super::spinner::render(state, 22.0).any(),
                )
                .otherwise(move || {
                    vector(res::vectors::check)
                        .frame(22.0, 22.0)
                        .tint(move || Color::hex(state.accent_color.get()))
                        .any()
                })
            });

            let icon_and_title = row((
                status_icon,
                label(move || match state.conn_status.get() {
                    ConnStatus::Offline => "Нет связи с сервером",
                    ConnStatus::Error => "Ошибка сети",
                    ConnStatus::Connecting => "Обновление данных",
                    ConnStatus::Connected | ConnStatus::Idle => "Обновлено",
                })
                .font(Font::Headline)
                .grow(),
            ))
            .spacing(10.0)
            .grow()
            .padding(Insets {
                top: 0.0,
                leading: 10.0,
                bottom: 0.0,
                trailing: 10.0,
            });

            let body = label(move || match state.conn_status.get() {
                ConnStatus::Offline | ConnStatus::Error => {
                    "Не удалось подключиться к серверу. Используются сохранённые данные, они могут быть неактуальны."
                }
                ConnStatus::Connecting => "Идёт загрузка данных, подождите немного.",
                ConnStatus::Connected | ConnStatus::Idle => "Данные обновлены, всё в порядке.",
            })
            .font(Font::Subheadline)
            .secondary()
            .grow()
            .padding(Insets {
                top: 0.0,
                leading: 10.0,
                bottom: 14.0,
                trailing: 10.0,
            });

            zstack((
                // The dim is the cover's own background (edge-to-edge, over the
                // status bar and the home indicator); this layer is transparent
                // and only catches taps outside the card. The card above it is a
                // later zstack child, so it wins the hit-test over its own area.
                button("")
                    .action(move || close_sheet(s_tap))
                    .grow()
                    .any(),

                // Bottom sheet card: day's semantic section surface — the platform's
                // theme-adaptive grouped-card material (secondary system grouped
                // background on iOS), so it tracks light/dark mode with no app palette.
                // The paddings below compose with the section's own 14pt inset to keep
                // the original 24/16/32pt edges. The translation is the sheet drag —
                // no node `.animation`, so it tracks the finger exactly; only the
                // release snaps back (animated inside the drag handler).
                section((
                    // Drag handle pill at the top
                    row((
                        spacer().grow(),
                        column(()).frame(36.0, 4.0).background(Color::rgba(0.7, 0.7, 0.7, 0.6)).corner_radius(2.0),
                        spacer().grow(),
                    ))
                    .grow()
                    .padding(Insets {
                        top: 0.0,
                        leading: 0.0,
                        bottom: 6.0,
                        trailing: 0.0,
                    }),

                    icon_and_title,
                    body,

                    // Action buttons
                    column((
                        when(
                            move || matches!(
                                state.conn_status.get(),
                                ConnStatus::Offline | ConnStatus::Error
                            ),
                            move || button("Повторить попытку")
                                .prominent()
                                .action(move || {
                                    let st = s_retry;
                                    close_sheet(st);
                                    if st.is_authenticated.get() {
                                        crate::features::diary::load_all(st);
                                    }
                                })
                                .padding(Insets { top: 4.0, leading: 16.0, bottom: 4.0, trailing: 16.0 })
                                .any(),
                        ),

                        button("Понятно")
                            .action(move || close_sheet(s_ok))
                            .padding(Insets { top: 4.0, leading: 16.0, bottom: 4.0, trailing: 16.0 }),
                    ))
                    .spacing(8.0)
                    .align(HAlign::Center)
                    .grow()
                    .padding(Insets {
                        top: 0.0,
                        leading: 2.0,
                        bottom: 18.0,
                        trailing: 2.0,
                    }),
                ))
                .translation(0.0, move || drag_y.get())
                .any(),
            ))
            .align(Alignment::Bottom)
            .grow()
            // The recognizer sits on the whole overlay — a pull starting on the dim
            // drags too (the dim itself doesn't move: only the card translates).
            .on_drag(sheet_drag)
        },
    )
    .unrouted()
    .background(|_| Color::rgba(0.0, 0.0, 0.0, 0.50))
}
