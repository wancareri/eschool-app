use day::prelude::*;

pub fn render(title: impl Into<String>, value: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    if day_piece_swiftui::support() == Support::Native {
        crate::swiftui::GlassCardView(title.into(), value.into(), icon.into())
            .frame(140.0, 100.0)
            .id("glass-card")
            .any()
    } else {
        super::super::common::card::render(title, value, icon)
    }
}
