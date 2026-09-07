use crate::native;
use crate::res;
use day::prelude::*;

/// Teachers page - shows list of teachers
pub(crate) fn teachers_page() -> impl Piece {
    column((
        native::header::render(
            res::str::teachers_title().format(),
            "Список учителей",
        ),
        spacer(),
        // TODO: teachers list
        label("Загрузка списка учителей...")
            .font(Font::Body),
        spacer(),
    ))
    .spacing(16.0)
    .padding(24.0)
    .grow()
}
