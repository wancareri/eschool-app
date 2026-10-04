use day::prelude::*;

/// The rubber band's ceiling: how far the card can be pulled up.
const MAX_UP: f64 = 70.0;
/// The lift-gap filler's height: the ceiling plus 14pt that always tuck behind
/// the card's bottom edge — the filler's top edge then hides under the card and
/// the card's rounded bottom corners dissolve into filler material instead of
/// the dim showing through.
const FILLER_H: f64 = MAX_UP + 14.0;
/// The card's resize animation: content that grows (a panel opens, marks
/// appear) or shrinks re-lays the card with this ease — the window's top edge
/// slides up instead of jumping. Attached to the card itself (§8.4 implicit
/// animation), so it covers every content-driven size change in any caller.
const RESIZE_MS: u32 = 240;

/// Close is just the signal flip: the cover piece runs the native dismissal
/// itself and keeps its content mounted until the backend reports it hidden.
fn close_modal<R: Route, S: Binding<Option<R>>>(open: &S, reason: &str) {
    crate::shared::nslog::nslog(&format!("[cover] close_sheet: {reason}"));
    crate::shared::haptics::tick();
    open.write(None);
}

/// The material that fills the gap under the card while it is pulled up: a
/// spare section-card surface (the same dynamic grouped-card color on iOS),
/// parked one MAX_UP below the bottom edge and slid up by exactly the lift
/// distance — at rest it hides behind the card, while lifted it fills the gap
/// line-to-line. Interaction off, so a tap in the gap falls through to the dim.
#[cfg(target_os = "ios")]
fn gap_filler(drag_y: Signal<f64>) -> AnyPiece {
    use day_uikit::UiKitExt;
    use objc2_ui_kit::UIColor;
    column(())
        // The tweak sits first: the native view belongs to the column's own
        // node — the frame wrapper is layout-only and has no view to tweak.
        .uikit(|view, _class, _mtm| {
            view.setBackgroundColor(Some(&UIColor::secondarySystemGroupedBackgroundColor()));
            view.setUserInteractionEnabled(false);
        })
        .frame(crate::pages::diary::get_screen_width(), FILLER_H)
        .translation(0.0, move || MAX_UP + drag_y.get())
        .any()
}

#[cfg(not(target_os = "ios"))]
fn gap_filler(drag_y: Signal<f64>) -> AnyPiece {
    column(())
        .frame(crate::pages::diary::get_screen_width(), FILLER_H)
        .translation(0.0, move || MAX_UP + drag_y.get())
        .any()
}

