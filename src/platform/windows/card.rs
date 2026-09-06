use day::prelude::*;

pub fn render(title: impl Into<String>, value: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    // TODO: WinUI 3 card
    super::super::common::card::render(title, value, icon)
}
