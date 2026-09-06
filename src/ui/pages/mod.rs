pub mod detail;
pub mod navigate;
pub mod settings;
pub mod welcome;

pub(crate) use navigate::{detail_title, item_list_pane, navigate_page};
pub(crate) use settings::{settings_body, settings_page};
pub(crate) use welcome::welcome_page;
