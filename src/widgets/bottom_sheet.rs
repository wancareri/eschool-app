use crate::app::{AppState, ConnStatus};
use day::prelude::*;

/// Slide the sheet below the screen edge, then flip `show_network_modal` off —
/// the window unmounts on that signal, so the flag can only change after the
/// exit animation has had time to play.
fn close_sheet(state: AppState, sheet_y: Signal<f64>, dim_op: Signal<f64>) {
    let setter_show = state.show_network_modal.setter();
    with_animation(AnimSpec::ease_out(250), move || {
        sheet_y.set(crate::pages::diary::get_screen_height());
        dim_op.set(0.0);
    });
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(270));
        day::reactive::on_main(move || {
            setter_show.set(false);
        });
    });
}

pub fn network_error_sheet(state: AppState) -> impl Piece {
    // The widget itself is mounted by the window only while the modal is open,
    // so these per-mount signals start every opening from the same state.
    let h = crate::pages::diary::get_screen_height();
    let sheet_y = Signal::new(h);
    let dim_op = Signal::new(0.0);
    let s_retry = state;
    let s_close = state;
    let s_close2 = state;
    let y_close = sheet_y;
    let d_close = dim_op;
    let y_close2 = sheet_y;
    let d_close2 = dim_op;

    // TG-style slide-up: mount sits at y = screen height; animate in once the
    // first frame is committed (an animation started during the mount drain
    // would have no presented frame to interpolate from yet). Setters cross
    // the thread boundary — Signals are not Send.
    let setter_y = sheet_y.setter();
    let setter_d = dim_op.setter();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(30));
        day::reactive::on_main(move || {
            with_animation(AnimSpec::ease_out(300), move || {
                setter_y.set(0.0);
                setter_d.set(1.0);
            });
        });
    });

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
        // Dimmed background that closes the sheet when tapped
        button("")
            .action(move || close_sheet(s_close, y_close, d_close))
            .background(Color::rgba(0.0, 0.0, 0.0, 0.50))
            .opacity(move || dim_op.get())
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
                            close_sheet(st, y_close2, d_close2);
                            if st.is_authenticated.get() {
                                crate::features::diary::load_all(st);
                            }
                        })
                        .padding(Insets { top: 4.0, leading: 16.0, bottom: 4.0, trailing: 16.0 })
                        .any(),
                ),

                button("Понятно")
                    .action(move || close_sheet(s_close2, sheet_y, dim_op))
                    .padding(Insets { top: 4.0, leading: 16.0, bottom: 4.0, trailing: 16.0 }),
            ))
            .spacing(8.0)
            .align(HAlign::Center)
            .padding(Insets { top: 0.0, leading: 24.0, bottom: 32.0, trailing: 24.0 }),
        ))
        .align(HAlign::Center)
        .background(Color::rgba(0.12, 0.12, 0.14, 0.98))
        .corner_radius(20.0)
        .translation(0.0, move || sheet_y.get())
        .any(),
    ))
    .align(Alignment::Bottom)
    .grow()
    .any()
}
