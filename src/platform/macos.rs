use day::prelude::*;

pub fn header(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
    // macOS uses SwiftUI bridge (same as iOS)
    super::ios::header(title, subtitle)
}

pub fn card(title: impl Into<String>, value: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    super::ios::card(title, value, icon)
}

pub fn button(title: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    super::ios::button(title, icon)
}
