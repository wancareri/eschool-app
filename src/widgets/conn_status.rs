//! Connection status lives in the native navbar, using the same toolbar on every page.
use crate::app::{AppState, ConnStatus};
use crate::res;
use day::prelude::*;

pub fn title(status: ConnStatus) -> String {
    match status {
        ConnStatus::Connecting => res::str::status_connecting_title().format(),
        ConnStatus::Offline => res::str::status_offline_title().format(),
        ConnStatus::Error => res::str::status_error_title().format(),
        ConnStatus::Connected | ConnStatus::Idle => res::str::status_ready_title().format(),
    }
}

pub fn message(status: ConnStatus) -> String {
    match status {
        ConnStatus::Connecting => res::str::status_connecting_message().format(),
        ConnStatus::Offline | ConnStatus::Error => res::str::status_error_message().format(),
        ConnStatus::Connected | ConnStatus::Idle => res::str::status_ready_message().format(),
    }
}

pub fn toolbar(state: AppState) -> Vec<ToolbarEntry> {
    let status = state.conn_status.get();
    let icon = match status {
        ConnStatus::Connecting => Symbol::Refresh,
        ConnStatus::Offline | ConnStatus::Error => Symbol::Warning,
        ConnStatus::Connected | ConnStatus::Idle => Symbol::Check,
    };
    vec![
        toolbar_button("connection-status", title(status))
            .icon(icon)
            .label_style(LabelStyle::IconOnly)
            .placement(ToolbarPlacement::Primary)
            .action(move || {
                crate::shared::haptics::tick();
                state.show_network_modal.set(Some("net".into()));
            }),
    ]
}
