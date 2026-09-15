use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub school_id: String,
    pub school_name: String,
    pub our_school_id: String,
    pub profile_id: String,
    pub person_kind: String,
    pub full_name: String,
    #[serde(default)]
    pub roles: Vec<String>,
    pub position: Option<String>,
    pub students_for_parent: Option<String>,
    pub parent_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchoolYear {
    pub uuid: String,
    pub summary: Option<String>,
    pub start_year: u32,
    pub end_year: u32,
    pub label: String,
    pub start_ts: u64,
    pub end_ts: u64,
    pub status: String,
    #[serde(default)]
    pub author: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmployeeFio {
    pub name: String,
}
