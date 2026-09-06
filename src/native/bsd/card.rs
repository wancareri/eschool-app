use day::prelude::*;

pub fn render(title: impl Into<String>, value: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    super::super::linux::card::render(title, value, icon)
}
