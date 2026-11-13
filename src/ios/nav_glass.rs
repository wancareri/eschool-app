//! Edge-to-edge navigation-bar glass with a vertical fade into page content.

use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_core_foundation::CGPoint;
use objc2_core_graphics::CGColor;
use objc2_foundation::NSArray;
use objc2_quartz_core::{CAAutoresizingMask, CAGradientLayer};
use objc2_ui_kit::{
    UIColor, UINavigationBar, UIViewAutoresizing, UIVisualEffect, UIVisualEffectView,
};
use std::ptr::NonNull;

const NAV_GLASS_TAG: isize = 8_417_263;

/// Install or refresh the bar's gradient glass backdrop without covering its controls.
pub fn apply(bar: &UINavigationBar, effect: &UIVisualEffect, mtm: MainThreadMarker) {
    let view = bar
        .viewWithTag(NAV_GLASS_TAG)
        .and_then(|view| view.downcast::<UIVisualEffectView>().ok())
        .unwrap_or_else(|| {
            let view =
                UIVisualEffectView::initWithEffect(UIVisualEffectView::alloc(mtm), Some(effect));
            view.setTag(NAV_GLASS_TAG);
            view.setUserInteractionEnabled(false);
            view.setAutoresizingMask(
                UIViewAutoresizing::FlexibleWidth | UIViewAutoresizing::FlexibleHeight,
            );
            bar.insertSubview_atIndex(&view, 0);
            view
        });

    let bounds = bar.bounds();
    view.setFrame(bounds);
    bar.sendSubviewToBack(&view);

    let gradient = CAGradientLayer::new();
    gradient.setFrame(bounds);
    gradient.setAutoresizingMask(
        CAAutoresizingMask::LayerWidthSizable | CAAutoresizingMask::LayerHeightSizable,
    );
    gradient.setStartPoint(CGPoint::new(0.5, 0.0));
    gradient.setEndPoint(CGPoint::new(0.5, 1.0));

    // CAGradientLayer consumes CGColor objects in NSArray form.
    let top = unsafe { UIColor::colorWithWhite_alpha(1.0, 0.48).CGColor() };
    let bottom = unsafe { UIColor::colorWithWhite_alpha(1.0, 0.0).CGColor() };
    let mut color_objects = [
        unsafe { NonNull::new_unchecked((&*top as *const CGColor).cast_mut().cast::<AnyObject>()) },
        unsafe {
            NonNull::new_unchecked((&*bottom as *const CGColor).cast_mut().cast::<AnyObject>())
        },
    ];
    let color_count = color_objects.len();
    let colors = unsafe {
        NSArray::<AnyObject>::arrayWithObjects_count(
            NonNull::new_unchecked(color_objects.as_mut_ptr()),
            color_count,
        )
    };
    unsafe { gradient.setColors(Some(&colors)) };
    unsafe { view.layer().setMask(Some(&gradient)) };
}
