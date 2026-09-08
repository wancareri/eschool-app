pub mod detail;
pub mod diary;
pub mod login;
pub mod navigate;
pub mod schedule;
pub mod settings;
pub mod teachers;

pub(crate) use diary::diary_page;
pub(crate) use login::login_page;
pub(crate) use schedule::schedule_page;
pub(crate) use settings::{settings_body, settings_page};
pub(crate) use teachers::teachers_page;
