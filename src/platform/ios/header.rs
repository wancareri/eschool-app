use day::prelude::*;

pub fn render(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
    if day_piece_swiftui::support() == Support::Native {
        crate::swiftui::GlassHeaderView(title.into(), subtitle.into())
            .frame(320.0, 80.0)
            .id("glass-header")
            .any()
    } else {
        super::super::common::header::render(title, subtitle)
    }
}
