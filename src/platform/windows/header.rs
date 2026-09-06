use day::prelude::*;

pub fn render(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
    // TODO: WinUI 3 glass effect
    super::super::common::header::render(title, subtitle)
}
