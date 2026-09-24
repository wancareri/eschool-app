use crate::app::AppState;
use day::prelude::*;

pub fn network_error_sheet(state: AppState) -> impl Piece {
    let show = state.show_network_modal;
    let s_retry = state;
    let s_close = state;

    when(
        move || show.get(),
        move || {
            zstack((
                // Dimmed background that closes the sheet when tapped
                button("")
                    .action(move || show.set(false))
                    .background(Color::rgba(0.0, 0.0, 0.0, 0.50))
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

                    // Icon & Title
                    row((
                        label("📡").font(Font::Title2),
                        label("Нет связи с сервером")
                            .font(Font::Headline)
                            .grow(),
                    ))
                    .spacing(10.0)
                    .padding(Insets { top: 0.0, leading: 24.0, bottom: 8.0, trailing: 24.0 }),

                    // Body explanation
                    label("Не удалось подключиться к серверу. Используются сохранённые данные, они могут быть неактуальны.")
                        .font(Font::Subheadline)
                        .secondary()
                        .padding(Insets { top: 0.0, leading: 24.0, bottom: 24.0, trailing: 24.0 }),

                    // Action buttons
                    column((
                        button("Повторить попытку")
                            .prominent()
                            .action(move || {
                                s_retry.show_network_modal.set(false);
                                if s_retry.is_authenticated.get() {
                                    crate::features::diary::load_all(s_retry);
                                }
                            })
                            .padding(Insets { top: 4.0, leading: 16.0, bottom: 4.0, trailing: 16.0 }),

                        button("Понятно")
                            .action(move || s_close.show_network_modal.set(false))
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
            .any()
        },
    )
}
