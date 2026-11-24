use crate::app::{AppState, ConnStatus};
use crate::res;
use day::prelude::*;

fn close_sheet(state: AppState, reason: &str) {
    crate::ios::nslog::nslog(&format!("[cover] close_sheet: {reason}"));
    state.show_network_modal.set(None);
}

pub fn network_error_sheet(state: AppState) -> impl Piece {
    let s_retry = state;

    crate::components::modal::modal_window(state.show_network_modal, 360.0, move |_| {
        let status_icon = when(
            move || {
                matches!(
                    state.conn_status.get(),
                    ConnStatus::Offline | ConnStatus::Error
                )
            },
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
                move || crate::components::spinner::render(state, 16.0).any(),
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
            label(move || super::connection_status::title(state.conn_status.get()))
                .font(Font::Headline)
                .grow(Dimension::Both),
        ))
        .spacing(10.0)
        .grow(Dimension::Both)
        .padding(Insets {
            top: 0.0,
            leading: 10.0,
            bottom: 0.0,
            trailing: 10.0,
        });

        let body = label(move || super::connection_status::message(state.conn_status.get()))
            .font(Font::Subheadline)
            .secondary()
            .grow(Dimension::Both)
            .padding(Insets {
                top: 0.0,
                leading: 10.0,
                bottom: 14.0,
                trailing: 10.0,
            });

        column((
            icon_and_title,
            body,
            spacer().grow(Dimension::Height),
            column((when(
                move || {
                    matches!(
                        state.conn_status.get(),
                        ConnStatus::Offline | ConnStatus::Error
                    )
                },
                move || {
                    button(res::str::notice_retry())
                        .prominent()
                        .tweak(|node| crate::components::modal::native_glass_button(node, true))
                        .grow(Dimension::Width)
                        .action(move || {
                            let st = s_retry;
                            close_sheet(st, "retry");
                            if st.is_authenticated.get() {
                                crate::pages::diary::data::load_all(st);
                            }
                        })
                        .padding(Insets {
                            top: 4.0,
                            leading: 16.0,
                            bottom: 4.0,
                            trailing: 16.0,
                        })
                        .any()
                },
            ),))
            .align(HAlign::Center)
            .grow(Dimension::Width)
            .padding(Insets {
                top: 8.0,
                leading: 2.0,
                bottom: 14.0,
                trailing: 2.0,
            }),
        ))
        .spacing(10.0)
        .height(360.0)
        .grow(Dimension::Both)
    })
}
