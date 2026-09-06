//! SwiftUI integration for Liquid Glass components.
//!
//! This module exposes native SwiftUI views with iOS 26 Liquid Glass effects.
//! On unsupported platforms, it provides fallback native controls.

use day::prelude::*;

/// A Liquid Glass header with title and subtitle.
pub fn glass_header(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
    #[cfg(any(feature = "appkit", feature = "uikit"))]
    if day_piece_swiftui::support() == Support::Native {
        return crate::swiftui::GlassHeaderView(title.into(), subtitle.into())
            .frame(320.0, 80.0)
            .id("glass-header")
            .any();
    }

    // Fallback for non-Apple platforms
    let title_str = title.into();
    let subtitle_str = subtitle.into();
    column((
        label(title_str.clone())
            .font(Font::LargeTitle),
        label(subtitle_str.clone())
            .font(Font::Subheadline),
    ))
    .spacing(4.0)
    .padding(16.0)
    .any()
}

/// A Liquid Glass card with icon, value, and title.
pub fn glass_card(title: impl Into<String>, value: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    #[cfg(any(feature = "appkit", feature = "uikit"))]
    if day_piece_swiftui::support() == Support::Native {
        return crate::swiftui::GlassCardView(
            title.into(),
            value.into(),
            icon.into(),
        )
        .frame(140.0, 100.0)
        .id("glass-card")
        .any();
    }

    // Fallback for non-Apple platforms
    let title_str = title.into();
    let value_str = value.into();
    column((
        label(value_str.clone())
            .font(Font::Title),
        label(title_str.clone())
            .font(Font::Caption),
    ))
    .spacing(4.0)
    .padding(12.0)
    .any()
}

/// A Liquid Glass button with icon and title (display only — no action via SwiftUI bridge).
pub fn glass_button(title: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    #[cfg(any(feature = "appkit", feature = "uikit"))]
    if day_piece_swiftui::support() == Support::Native {
        return crate::swiftui::GlassButtonView(title.into(), icon.into())
            .frame(200.0, 48.0)
            .id("glass-button")
            .any();
    }

    // Fallback for non-Apple platforms
    let title_str = title.into();
    button(title_str).any()
}
