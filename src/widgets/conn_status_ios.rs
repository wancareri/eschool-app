//! Crossfade the native bar's changed pixels before Day replaces its items.
//! Keep UIKit's own buttons, glass, accessibility and action target intact.
use crate::app::ConnStatus;
use objc2::MainThreadMarker;
use objc2_foundation::NSString;
use objc2_quartz_core::{
    CAMediaTiming, CAMediaTimingFunction, CATransition, kCAMediaTimingFunctionEaseInEaseOut,
};
use objc2_ui_kit::{NSObjectUIAccessibility, UIApplication, UINavigationBar, UIView};

fn transition_bars(view: &UIView, status: ConnStatus, mtm: MainThreadMarker) {
    if view.isHidden() || view.alpha() == 0.0 {
        return;
    }
    if let Some(bar) = view.downcast_ref::<UINavigationBar>() {
        let old_status = bar
            .topItem()
            .and_then(|item| item.pinnedTrailingGroup())
            .map(|group| group.barButtonItems())
            .into_iter()
            .flatten()
            .find_map(|item| {
                let label = item.accessibilityLabel(mtm)?.to_string();
                [
                    ConnStatus::Connecting,
                    ConnStatus::Offline,
                    ConnStatus::Error,
                    ConnStatus::Connected,
                ]
                .into_iter()
                .find(|s| super::title(*s) == label)
            });
        if old_status.is_some_and(|old| super::icon_group(old) != super::icon_group(status)) {
            // The navbar stays in place: only the glyph's pixels change. A
            // transition on this stable layer survives Day rebuilding all its
            // UIBarButtonItems, unlike an animation on the outgoing item view.
            let fade = CATransition::new();
            fade.setDuration(0.22);
            unsafe {
                fade.setTimingFunction(Some(&CAMediaTimingFunction::functionWithName(
                    kCAMediaTimingFunctionEaseInEaseOut,
                )));
            }
            bar.layer().addAnimation_forKey(
                &fade,
                Some(&NSString::from_str("eschool.connection-status")),
            );
        }
        return;
    }
    for child in view.subviews() {
        transition_bars(&child, status, mtm);
    }
}

#[allow(deprecated)] // Single-scene app, matching the existing window helpers.
pub(super) fn prepare_transition(status: ConnStatus) {
    if day::testing::fast_animations() {
        return;
    }
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = UIApplication::sharedApplication(mtm);
    if let Some(window) = app.keyWindow() {
        transition_bars(&window, status, mtm);
    }
}
