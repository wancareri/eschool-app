use day::prelude::*;

const MAX_UP: f64 = 70.0;
// Keep an ample sheet-colored surface behind the card while it rubber-bands upward.
// A short strip exposed the transparent cover near the bottom on tall iPhones.
const FILLER_H: f64 = 320.0;
#[allow(dead_code)]
const RESIZE_MS: u32 = 380;
pub(crate) const DIM_ALPHA: f64 = 0.50;

#[cfg(feature = "uikit")]
pub(crate) fn native_glass_button(node: day::RNode, prominent: bool) {
    use objc2::rc::Weak;
    use objc2_ui_kit::{
        NSDirectionalEdgeInsets, UIButton, UIButtonConfiguration, UIButtonConfigurationCornerStyle,
        UIFont,
    };

    fn apply(button: &UIButton, prominent: bool, mtm: objc2::MainThreadMarker) {
        // Keep the declarative title as the source of truth. UIKit can leave a
        // transient configuration title behind while a layout/drag update is
        // in flight, which previously produced a clipped-looking `Coninu`.
        let title = button
            .titleForState(objc2_ui_kit::UIControlState::Normal)
            .or_else(|| {
                button
                    .configuration()
                    .and_then(|configuration| configuration.title())
            })
            .unwrap_or_default();
        let class = objc2::class!(UIButtonConfiguration);
        let glass = if prominent {
            class
                .class_method(objc2::sel!(prominentGlassButtonConfiguration))
                .is_some()
        } else {
            class
                .class_method(objc2::sel!(clearGlassButtonConfiguration))
                .is_some()
        };
        let configuration = match (prominent, glass) {
            (true, true) => UIButtonConfiguration::prominentGlassButtonConfiguration(mtm),
            (false, true) => UIButtonConfiguration::clearGlassButtonConfiguration(mtm),
            (true, false) => UIButtonConfiguration::borderedProminentButtonConfiguration(mtm),
            (false, false) => UIButtonConfiguration::borderedButtonConfiguration(mtm),
        };
        configuration.setTitle(Some(&title));
        configuration.setCornerStyle(UIButtonConfigurationCornerStyle::Capsule);
        configuration.setContentInsets(NSDirectionalEdgeInsets {
            top: 12.0,
            leading: 24.0,
            bottom: 12.0,
            trailing: 24.0,
        });
        button.setConfiguration(Some(&configuration));
        if let Some(title_label) = button.titleLabel() {
            let size = title_label
                .font()
                .map_or(17.0, |font| unsafe { font.pointSize() });
            unsafe { title_label.setFont(Some(&UIFont::boldSystemFontOfSize(size))) };
        }
        button.layer().setMasksToBounds(false);
        button.setClipsToBounds(false);
        // Day places a control inside a few layout-only UIView wrappers.  A
        // glass button is allowed to paint outside its measured bounds while
        // being pulled; release only those immediate wrappers as well.  The
        // screen/root view remains clipped by UIKit as usual.
        let mut ancestor = button.superview();
        for _ in 0..3 {
            let Some(parent) = ancestor else { break };
            parent.layer().setMasksToBounds(false);
            parent.setClipsToBounds(false);
            ancestor = parent.superview();
        }
    }

    let _ = day_uikit::with_native(node, move |view, _, mtm| {
        let Some(button) = view.downcast_ref::<UIButton>() else {
            return;
        };
        // A button's title/style patch is performed by Day in the same mount
        // pass.  Set it now and once again on the next main-queue turn, after
        // that managed patch has finished, so UIKit keeps the glass surface.
        apply(button, prominent, mtm);
        let weak = dispatch2::MainThreadBound::new(Weak::new(button), mtm);
        dispatch2::DispatchQueue::main().exec_async(move || {
            let Some(mtm) = objc2::MainThreadMarker::new() else {
                return;
            };
            let Some(button) = weak.get(mtm).load() else {
                return;
            };
            apply(&button, prominent, mtm);
        });
    });
}

#[cfg(not(feature = "uikit"))]
pub(crate) fn native_glass_button(_node: day::RNode, _prominent: bool) {}

fn close_modal<R: Route, S: Binding<Option<R>>>(open: &S, reason: &str) {
    crate::ios::nslog::nslog(&format!("[cover] close_sheet: {reason}"));
    crate::ios::keyboard::dismiss();
    crate::ios::keyboard::set_keyboard_height(0.0);
    crate::app::preferences::haptics::tick();
    open.write(None);
}

fn card_bg() -> impl Fn() -> Color + Copy {
    move || {
        if day::dark_mode() {
            Color::hex(0x1C1C1E)
        } else {
            Color::hex(0xFFFFFF)
        }
    }
}

#[cfg(feature = "uikit")]
fn gap_filler(drag_y: Signal<f64>, kb_h: Signal<f64>) -> AnyPiece {
    use day_uikit::UiKitExt;
    use objc2_ui_kit::UIColor;
    column(())
        .background(card_bg())
        .uikit(|view, _class, _mtm| {
            view.setBackgroundColor(Some(&UIColor::secondarySystemGroupedBackgroundColor()));
            view.setUserInteractionEnabled(false);
            view.setTag(crate::ios::modal::FILLER_TAG);
        })
        .any()
        .frame(crate::ios::viewport::get_screen_width(), FILLER_H)
        .translation(0.0, move || MAX_UP + drag_y.get() - kb_h.get())
        .any()
        .uikit(|view, _, _| view.setUserInteractionEnabled(false))
        .any()
}

