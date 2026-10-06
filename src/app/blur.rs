// SPDX-License-Identifier: MPL-2.0
//! Helper for iOS glass blur effect.
//!
//! The original implementation lived in `src/widgets/conn_status.rs`. It is extracted here
//! so that both the navigation bar and the connection‑status capsule can reuse the same blur
//! logic without duplicating code.

use day::prelude::*;
#[cfg(target_os = "ios")]
use day_uikit::UiKitExt;
#[cfg(target_os = "ios")]
use objc2::MainThreadOnly;
#[cfg(target_os = "ios")]
use objc2::rc::Retained;
#[cfg(target_os = "ios")]
use objc2_quartz_core::CALayer;
#[cfg(target_os = "ios")]
use objc2_ui_kit::{
    UIVisualEffectView, UIBlurEffect, UIBlurEffectStyle, UIViewAutoresizing,
};

/// Apply a thin‑material glass blur on iOS.
///
/// The function is a no‑op on non‑iOS targets.
#[cfg(target_os = "ios")]
pub fn apply_glass_blur(piece: impl Decorate) -> impl Piece {
    piece.uikit(|view, _class, mtm| {
        let tag: objc2::ffi::NSInteger = 9991;
        if view.viewWithTag(tag).is_some() {
            return;
        }
        let blur = UIBlurEffect::effectWithStyle(
            UIBlurEffectStyle::SystemUltraThinMaterial,
            mtm,
        );
        let effect_view = UIVisualEffectView::initWithEffect(
            UIVisualEffectView::alloc(mtm),
            Some(&blur),
        );
        effect_view.setTag(tag);
        effect_view.setFrame(view.bounds());
        effect_view.setAutoresizingMask(UIViewAutoresizing(
            UIViewAutoresizing::FlexibleWidth.0 | UIViewAutoresizing::FlexibleHeight.0,
        ));
        effect_view.setUserInteractionEnabled(false);
        let view_layer: Retained<CALayer> = view.layer();
        view_layer.setCornerRadius(12.5);
        view_layer.setMasksToBounds(true);
        view.insertSubview_atIndex(&effect_view, 0);
        view.setClipsToBounds(true);
    })
}

/// On non‑iOS platforms the blur is a simple passthrough.
#[cfg(not(target_os = "ios"))]
pub fn apply_glass_blur(piece: impl Decorate) -> impl Piece {
    piece
}
/// Extension trait to allow method-like usage of `apply_glass_blur`.
pub trait ApplyGlassBlur {
    fn apply_glass_blur(self) -> impl Piece;
}

impl<P: Decorate> ApplyGlassBlur for P {
    fn apply_glass_blur(self) -> impl Piece {
        apply_glass_blur(self)
    }
}
