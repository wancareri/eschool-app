use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Class {
    pub uuid: String,
    pub summary: Option<String>,
    pub label: String,
    pub level: u32,
    pub school: Option<String>,
    pub dt_disbanded: Option<String>,
    pub graduating_year: Option<u32>,
    #[serde(default)]
    pub class_number: Option<String>,
    #[serde(default)]
    pub graduating: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassesByDate {
    pub classes_by_date: std::collections::HashMap<String, Vec<String>>,
    pub classes: Vec<Class>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeActivity {
    pub uuid: String,
    pub summary: Option<String>,
    pub school_year: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "type", default)]
    pub activity_type: Option<String>,
    #[serde(default)]
    pub start_ts: Option<u64>,
    #[serde(default)]
    pub end_ts: Option<u64>,
    #[serde(default)]
    pub start_date: Option<u64>,
    #[serde(default)]
    pub end_date: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeekActivity {
    pub uuid: String,
    #[serde(default)]
    pub summary: String,
    pub time_activity_id: String,
    pub start_ts: u64,
    pub end_ts: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EducationalSubject {
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiarySignature {
    pub uuid: String,
    pub school_id: String,
    pub class_id: String,
    pub student: String,
    pub period: String,
    pub parent_name: String,
    pub dt_create: u64,
    pub activity_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinalMarks {
    #[serde(default)]
    pub marks: serde_json::Value,
    #[serde(default)]
    pub behaviour: serde_json::Value,
    #[serde(default)]
    pub diary_signature: serde_json::Value,
}
