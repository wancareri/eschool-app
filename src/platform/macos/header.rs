use day::prelude::*;

pub fn render(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
    // macOS uses SwiftUI bridge (same as iOS)
    super::ios::header::render(title, subtitle)
}
