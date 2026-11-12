day::routes! {
    pub enum Section {
        Login => "login",
        Diary => "diary",
        Schedule => "schedule",
        Teachers => "teachers",
        Settings => "settings",
    }
}

pub mod diary;
pub mod login;
pub mod pin;
pub mod schedule;
pub mod settings;
pub mod teachers;
