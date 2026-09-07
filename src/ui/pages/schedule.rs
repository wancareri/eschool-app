use crate::native;
use crate::res;
use day::prelude::*;

/// Schedule page - shows bell schedule and timetable
pub(crate) fn schedule_page() -> impl Piece {
    column((
        native::header::render(
            res::str::schedule_title().format(),
            "Расписание звонков",
        ),
        spacer(),
        // TODO: bell schedule list
        label("Загрузка расписания звонков...")
            .font(Font::Body),
        spacer(),
    ))
    .spacing(16.0)
    .padding(24.0)
    .grow()
}
