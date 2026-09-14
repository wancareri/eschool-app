use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubjectWithTeacher {
    pub id: String,
    pub partition_type: String,
    #[serde(rename = "type")]
    pub subject_type: String,
    pub subject_title: String,
    pub level_of_study: String,
    pub teacher_id: String,
    pub teacher: String,
    pub group_id: String,
    pub group_name: Option<String>,
}
