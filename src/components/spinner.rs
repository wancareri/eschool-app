use crate::app::AppState;
use crate::res;
use day::prelude::*;

pub fn render(state: AppState, size: f64) -> impl Piece {
    render_gated(state, size, || true)
}

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
                let next = angle.get_untracked() + dt.as_secs_f64() * 360.0;
                angle.set(if next >= 360.0 { next - 360.0 } else { next });
            })
            .any()
        }),
    ))
}
