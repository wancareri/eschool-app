// SPDX-License-Identifier: MPL-2.0
//! App-owned glass surfaces; Day 0.7 uses semantic cards instead of glass decorators.
use day::prelude::*;

pub trait ApplyGlassBlur: Decorate + Sized + 'static {
    fn apply_glass_blur(self) -> AnyPiece {
        self.glass_radius(16.0)
    }

    fn glass_radius(self, radius: f64) -> AnyPiece {
        #[cfg(target_os = "ios")]
        {
            use day_uikit::UiKitExt;
            // The background creates the native container the tweak owns.
            self.background(Color::rgba(0.0, 0.0, 0.0, 0.0))
                .uikit(move |view, _, mtm| {
                    use objc2::rc::Retained;
                    use objc2::{MainThreadOnly, msg_send};
                    use objc2_ui_kit::{
                        UIBlurEffect, UIBlurEffectStyle, UIViewAutoresizing, UIVisualEffect,
                        UIVisualEffectView,
                    };
                    let effect: Retained<UIVisualEffect> =
                        if let Some(class) = objc2::runtime::AnyClass::get(c"UIGlassEffect") {
                            unsafe { msg_send![class, effectWithStyle: 0isize] }
                        } else {
                            UIBlurEffect::effectWithStyle(
                                UIBlurEffectStyle::SystemUltraThinMaterial,
                                mtm,
                            )
                            .into_super()
                        };
                    let blur = UIVisualEffectView::initWithEffect(
                        UIVisualEffectView::alloc(mtm),
                        Some(&effect),
                    );
                    blur.setFrame(view.bounds());
                    blur.setAutoresizingMask(
                        UIViewAutoresizing::FlexibleWidth | UIViewAutoresizing::FlexibleHeight,
                    );
                    blur.setUserInteractionEnabled(false);
                    view.insertSubview_atIndex(&blur, 0);
                    view.layer().setCornerRadius(radius);
                    view.setClipsToBounds(true);
                })
                .any()
        }
        #[cfg(not(target_os = "ios"))]
        {
            self.background(move || {
                if day::dark_mode() {
                    Color::rgba(0.18, 0.18, 0.20, 0.82)
                } else {
                    Color::rgba(0.96, 0.96, 0.98, 0.82)
                }
            })
            .corner_radius(radius)
            .any()
        }
    }
}
impl<P: Decorate + Sized + 'static> ApplyGlassBlur for P {}
