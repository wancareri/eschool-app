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
pub fn network_error_sheet(state: AppState) -> impl Piece {
    let s_tap = state;
    let s_retry = state;
    let s_ok = state;

    cover(
        state.show_network_modal,
        move |_| {
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
                // the original 24/16/32pt edges.
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
                .any(),
            ))
            .align(Alignment::Bottom)
            .grow()
        },
    )
    .unrouted()
    .background(|_| Color::rgba(0.0, 0.0, 0.0, 0.50))
}
