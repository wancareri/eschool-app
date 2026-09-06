use day::prelude::*;

pub fn render(title: impl Into<String>, value: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    super::ios::card::render(title, value, icon)
}
