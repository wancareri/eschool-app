use crate::native;
use crate::res;
use day::prelude::*;

/// Diary page - shows lessons for the current week
pub(crate) fn diary_page() -> impl Piece {
    column((
        native::header::render(
            res::str::diary_title().format(),
            "Расписание уроков",
        ),
        spacer(),
        // TODO: lessons list
        label("Загрузка расписания...")
            .font(Font::Body),
        spacer(),
    ))
    .spacing(16.0)
    .padding(24.0)
    .grow()
}
