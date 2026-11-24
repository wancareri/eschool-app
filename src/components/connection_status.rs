use crate::app::{AppState, ConnStatus};
use crate::res;
use day::prelude::*;


#[cfg(feature = "uikit")]
pub(crate) fn icon_group(status: ConnStatus) -> u8 {
    match status {
        ConnStatus::Connecting => 0,
        ConnStatus::Offline | ConnStatus::Error => 1,
        ConnStatus::Connected | ConnStatus::Idle => 2,
    }
}

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
    #[cfg(feature = "uikit")]
    crate::ios::conn_status::prepare_transition(status);
    let button = toolbar_button(day::id!(connection_status), title(status));
    let button = match status {
        ConnStatus::Connecting => button.icon(Symbol::Refresh),
        ConnStatus::Offline | ConnStatus::Error => button.icon(Symbol::Warning),
        ConnStatus::Connected | ConnStatus::Idle => button.icon(Symbol::Check),
    };
    vec![
        button
            .label_style(LabelStyle::IconOnly)
            .placement(ToolbarPlacement::Primary)
            .action(move || {
                crate::app::preferences::haptics::tick();
                state.show_network_modal.set(Some("net".into()));
            }),
    ]
}
