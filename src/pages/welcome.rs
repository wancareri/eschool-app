use day::prelude::*;
use crate::swiftui_glass;

/// The opening screen: vector art, a greeting, and markdown prose whose emphasis and link live
/// in the translation (https://daybrite.dev/docs/vectors, https://daybrite.dev/docs/markdown).
pub(crate) fn welcome_page() -> impl Piece {
    column((
        spacer(),
        vector(crate::res::vectors::app_mark)
            .frame(132.0, 132.0)
            .corner_radius(30.0)
            .id("welcome-mark"),
        // Liquid Glass header (native on iOS 26+, fallback on other platforms)
        swiftui_glass::glass_header(
            crate::res::str::welcome_title().format(),
            "Liquid Glass UI",
        ),
        label(crate::res::str::welcome_body())
            .markdown()
            .align(TextAlign::Center)
            .max_width(440.0)
            .id("welcome-body"),
        // Glass cards row
        row((
            swiftui_glass::glass_card("Items", "12", "doc.text"),
            swiftui_glass::glass_card("Done", "8", "checkmark.circle"),
            swiftui_glass::glass_card("Pending", "4", "clock"),
        ))
        .spacing(12.0),
        spacer(),
    ))
    .spacing(20.0)
    .align(HAlign::Center)
    // Fill the pane first, so the centering has room to mean anything.
    .grow()
    .padding(24.0)
}
