use day::prelude::*;

pub fn render(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
    // TODO: GTK4/libadwaita header (same as Linux)
    super::super::linux::header::render(title, subtitle)
}
