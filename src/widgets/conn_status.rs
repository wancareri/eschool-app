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
            let (bg_color, content) = match status {
                ConnStatus::Connecting => (
                    Color::rgba(0.2, 0.25, 0.35, 0.14),
                    row((
                        spinner().frame(13.0, 13.0),
                        label("Обновление…").font(Font::Caption2).secondary(),
                    )).spacing(5.0).any(),
                ),
                ConnStatus::Connected => (
                    Color::rgba(0.06, 0.72, 0.50, 0.16),
                    row((
                        vector(res::vectors::status_ok).frame(13.0, 13.0).tint(colors::SUCCESS),
                        label("Подключено").font(Font::Caption2).color(colors::SUCCESS),
                    )).spacing(5.0).any(),
                ),
                ConnStatus::Offline => (
                    Color::rgba(0.96, 0.62, 0.04, 0.16),
                    row((
                        vector(res::vectors::status_offline).frame(13.0, 13.0).tint(colors::WARNING),
                        label("Оффлайн").font(Font::Caption2).color(colors::WARNING),
                    )).spacing(5.0).any(),
                ),
                ConnStatus::Error => (
                    Color::rgba(0.94, 0.27, 0.27, 0.16),
                    row((
                        vector(res::vectors::status_error).frame(13.0, 13.0).tint(colors::ERROR),
                        label("Ошибка сети").font(Font::Caption2).color(colors::ERROR),
                    )).spacing(5.0).any(),
                ),
                ConnStatus::Idle => (
                    Color::CLEAR,
                    spacer().frame(0.0, 0.0).any(),
                ),
            };

            content
                .padding(Insets { top: 5.0, leading: 10.0, bottom: 5.0, trailing: 10.0 })
                .background(bg_color)
                .corner_radius(12.0)
        },
    )
}