#[cfg(not(feature = "uikit"))]
fn gap_filler(drag_y: Signal<f64>, kb_h: Signal<f64>) -> AnyPiece {
    column(())
        .background(card_bg())
        .frame(crate::ios::viewport::get_screen_width(), FILLER_H)
        .translation(0.0, move || MAX_UP + drag_y.get() - kb_h.get())
        .any()
}

#[cfg(feature = "uikit")]
fn card_surface<P: Piece + 'static>(p: P) -> AnyPiece {
    use day_uikit::UiKitExt;
    use objc2_ui_kit::UIColor;
    // Give the sheet its own native container.  Applying a radius directly to
    // the content section lets a child background paint square into the top
    // edge on UIKit; this container is the single clipped sheet surface.
    column((p,))
        .background(card_bg())
        .corner_radius(20.0)
        .uikit(|view, _class, _mtm| {
            view.setBackgroundColor(Some(&UIColor::secondarySystemGroupedBackgroundColor()));
            crate::ios::modal::configure_card_surface(view, _mtm);
        })
        .any()
}

#[cfg(not(feature = "uikit"))]
fn card_surface<P: Piece + 'static>(p: P) -> AnyPiece {
    column((p,)).background(card_bg()).corner_radius(20.0).any()
}

pub fn modal_window<R: Route, S: Binding<Option<R>>, C: Piece + 'static>(
    open: S,
    min_h: f64,
    content: impl Fn(&R) -> C + 'static,
) -> impl Piece {
    let o_dim = open.clone();
    let o_drag = open.clone();

    cover(open, move |route| {
        crate::ios::nslog::nslog(&format!(
            "[Modal] cover closure evaluated for route: {}",
            route.key()
        ));
        let drag_y = Signal::new(0.0);
        let kb_h = crate::ios::keyboard::keyboard_height_signal();
        let o_freeze = o_drag.clone();
        let sheet_drag = move |d: Drag| {
            if o_freeze.peek().is_none() {
                return;
            }
            match d.phase {
                DragPhase::Began => drag_y.set(0.0),
                DragPhase::Changed => {
                    let dy = d.translation.y;
                    let off = if dy > 0.0 {
                        dy
                    } else {
                        -MAX_UP * (1.0 - 1.0 / (1.0 + (-dy) / MAX_UP))
                    };
                    with_animation(AnimSpec::linear(0), move || drag_y.set(off));
                }
                DragPhase::Ended => {
                    if kb_h.get() > 20.0 {
                        crate::ios::keyboard::dismiss();
                        with_animation(AnimSpec::ease_out(240), move || drag_y.set(0.0));
                    } else if drag_y.get() > 70.0 {
                        crate::ios::nslog::nslog(&format!(
                            "[Modal] drag ended past threshold ({:.1}), closing",
                            drag_y.get()
                        ));
                        close_modal(&o_freeze, "drag-end");
                    } else {
                        with_animation(AnimSpec::ease_out(240), move || drag_y.set(0.0));
                    }
                }
            }
        };

        let o_tap = o_dim.clone();
        let grabber = row((
            spacer().grow(Dimension::Both),
            column(())
                .frame(36.0, 4.0)
                .background(Color::rgba(0.7, 0.7, 0.7, 0.6))
                .corner_radius(2.0),
            spacer().grow(Dimension::Both),
        ))
        .grow(Dimension::Both)
        .padding(Insets {
            top: 0.0,
            leading: 0.0,
            bottom: 6.0,
            trailing: 0.0,
        });
        let card = card_surface(section((
            grabber,
            if min_h > 0.0 {
                zstack((column(()).height(min_h), content(route).any()))
                    .align(Alignment::Top)
                    .any()
            } else {
                content(route).any()
            },
        )))
        .on_drag(sheet_drag);
        let card = card.translation(0.0, move || drag_y.get() - kb_h.get());

        #[cfg(feature = "uikit")]
        let (filler, card) = (gap_filler(drag_y, kb_h).any(), card.any());
        #[cfg(not(feature = "uikit"))]
        let (filler, card) = (
            gap_filler(drag_y, kb_h)
                .animation(AnimSpec::ease_out(RESIZE_MS))
                .any(),
            card.animation(AnimSpec::ease_out(RESIZE_MS)).any(),
        );

        zstack((
            modal_dim(button("").action(move || {
                crate::ios::nslog::nslog("[Modal] dim button tapped");
                if kb_h.get() > 20.0 {
                    crate::ios::keyboard::dismiss();
                } else {
                    close_modal(&o_tap, "dim-tap");
                }
            }))
            .grow(Dimension::Both)
            .any(),
            filler,
            card,
        ))
        .align(Alignment::Bottom)
        .grow(Dimension::Both)
    })
    .unrouted()
    .background(|_| Color::rgba(0.0, 0.0, 0.0, DIM_ALPHA))
}

fn modal_dim<P: Piece>(p: P) -> AnyPiece {
    #[cfg(feature = "uikit")]
    {
        use day_uikit::UiKitExt;
        p.uikit(|view, _, mtm| crate::ios::modal::configure_cover(view, mtm))
            .any()
    }
    #[cfg(not(feature = "uikit"))]
    p.any()
}
