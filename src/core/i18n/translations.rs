use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Translations {
    pub app: AppStrings,
    pub nav: NavStrings,
    pub menu: MenuStrings,
    pub cmd: CmdStrings,
    pub item: ItemStrings,
    pub section: SectionStrings,
    pub field: FieldStrings,
    pub diary: DiaryStrings,
    pub schedule: ScheduleStrings,
    pub teachers: TeachersStrings,
    pub settings: SettingsStrings,
    pub auth: AuthStrings,
    pub common: CommonStrings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppStrings {
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NavStrings {
    pub diary: String,
    pub schedule: String,
    pub teachers: String,
    pub settings: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MenuStrings {
    pub file: String,
    pub edit: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CmdStrings {
    pub add: String,
    pub delete: String,
    pub done: String,
    pub show_done: String,
    pub sidebar: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ItemStrings {
    pub none: String,
    pub kind_note: String,
    pub kind_task: String,
    pub kind_idea: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SectionStrings {
    pub basics: String,
    pub details: String,
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldStrings {
    pub name: String,
    pub name_hint: String,
    pub count: String,
    pub date: String,
    pub kind: String,
    pub done: String,
    pub rating: String,
    pub color: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiaryStrings {
    pub title: String,
    pub subtitle: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleStrings {
    pub title: String,
    pub subtitle: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TeachersStrings {
    pub title: String,
    pub subtitle: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsStrings {
    pub title: String,
    pub subtitle: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthStrings {
    pub login: String,
    pub logout: String,
    pub username: String,
    pub password: String,
    pub school: String,
    pub login_button: String,
    pub login_error: String,
    pub login_success: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommonStrings {
    pub loading: String,
    pub error: String,
    pub retry: String,
    pub cancel: String,
    pub save: String,
}
