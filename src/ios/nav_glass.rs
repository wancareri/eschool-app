//! Transparent UIKit navigation glass that fades into the page content.

use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_core_graphics::CGColor;
use objc2_foundation::NSArray;
use objc2_quartz_core::{CAAutoresizingMask, CAGradientLayer};
use objc2_ui_kit::{
    UIBlurEffect, UIBlurEffectStyle, UINavigationBar, UIViewAutoresizing, UIVisualEffect,
    UIVisualEffectView,
};
use std::ptr::NonNull;

const NAV_GLASS_TAG: isize = 8_417_263;

/// Put a masked system blur between the page and navigation controls.
pub fn apply(bar: &UINavigationBar, mtm: MainThreadMarker) {
    let Some(parent) = bar.superview() else {
        return;
    };
    let subviews = parent.subviews();
    let Some(bar_index) = subviews
        .iter()
        .position(|view| {
            std::ptr::eq(
                view.as_ref() as *const objc2_ui_kit::UIView,
                bar as *const UINavigationBar as *const objc2_ui_kit::UIView,
            )
        })
    else {
        return;
    };

    let view = parent
        .viewWithTag(NAV_GLASS_TAG)
        .and_then(|view| view.downcast::<UIVisualEffectView>().ok())
        .unwrap_or_else(|| {
            let effect: objc2::rc::Retained<UIVisualEffect> = UIBlurEffect::effectWithStyle(
                UIBlurEffectStyle::SystemUltraThinMaterial,
                mtm,
            )
            .into_super();
            let view =
                UIVisualEffectView::initWithEffect(UIVisualEffectView::alloc(mtm), Some(&effect));
            view.setTag(NAV_GLASS_TAG);
            view.setUserInteractionEnabled(false);
            view.setAutoresizingMask(UIViewAutoresizing::FlexibleWidth);
            parent.insertSubview_atIndex(&view, bar_index as isize);
            view
        });

    // Extend the blur below the bar; the alpha mask reaches zero before the
    // bottom of the view, so UIKit never exposes a rectangular material edge.
    let bar_frame = bar.frame();
    let effect_height = bar_frame.size.height + 150.0;
    view.setFrame(CGRect::new(
        bar_frame.origin,
        CGSize::new(parent.bounds().size.width, effect_height),
    ));
    let current_subviews = parent.subviews();
    if let Some(index) = current_subviews
        .iter()
        .position(|subview| {
            std::ptr::eq(
                subview.as_ref() as *const objc2_ui_kit::UIView,
                &*view as *const UIVisualEffectView as *const objc2_ui_kit::UIView,
            )
        })
        && index > bar_index
    {
        parent.insertSubview_atIndex(&view, bar_index as isize);
    }

    let gradient = CAGradientLayer::new();
    gradient.setFrame(view.bounds());
    gradient.setAutoresizingMask(
        CAAutoresizingMask::LayerWidthSizable | CAAutoresizingMask::LayerHeightSizable,
    );
    gradient.setStartPoint(CGPoint::new(0.5, 0.0));
    gradient.setEndPoint(CGPoint::new(0.5, 1.0));

    let top = unsafe { objc2_ui_kit::UIColor::colorWithWhite_alpha(1.0, 0.78).CGColor() };
    let middle = unsafe { objc2_ui_kit::UIColor::colorWithWhite_alpha(1.0, 0.44).CGColor() };
    let bottom = unsafe { objc2_ui_kit::UIColor::colorWithWhite_alpha(1.0, 0.0).CGColor() };
    let mut color_objects = [
        unsafe {
            NonNull::new_unchecked((&*top as *const CGColor).cast_mut().cast::<AnyObject>())
        },
        unsafe {
            NonNull::new_unchecked((&*middle as *const CGColor).cast_mut().cast::<AnyObject>())
        },
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
