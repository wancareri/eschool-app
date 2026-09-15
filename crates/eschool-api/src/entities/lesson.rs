use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize null JSON values as empty string (for String fields).
fn null_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LessonMark {
    #[serde(default)]
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
    #[serde(default, deserialize_with = "null_string")]
    pub lesson_uuid: String,
    #[serde(default, deserialize_with = "null_string")]
    pub lesson_template_id: String,
    #[serde(default, deserialize_with = "null_string")]
    pub subject_id: String,
    #[serde(default, deserialize_with = "null_string")]
    pub subject_title: String,
    #[serde(default, deserialize_with = "null_string")]
    pub teacher_id: String,
    #[serde(rename = "type")]
    #[serde(default)]
    pub lesson_type: Option<String>,
    #[serde(default)]
    pub number: u32,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub homework: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default, deserialize_with = "null_string")]
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
