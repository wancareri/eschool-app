use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize null JSON values or numbers as String.
fn null_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<serde_json::Value>::deserialize(deserializer)?;
    match opt {
        Some(serde_json::Value::String(s)) => Ok(s),
        Some(serde_json::Value::Number(n)) => Ok(n.to_string()),
        _ => Ok(String::new()),
    }
}

/// Flexible boolean deserializer supporting bool, numeric 0/1, string "1"/"true", arrays, objects.
fn flexible_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<serde_json::Value>::deserialize(deserializer)?;
    match opt {
        Some(serde_json::Value::Bool(b)) => Ok(b),
        Some(serde_json::Value::Number(n)) => Ok(n.as_i64().map_or(false, |v| v != 0)),
        Some(serde_json::Value::String(s)) => Ok(s == "1" || s.eq_ignore_ascii_case("true")),
        Some(serde_json::Value::Array(arr)) => Ok(!arr.is_empty()),
        Some(serde_json::Value::Object(map)) => Ok(!map.is_empty()),
        _ => Ok(false),
    }
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
    #[serde(default, deserialize_with = "null_string", alias = "uuid", alias = "id")]
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
    #[serde(
        default,
        deserialize_with = "flexible_bool",
        alias = "has_attachments",
        alias = "has_attachment",
        alias = "hasAttachments",
        alias = "has_files",
        alias = "has_links",
        alias = "hasAttachmentsOrLinks"
    )]
    pub has_attachments_or_links: bool,
    #[serde(default)]
    pub homework_source_id: Option<serde_json::Value>,
    #[serde(default)]
    pub lesson_mark: Option<LessonMark>,
    #[serde(default)]
    pub substitution: Option<serde_json::Value>,
    #[serde(default)]
    pub attachments: Option<serde_json::Value>,
    #[serde(default)]
    pub files: Option<serde_json::Value>,
    #[serde(default)]
    pub links: Option<serde_json::Value>,
}

impl LessonSlot {
    pub fn has_attachments(&self) -> bool {
        if self.has_attachments_or_links {
            return true;
        }
        if let Some(att) = &self.attachments {
            if let Some(arr) = att.as_array() {
                if !arr.is_empty() {
                    return true;
                }
            } else if !att.is_null() && att.as_bool() != Some(false) {
                return true;
            }
        }
        if let Some(f) = &self.files {
            if let Some(arr) = f.as_array() {
                if !arr.is_empty() {
                    return true;
                }
            } else if !f.is_null() && f.as_bool() != Some(false) {
                return true;
            }
        }
        if let Some(l) = &self.links {
            if let Some(arr) = l.as_array() {
                if !arr.is_empty() {
                    return true;
                }
            } else if !l.is_null() && l.as_bool() != Some(false) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DaySchedule {
    pub day_of_week: u32,
    pub date: u64,
    pub slots: Vec<LessonSlot>,
}
