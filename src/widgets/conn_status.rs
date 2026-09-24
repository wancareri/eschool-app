/// Telegram-style connection status indicator — small pill in the top-left corner.
use crate::app::{AppState, ConnStatus};
use crate::res;
use crate::shared::colors;
use day::prelude::*;

pub fn render() -> impl Piece {
    let state = AppState::ambient();

    when(
        move || state.conn_status.get() != ConnStatus::Idle,
        move || {
            let status = state.conn_status.get();
            row((
                match status {
                    ConnStatus::Connecting => row((
                        spinner().frame(14.0, 14.0),
                        label("Обновление…").font(Font::Caption2).secondary(),
                    )).spacing(4.0).any(),
                    ConnStatus::Connected => row((
                        vector(res::vectors::status_ok).frame(14.0, 14.0).tint(colors::SUCCESS),
                        label("Подключено").font(Font::Caption2).color(colors::SUCCESS),
                    )).spacing(4.0).any(),
                    ConnStatus::Offline => row((
                        vector(res::vectors::status_offline).frame(14.0, 14.0).tint(colors::WARNING),
                        label("Оффлайн").font(Font::Caption2).color(colors::WARNING),
                    )).spacing(4.0).any(),
                    ConnStatus::Error => row((
                        vector(res::vectors::status_error).frame(14.0, 14.0).tint(colors::ERROR),
                        label("Ошибка сети").font(Font::Caption2).color(colors::ERROR),
                    )).spacing(4.0).any(),
                    ConnStatus::Idle => spacer().frame(0.0, 0.0).any(),
                },
            ))
            .padding(Insets { top: 4.0, leading: 8.0, bottom: 4.0, trailing: 8.0 })
        },
    )
}
