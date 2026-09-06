use day::prelude::*;

pub fn render(title: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    if day_piece_swiftui::support() == Support::Native {
        crate::swiftui::GlassButtonView(title.into(), icon.into())
            .frame(200.0, 48.0)
            .id("glass-button")
            .any()
    } else {
        super::super::common::button::render(title, icon)
    }
}
