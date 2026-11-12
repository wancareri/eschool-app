use crate::app::AppState;
use crate::components::blur::ApplyGlassBlur;
use crate::app::preferences::colors;
use crate::ios::layout_diag::{PosLogExt, debug_log_button, debug_log_overlay};
use crate::res;
use day::prelude::*;

pub fn render() -> impl Piece {
    let state = AppState::ambient();
    let show_logs = Signal::new(false);

    zstack((
        when(
            move || state.setup_new_password_active.get(),
            move || {
                crate::components::header::page(
                    res::str::auth_setup_title(),
                    Signal::new(Vec::<String>::new()),
                    setup_password_form(state, show_logs),
                )
                .grow(Dimension::Both)
                .any()
            },
        )
        .otherwise(move || auth_form(state, show_logs).any()),
        when(
            move || show_logs.get(),
            move || {
                debug_log_overlay(show_logs)
                    .animation(AnimSpec::ease_out(250))
                    .any()
            },
        ),
    ))
    .grow(Dimension::Both)
    .background(move || {
        if day::dark_mode() {
            Color::hex(0x000000)
        } else {
            Color::hex(0xFFFFFF)
        }
    })
}

#[path = "profile.rs"]
mod profile;

#[path = "controls.rs"]
mod controls;
#[path = "credentials.rs"]
mod credentials;
#[path = "errors.rs"]
mod errors;
#[path = "flow.rs"]
mod flow;
#[path = "forms.rs"]
mod forms;

pub use controls::auth_toolbar;
use controls::*;
use credentials::*;
#[cfg(test)]
use errors::auth_error_code;
pub use errors::error_notice;
pub use flow::reset_auth_flow;
use flow::*;
use forms::*;

#[cfg(test)]
#[path = "auth_error_tests.rs"]
mod auth_error_tests;
