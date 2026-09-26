use crate::app::{AppState, ConnStatus};
use day::prelude::*;

/// The connection status as a native bottom action sheet (day `Alert::sheet()`).
///
/// The dialog rides the backend's present FIFO — it is never part of the page
/// tree, so it cannot break the page's full-bleed subview chain the way an
/// in-page sheet sibling did (that lift lifted the tab bar and re-laid the
/// status island on every open). UIKit presents it over a still-visible,
/// system-dimmed page: a fullscreen `cover` cannot do that — iOS drops the
/// presenting view once a fullScreen transition completes.
///
/// `day::task` is the sanctioned bridge from the island's sync tap into this
/// future (docs/dialogs.md); the handle is freely discardable.
pub fn present(state: AppState) {
    let (title, body) = match state.conn_status.get() {
        ConnStatus::Offline => (
            "Нет связи с сервером",
            "Не удалось подключиться к серверу. Используются сохранённые данные, они могут быть неактуальны.",
        ),
        ConnStatus::Error => (
            "Ошибка сети",
            "Не удалось подключиться к серверу. Используются сохранённые данные, они могут быть неактуальны.",
        ),
        ConnStatus::Connecting => ("Обновление данных", "Идёт загрузка данных, подождите немного."),
        ConnStatus::Connected | ConnStatus::Idle => {
            ("Соединение в порядке", "Данные обновлены, всё в порядке.")
        }
    };
    let retry = matches!(
        state.conn_status.get(),
        ConnStatus::Offline | ConnStatus::Error
    );

    day::task(async move {
        let mut sheet = Alert::new(title).message(body).sheet();
        if retry {
            sheet = sheet.button("Повторить попытку", true);
        }
        let choice = sheet.button("Понятно", false).present().await;
        if choice == Some(true) && state.is_authenticated.get() {
            crate::features::diary::load_all(state);
        }
    });
}
