use crate::platform::glass;
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
        glass::glass_header(
            res::str::welcome_title().format(),
            "Liquid Glass UI",
        ),
        label(res::str::welcome_body())
            .markdown()
            .align(TextAlign::Center)
            .max_width(440.0)
            .id("welcome-body"),
        row((
            glass::glass_card("Items", "12", "doc.text"),
            glass::glass_card("Done", "8", "checkmark.circle"),
            glass::glass_card("Pending", "4", "clock"),
        ))
        .spacing(12.0),
        spacer(),
    ))
    .spacing(20.0)
    .align(HAlign::Center)
    .grow()
    .padding(24.0)
}
