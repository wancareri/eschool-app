use day::prelude::*;

pub fn header(title: impl Into<String>, subtitle: impl Into<String>) -> AnyPiece {
    if day_piece_swiftui::support() == Support::Native {
        crate::swiftui::GlassHeaderView(title.into(), subtitle.into())
            .frame(320.0, 80.0)
            .id("glass-header")
            .any()
    } else {
        super::common::header(title, subtitle)
    }
}

pub fn card(title: impl Into<String>, value: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    if day_piece_swiftui::support() == Support::Native {
        crate::swiftui::GlassCardView(title.into(), value.into(), icon.into())
            .frame(140.0, 100.0)
            .id("glass-card")
            .any()
    } else {
        super::common::card(title, value, icon)
    }
}

pub fn button(title: impl Into<String>, icon: impl Into<String>) -> AnyPiece {
    if day_piece_swiftui::support() == Support::Native {
        crate::swiftui::GlassButtonView(title.into(), icon.into())
            .frame(200.0, 48.0)
            .id("glass-button")
            .any()
    } else {
        super::common::button(title, icon)
    }
}
