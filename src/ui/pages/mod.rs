pub mod detail;
pub mod diary;
pub mod navigate;
pub mod schedule;
pub mod settings;
pub mod teachers;
pub mod welcome;

pub(crate) use diary::diary_page;
pub(crate) use navigate::{detail_title, item_list_pane, navigate_page};
pub(crate) use schedule::schedule_page;
pub(crate) use settings::{settings_body, settings_page};
pub(crate) use teachers::teachers_page;
pub(crate) use welcome::welcome_page;
