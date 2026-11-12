use crate::app::AppState;
use crate::res;
use day::prelude::*;

/// Compact accent-colored spinner: the `spinner.svg` loader-circle vector spun
/// by the frame clock, tinted with the app accent. Replaces the native
/// UIActivityIndicatorView, whose fixed 20×20 intrinsic size ignored the
/// requested frame and rendered in the system grey. The display link only runs
/// while this piece is mounted, so an idle app never wakes it.
pub fn render(state: AppState, size: f64) -> impl Piece {
    render_gated(state, size, || true)
}

/// Like [`render`], but the display link only runs while `active` reports true.
/// A spinner kept mounted to crossfade its opacity must not keep the clock
/// ticking once it is invisible.
pub fn render_gated(
    state: AppState,
    size: f64,
    active: impl Fn() -> bool + 'static,
) -> impl Piece {
    let angle = Signal::new(0.0);
    let spin = angle;
    zstack((
        vector(res::vectors::spinner)
            .frame(size, size)
            .tint(move || Color::hex(state.accent_color.get()))
            .rotation(move || spin.get())
            .any(),
        when(active, move || {
            frame_clock(move |dt| {
                // ~one turn per second; untracked so the tick never subscribes.
                let next = angle.get_untracked() + dt.as_secs_f64() * 360.0;
                angle.set(if next >= 360.0 { next - 360.0 } else { next });
            })
            .any()
        }),
    ))
}
