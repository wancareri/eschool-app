use day::prelude::*;

pub fn render(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
    // TODO: GTK4/libadwaita header
    super::super::common::header::render(title, subtitle)
}
