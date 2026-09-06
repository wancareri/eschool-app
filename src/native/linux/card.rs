use day::prelude::*;

pub fn render(title: impl Into<String>, value: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    // TODO: GTK4/libadwaita card
    super::super::common::card::render(title, value, icon)
}