/// The modal window — the reusable chrome of a bottom sheet, with no content in
/// it: a fullscreen cover with its dim, a draggable card at the bottom edge, the
/// grab handle, and everything the native side animates. What goes inside the
/// card is the caller's business; this piece owns only the shell.
///
/// `min_h` is the card's floor: a transparent fixed-height child under the body
/// makes the window AT LEAST that tall — content taller than the floor grows
/// the card past it, content under it still gets the taller sheet. The card
/// carries an implicit `RESIZE_MS` animation, so any content-driven height
/// change slides the window's top edge instead of jumping; the live drag opts
/// out of it with a zero-duration ambient (ambient intent wins over implicit).
///
/// day never attaches a cover's view to the page it sits in — it lives in its
/// own modal VC and its node measures 0×0 in the tree — so presenting it leaves
/// the page's subview walk untouched: the full-bleed chain holds and the tab bar
/// never moves. The cover is presented OverFullScreen (patched in the
/// wancareri/day fork: upstream's FullScreen drops the presenting view once the
/// transition lands, which showed black behind this dim instead of the live
/// page), so the page stays visible beneath the dim. The fork presents with the
/// card sliding up from off-screen under a dim that washes in on its own track,
/// and dismisses with the card gliding down the same distance while the dim
/// washes out in place — entrance and exit are one motion mirrored in time, the
/// darkening never travels with the card, and the card never fades mid-flight.
///
/// The card is draggable: pulling UP resists (a rubber band with ~70pt of
/// travel) and always glides back down; pulling DOWN follows the finger and
/// closes it after a nudge — a dismissal never travels upward in any phase.
/// Dim-tap and a downward release close the sheet through `open`; everything
/// else the caller closes itself through the same binding.
pub fn modal_window<R: Route, S: Binding<Option<R>>, C: Piece + 'static>(
    open: S,
    min_h: f64,
    content: impl Fn(&R) -> C + 'static,
) -> impl Piece {
    let o_dim = open.clone();
    let o_drag = open.clone();

    cover(
        open,
        move |route| {
            // Sheet drag. Vertical only (horizontal travel is simply ignored).
            // Both directions start from the finger; only the release animates.
            let drag_y = Signal::new(0.0);
            let o_freeze = o_drag.clone();
            let sheet_drag = move |d: Drag| {
                // Once the close has started the card is frozen: no event may touch it —
                // a fresh touch's zeroing would teleport it up while the sheet glides down.
                if o_freeze.peek().is_none() {
                    return;
                }
                match d.phase {
                DragPhase::Began => drag_y.set(0.0),
                DragPhase::Changed => {
                    let dy = d.translation.y;
                    let off = if dy > 0.0 {
                        // Down: follows the finger 1:1 — this is the dismiss direction.
                        dy
                    } else {
                        // Up: rubber band — asymptotes at MAX_UP instead of refusing,
                        // so the sheet yields a little and then visibly resists.
                        -MAX_UP * (1.0 - 1.0 / (1.0 + (-dy) / MAX_UP))
                    };
                    // The card carries the implicit RESIZE animation, and ambient
                    // intent overrides it — a zero-duration ambient keeps the live
                    // drag on the finger (it resolves to a 10ms retarget), while the
                    // release below still asks for its own 240ms ease explicitly.
                    with_animation(AnimSpec::linear(0), move || drag_y.set(off));
                }
                DragPhase::Ended => {
                    if drag_y.get() > 10.0 {
                        // Any downward pull past a tap-sized nudge dismisses — the card keeps
                        // its dragged offset while the cover slides everything down as ONE
                        // downward motion. No snap-back exists for the down direction any
                        // more, so nothing can fly up here; the next open builds a fresh drag
                        // signal anyway, so there is nothing to reset.
                        close_modal(&o_freeze, "drag-end");
                    } else {
                        // The card is at rest or above its spot: returning is downward in
                        // every case (a rubber-band lift glides back down; a nudge under the
                        // nudge threshold is a couple of points), so this direction can never
                        // read as an upward jerk either.
                        with_animation(AnimSpec::ease_out(240), move || drag_y.set(0.0));
                    }
                }
                }
            };

            let o_tap = o_dim.clone();
            zstack((
                // The dim is the cover's own background (edge-to-edge, over the
                // status bar and the home indicator); this layer is transparent
                // and only catches taps outside the card. The card above it is a
                // later zstack child, so it wins the hit-test over its own area.
                button("")
                    .action(move || close_modal(&o_tap, "dim-tap"))
                    .grow()
                    .any(),

                // The lift gap: pulling the card up would open a hole above the
                // bottom edge — this parks the same card material one MAX_UP
                // below and slides it up by exactly the lift distance, so the
                // hole always shows the sheet, never the dim.
                gap_filler(drag_y),

                // Bottom sheet card: day's semantic section surface — the platform's
                // theme-adaptive grouped-card material (secondary system grouped
                // background on iOS), so it tracks light/dark mode with no app palette.
                // The paddings below compose with the section's own 14pt inset to keep
                // the original 24/16/32pt edges. The translation is the sheet drag —
                // the card's implicit animation would smooth it, so the live drag
                // runs under a zero-duration ambient; only the release snaps back
                section((
                    // Drag handle pill at the top
                    row((
                        spacer().grow(),
                        column(()).frame(36.0, 4.0).background(Color::rgba(0.7, 0.7, 0.7, 0.6)).corner_radius(2.0),
                        spacer().grow(),
                    ))
                    .grow()
                    .padding(Insets {
                        top: 0.0,
                        leading: 0.0,
                        bottom: 6.0,
                        trailing: 0.0,
                    }),

                    // The card body: the floor is a transparent fixed-height
                    // child, so the stack sizes to the LARGER of the two —
                    // content past the floor grows the card, content under it
                    // still gets the taller window. Top-aligned, so the body
                    // sits under the handle and the slack fills the bottom.
                    if min_h > 0.0 {
                        zstack((
                            column(()).height(min_h),
                            content(route).any(),
                        ))
                        .align(Alignment::Top)
                        .any()
                    } else {
                        content(route).any()
                    },
                ))
                // The card's own §8.4 implicit animation: every content-driven
                // size change of the card and of everything inside it eases
                // RESIZE_MS, so the window's top edge slides up when a panel
                // adds elements instead of jumping.
                .animation(AnimSpec::ease_out(RESIZE_MS))
                .translation(0.0, move || drag_y.get())
                .any(),
            ))
            .align(Alignment::Bottom)
            .grow()
            // The recognizer sits on the whole overlay — a pull starting on the dim
            // drags too (the dim itself doesn't move: only the card translates).
            .on_drag(sheet_drag)
        },
    )
    .unrouted()
    .background(|_| Color::rgba(0.0, 0.0, 0.0, 0.50))
}
