use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LessonMark {
    pub mark: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub uuid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LessonSlot {
    pub lesson_uuid: String,
    pub lesson_template_id: String,
    pub subject_id: String,
    pub subject_title: String,
    pub teacher_id: String,
    #[serde(rename = "type")]
    pub lesson_type: Option<String>,
    pub number: u32,
    pub topic: Option<String>,
    pub homework: Option<String>,
    pub message: Option<String>,
    pub start_time: String,
    #[serde(default)]
    pub has_attachments_or_links: bool,
    #[serde(default)]
    pub homework_source_id: Option<serde_json::Value>,
    #[serde(default)]
    pub lesson_mark: Option<LessonMark>,
    #[serde(default)]
    pub substitution: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DaySchedule {
    pub day_of_week: u32,
    pub date: u64,
    pub slots: Vec<LessonSlot>,
}
