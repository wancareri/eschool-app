//! Native liquid glass circular connection status button for iOS.
//! Fully native UIKit UIButton component with UIBackgroundConfiguration (SystemUltraThinMaterial)
//! and SF Symbols ("wifi", "wifi.slash", "wifi.exclamationmark") / native UIActivityIndicatorView.
//! Tapping it opens the detailed network status sheet.

use crate::app::{AppState, ConnStatus};
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
        btn.uikit(move |view, _class, _mtm| {
            // Keep full bleed layout
            view.setTag(day_uikit::ANNOTATION_TAG);
            if let Some(parent) = view.superview() {
                parent.setTag(day_uikit::ANNOTATION_TAG);
            }

            // Sync with actual window safe-area top
            if let Some(win) = view.window() {
                let top = win.safeAreaInsets().top as f64;
                if top > 0.0 {
                    top_setter.set(top + 6.0);
                }
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

#[cfg(target_os = "ios")]
fn setup_native_liquid_glass_button(
    btn: &objc2_ui_kit::UIButton,
    state: AppState,
    mtm: objc2::MainThreadMarker,
) {
    use objc2::Message;
    use objc2_foundation::NSString;
    use objc2_ui_kit::{
        UIBackgroundConfiguration, UIBlurEffect, UIBlurEffectStyle, UIButtonConfiguration,
        UIButtonConfigurationCornerStyle, UIColor, UIImage,
    };

    fn apply_status(
        btn: &objc2_ui_kit::UIButton,
        status: ConnStatus,
        accent_hex: u32,
        mtm: objc2::MainThreadMarker,
    ) {
        let config = UIButtonConfiguration::plainButtonConfiguration(mtm);

        let bg = UIBackgroundConfiguration::clearConfiguration(mtm);
        let blur = UIBlurEffect::effectWithStyle(
            UIBlurEffectStyle::SystemUltraThinMaterial,
            mtm,
        );
        bg.setVisualEffect(Some(&blur));
        bg.setCornerRadius(19.0);
        bg.setBackgroundColor(Some(&UIColor::clearColor()));

        config.setBackground(&bg);
        config.setCornerStyle(UIButtonConfigurationCornerStyle::Capsule);

        let r = ((accent_hex >> 16) & 0xFF) as f64 / 255.0;
        let g = ((accent_hex >> 8) & 0xFF) as f64 / 255.0;
        let b = (accent_hex & 0xFF) as f64 / 255.0;

        match status {
            ConnStatus::Connecting => {
                config.setShowsActivityIndicator(true);
                let accent = UIColor::colorWithRed_green_blue_alpha(
                    r as _, g as _, b as _, 1.0,
                );
                config.setBaseForegroundColor(Some(&accent));
            }
            ConnStatus::Offline => {
                config.setShowsActivityIndicator(false);
                let img = UIImage::systemImageNamed(&NSString::from_str("wifi.slash"));
                config.setImage(img.as_deref());
                let gray = UIColor::colorWithWhite_alpha(0.55, 1.0);
                config.setBaseForegroundColor(Some(&gray));
            }
            ConnStatus::Error => {
                config.setShowsActivityIndicator(false);
                let img = UIImage::systemImageNamed(&NSString::from_str("wifi.exclamationmark"));
                config.setImage(img.as_deref());
                let red = UIColor::colorWithRed_green_blue_alpha(0.94, 0.27, 0.27, 1.0);
                config.setBaseForegroundColor(Some(&red));
            }
            ConnStatus::Connected | ConnStatus::Idle => {
                config.setShowsActivityIndicator(false);
                let img = UIImage::systemImageNamed(&NSString::from_str("wifi"));
                config.setImage(img.as_deref());
                let accent = UIColor::colorWithRed_green_blue_alpha(
                    r as _, g as _, b as _, 1.0,
                );
                config.setBaseForegroundColor(Some(&accent));
            }
        }

        btn.setConfiguration(Some(&config));
    }

    apply_status(btn, state.conn_status.get(), state.accent_color.get(), mtm);

    let btn_retained = btn.retain();
    day::reactive::watch(
        move || (state.conn_status.get(), state.accent_color.get()),
        move |&(status, accent), _| {
            if let Some(mtm) = objc2::MainThreadMarker::new() {
                apply_status(&btn_retained, status, accent, mtm);
            }
        },
    );
}

/// The circular liquid glass native UIButton with status icon inside.
#[cfg(target_os = "ios")]
pub fn render() -> impl Piece {
    let state = AppState::ambient();
    use day_uikit::UiKitExt;

    button("")
        .action(move || {
            crate::shared::haptics::tick();
            state
                .show_network_modal
                .set(Some(String::from("net")));
        })
        .frame(38.0, 38.0)
        .id("conn-status-native-button")
        .uikit(move |view, _class, mtm| {
            if let Some(btn) = view.downcast_ref::<objc2_ui_kit::UIButton>() {
                setup_native_liquid_glass_button(btn, state, mtm);
            }
        })
}

#[cfg(not(target_os = "ios"))]
pub fn render() -> impl Piece {
    let state = AppState::ambient();
    use crate::res;

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
        button("")
            .action(move || {
                crate::shared::haptics::tick();
                state
                    .show_network_modal
                    .set(Some(String::from("net")));
            })
            .frame(38.0, 38.0)
            .id("conn-status-native-button"),
    ))
    .align(Alignment::Center)
    .frame(38.0, 38.0)
    .background(move || {
        if day::dark_mode() {
            Color::rgba(0.25, 0.25, 0.28, 0.55)
        } else {
            Color::rgba(1.0, 1.0, 1.0, 0.75)
        }
    })
    .corner_radius(19.0)
}
