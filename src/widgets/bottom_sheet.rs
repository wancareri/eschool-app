use crate::app::{AppState, ConnStatus};
use day::prelude::*;

/// Close is just the signal flip: the cover piece runs the native slide-down
/// itself and keeps its content mounted until the backend reports it hidden.
fn close_sheet(state: AppState) {
    state.show_network_modal.set(None);
}

/// The network status modal as a fullscreen cover. day never attaches a cover's
/// view to the page it sits in — it lives in its own modal VC and its node
/// measures 0×0 in the tree — so presenting it leaves the page's subview walk
/// untouched: the full-bleed chain holds and the tab bar no longer lifts the way
/// a root-level sheet sibling made it do on every open. UIKit owns the
/// slide-up/slide-down transition.
pub fn network_error_sheet(state: AppState) -> impl Piece {
    let s_tap = state;
    let s_retry = state;
    let s_ok = state;

    cover(
        state.show_network_modal,
        move |_| {
            let icon_and_title = row((
                label(move || match state.conn_status.get() {
                    ConnStatus::Offline | ConnStatus::Error => "📡",
                    ConnStatus::Connecting => "🔄",
                    ConnStatus::Connected | ConnStatus::Idle => "✅",
                })
                .font(Font::Title2),
                label(move || match state.conn_status.get() {
                    ConnStatus::Offline => "Нет связи с сервером",
                    ConnStatus::Error => "Ошибка сети",
                    ConnStatus::Connecting => "Обновление данных",
                    ConnStatus::Connected | ConnStatus::Idle => "Соединение в порядке",
                })
                .font(Font::Headline)
                .grow(),
            ))
            .spacing(10.0)
            .padding(Insets { top: 0.0, leading: 24.0, bottom: 8.0, trailing: 24.0 });

            let body = label(move || match state.conn_status.get() {
                ConnStatus::Offline | ConnStatus::Error => {
                    "Не удалось подключиться к серверу. Используются сохранённые данные, они могут быть неактуальны."
                }
                ConnStatus::Connecting => "Идёт загрузка данных, подождите немного.",
                ConnStatus::Connected | ConnStatus::Idle => "Данные обновлены, всё в порядке.",
            })
            .font(Font::Subheadline)
            .secondary()
            .padding(Insets { top: 0.0, leading: 24.0, bottom: 24.0, trailing: 24.0 });

            zstack((
                // The dim is the cover's own background (edge-to-edge, over the
                // status bar and the home indicator); this layer is transparent
                // and only catches taps outside the card. The card above it is a
                // later zstack child, so it wins the hit-test over its own area.
                button("")
                    .action(move || close_sheet(s_tap))
                    .grow()
                    .any(),

                // Bottom sheet card
                column((
                    // Drag handle pill at the top
                    row((
                        spacer().grow(),
                        column(()).frame(36.0, 4.0).background(Color::rgba(0.7, 0.7, 0.7, 0.6)).corner_radius(2.0),
                        spacer().grow(),
                    ))
                    .padding(Insets { top: 10.0, leading: 0.0, bottom: 16.0, trailing: 0.0 }),

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
                    .padding(Insets { top: 0.0, leading: 24.0, bottom: 32.0, trailing: 24.0 }),
                ))
                .align(HAlign::Center)
                .background(Color::rgba(0.12, 0.12, 0.14, 0.98))
                .corner_radius(20.0)
                .any(),
            ))
            .align(Alignment::Bottom)
            .grow()
        },
    )
    .unrouted()
    .background(|_| Color::rgba(0.0, 0.0, 0.0, 0.50))
}
