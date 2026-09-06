use day::prelude::*;

pub fn render(title: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    // TODO: GTK4/libadwaita button
    super::super::common::button::render(title, icon)
}
