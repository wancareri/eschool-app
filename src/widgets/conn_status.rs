//! Native liquid glass circular connection status button for iOS.
//! A frosted glass circle with dynamic status icon (spinner / wifi / wifi-off / error).
//! Tapping it opens the detailed network status sheet.

use crate::app::{AppState, ConnStatus};
use crate::res;
use day::prelude::*;

#[cfg(target_os = "ios")]
pub fn safe_area_top() -> f64 {
    use objc2::MainThreadMarker;
    use objc2_ui_kit::UIApplication;
    if let Some(mtm) = MainThreadMarker::new() {
        #[allow(deprecated)]
        if let Some(window) = UIApplication::sharedApplication(mtm).keyWindow() {
            let top = window.safeAreaInsets().top as f64;
            if top > 20.0 {
                return top;
            }
        }
    }
    54.0
}

#[cfg(not(target_os = "ios"))]
pub fn safe_area_top() -> f64 {
    54.0
}

/// The floating liquid glass status button overlay on the page:
/// tagged `ANNOTATION_TAG` so ios-uikit's bleed walk skips it — the page keeps its
/// single-scroll chain and full-bleed frame under the bars.
pub fn page_overlay() -> impl Piece {
    let initial_top = safe_area_top() + 6.0;
    let top_sig = Signal::new(initial_top);
    #[cfg(target_os = "ios")]
    let top_setter = top_sig.setter();
    #[cfg(target_os = "ios")]
    let top_setter_delayed = top_sig.setter();

    let btn = render();
    #[cfg(target_os = "ios")]
    let btn = {
        use day_uikit::UiKitExt;
        btn.uikit(move |view, _class, mtm| {
            // Keep full bleed layout
            view.setTag(day_uikit::ANNOTATION_TAG);
            if let Some(parent) = unsafe { view.superview() } {
                parent.setTag(day_uikit::ANNOTATION_TAG);
            }

            // Sync with actual window safe-area top
            if let Some(win) = unsafe { view.window() } {
                let top = win.safeAreaInsets().top as f64;
                if top > 0.0 {
                    top_setter.set(top + 6.0);
                }
            }

            // Native iOS liquid glass blur effect and subtle perimeter ring
            let tag: objc2::ffi::NSInteger = 9992;
            if view.viewWithTag(tag).is_none() {
                let blur = objc2_ui_kit::UIBlurEffect::effectWithStyle(
                    objc2_ui_kit::UIBlurEffectStyle::SystemUltraThinMaterial,
                    mtm,
                );
                let effect_view = objc2_ui_kit::UIVisualEffectView::initWithEffect(
                    objc2_ui_kit::UIVisualEffectView::alloc(mtm),
                    Some(&blur),
                );
                effect_view.setTag(tag);
                effect_view.setFrame(view.bounds());
                effect_view.setAutoresizingMask(objc2_ui_kit::UIViewAutoresizing(
                    objc2_ui_kit::UIViewAutoresizing::FlexibleWidth.0
                        | objc2_ui_kit::UIViewAutoresizing::FlexibleHeight.0,
                ));
                effect_view.setUserInteractionEnabled(false);

                let view_layer: objc2::rc::Retained<objc2_quartz_core::CALayer> = view.layer();
                view_layer.setCornerRadius(19.0);
                view_layer.setMasksToBounds(true);
                view_layer.setBorderWidth(0.5);
                let border_color = objc2_ui_kit::UIColor::colorWithWhite_alpha(1.0, 0.22);
                view_layer.setBorderColor(Some(&border_color.CGColor()));

                view.insertSubview_atIndex(&effect_view, 0);
                view.setClipsToBounds(true);
            }
        })
    };
    #[cfg(not(target_os = "ios"))]
    let btn = {
        btn
    };

    // Delayed fallback check in case window insets weren't populated on initial pass
    #[cfg(target_os = "ios")]
    day::reactive::on_main_delayed(200, move || {
        let top = safe_area_top();
        if top > 20.0 {
            top_setter_delayed.set(top + 6.0);
        }
    });

    btn
        .translation(0.0, move || top_sig.get())
        .padding(Insets {
            top: 0.0,
            leading: 16.0,
            bottom: 0.0,
            trailing: 0.0,
        })
}

/// The circular liquid glass button with status icon inside.
pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let pressed = Signal::new(false);
    let p_set = pressed.setter();

    let icon_piece = when(
        move || state.conn_status.get() == ConnStatus::Connecting,
        move || super::spinner::render(state, 17.0).any(),
    )
    .otherwise(move || {
        when(
            move || state.conn_status.get() == ConnStatus::Offline,
            move || {
                vector(res::vectors::status_offline)
                    .frame(17.0, 17.0)
                    .tint(Color::hex(0x8E8E93))
                    .any()
            },
        )
        .otherwise(move || {
            when(
                move || state.conn_status.get() == ConnStatus::Error,
                move || {
                    vector(res::vectors::status_error)
                        .frame(17.0, 17.0)
                        .tint(Color::hex(0xEF4444))
                        .any()
                },
            )
            .otherwise(move || {
                vector(res::vectors::status_ok)
                    .frame(17.0, 17.0)
                    .tint(move || Color::hex(state.accent_color.get()))
                    .any()
            })
        })
    });

    zstack((
        icon_piece,
    ))
    .align(Alignment::Center)
    .frame(38.0, 38.0)
    .background(move || {
        if day::dark_mode() {
            Color::rgba(0.25, 0.25, 0.28, 0.45)
        } else {
            Color::rgba(1.0, 1.0, 1.0, 0.65)
        }
    })
    .corner_radius(19.0)
    .scale(move || if pressed.get() { 0.90 } else { 1.0 })
    .animation(Animation::ease_out(120))
    .on_tap(move || {
        crate::shared::haptics::tick();
        p_set.set(true);
        day::reactive::on_main_delayed(120, move || {
            p_set.set(false);
        });
        state
            .show_network_modal
            .set(Some(String::from("net")));
    })
}
