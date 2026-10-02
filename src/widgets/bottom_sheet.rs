use crate::app::{AppState, ConnStatus};
use crate::res;
use day::prelude::*;

/// Close is just the signal flip: the cover piece runs the native dismissal
/// itself and keeps its content mounted until the backend reports it hidden.
fn close_sheet(state: AppState, reason: &str) {
    crate::shared::nslog::nslog(&format!("[cover] close_sheet: {reason}"));
    state.show_network_modal.set(None);
}

/// The network status modal: the reusable [`modal_window`] shell with this app's
/// card content in it — status glyph, headline, body and the two actions. The
/// window owns the dim, the drag, the handle and the animation (both directions
/// are the same motion mirrored in time); everything inside the card lives here.
pub fn network_error_sheet(state: AppState) -> impl Piece {
    let s_retry = state;
    let s_ok = state;

    crate::widgets::modal::modal_window(state.show_network_modal, move |_| {
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

        // The window's section lays its children out with 10pt gaps; this column
        // stands in the card body, so it keeps the same spacing to match what the
        // section used to space out directly (icon row, body, actions).
        column((
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
                            close_sheet(st, "retry");
                            if st.is_authenticated.get() {
                                crate::features::diary::load_all(st);
                            }
                        })
                        .padding(Insets { top: 4.0, leading: 16.0, bottom: 4.0, trailing: 16.0 })
                        .any(),
                ),

                button("Понятно")
                    .action(move || close_sheet(s_ok, "ok-button"))
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
        .spacing(10.0)
        .grow()
    })
}
