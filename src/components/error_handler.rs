//! Window-level binding from app errors to the shared notice presentation.

use crate::app::AppState;
use crate::res;
use crate::components::notice::{self, Notice};

pub(crate) fn bind(state: AppState) {
    notice::bind_error(state, state.error_msg, move |raw| {
        if !state.is_authenticated.get_untracked()
            && !state.setup_new_password_active.get_untracked()
        {
            crate::pages::login::error_notice(raw)
        } else {
            let message = if raw.len() <= 240 && !raw.contains("http") && !raw.contains('{') {
                raw.to_owned()
            } else {
                res::str::notice_error_message().format()
            };
            Notice::error(res::str::notice_error_title().format(), message)
        }
    });
}
