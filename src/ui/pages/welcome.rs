use crate::native;
use crate::res;
use day::prelude::*;

/// The opening screen: vector art, a greeting, and Liquid Glass cards.
pub(crate) fn welcome_page() -> impl Piece {
    column((
        spacer(),
        vector(res::vectors::app_mark)
            .frame(132.0, 132.0)
            .corner_radius(30.0)
            .id("welcome-mark"),
        native::header::render(
            res::str::welcome_title().format(),
            "Liquid Glass UI",
        ),
        label(res::str::welcome_body())
            .markdown()
            .align(TextAlign::Center)
            .max_width(440.0)
            .id("welcome-body"),
        row((
            native::card::render("Items", "12", "doc.text"),
            native::card::render("Done", "8", "checkmark.circle"),
            native::card::render("Pending", "4", "clock"),
        ))
        .spacing(12.0),
        spacer(),
    ))
    .spacing(20.0)
    .align(HAlign::Center)
    .grow()
    .padding(24.0)
}
