use serde::{Deserialize, Deserializer, Serialize};

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

/// Child/Student entry associated with a parent account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentForParent {
    pub profile_id: String,
    pub full_name: String,
    pub school_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInfo {
    #[serde(default, deserialize_with = "null_string")]
    pub user_id: String,
    #[serde(default, deserialize_with = "null_string")]
    pub school_id: String,
    #[serde(default, deserialize_with = "null_string")]
    pub school_name: String,
    #[serde(default, deserialize_with = "null_string")]
    pub our_school_id: String,
    #[serde(default, deserialize_with = "null_string")]
    pub profile_id: String,
    #[serde(default, deserialize_with = "null_string")]
    pub person_kind: String,
    #[serde(default, deserialize_with = "null_string")]
    pub full_name: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub position: Option<String>,
    #[serde(default)]
    pub students_for_parent: Option<Vec<StudentForParent>>,
    #[serde(default)]
    pub parent_code: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub second_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
}

impl UserInfo {
    /// Returns true if the user logged in as a parent.
    pub fn is_parent(&self) -> bool {
        self.person_kind.eq_ignore_ascii_case("PARENT")
            || self.roles.iter().any(|r| r.eq_ignore_ascii_case("PARENT"))
    }

    /// Returns true if the user logged in as a student/pupil.
    pub fn is_student(&self) -> bool {
        self.person_kind.eq_ignore_ascii_case("PUPIL")
            || self.person_kind.eq_ignore_ascii_case("STUDENT")
            || self.roles.iter().any(|r| r.eq_ignore_ascii_case("PUPIL") || r.eq_ignore_ascii_case("STUDENT"))
    }

    /// List of children attached to this parent account (empty if not a parent).
    pub fn children(&self) -> &[StudentForParent] {
        self.students_for_parent.as_deref().unwrap_or_default()
    }
}

/// Detailed parent information from `/api/v1/admin/schools/parents/{parent_uuid}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParentInfo {
    pub uuid: String,
    #[serde(default)]
    pub login: Option<String>,
    #[serde(default)]
    pub fio: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub is_temp_password: Option<bool>,
    #[serde(default)]
    pub temp_password: Option<String>,
    #[serde(default)]
    pub email_address: Option<String>,
}

/// Child representative profile from `/api/v1/institution/child_representatives/{parent_uuid}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChildRepresentative {
    pub uuid: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub surname: Option<String>,
    #[serde(default)]
    pub patronymic: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub contact_number: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_student_user_info() {
        let json = r#"{
            "user_id": "111111",
            "school_id": "222222",
            "school_name": "Гимназия № 1",
            "our_school_id": "222222",
            "profile_id": "333333",
            "person_kind": "PUPIL",
            "full_name": "Иванов Иван",
            "roles": ["PUPIL"]
        }"#;
        let u: UserInfo = serde_json::from_str(json).unwrap();
        assert!(u.is_student());
        assert!(!u.is_parent());
        assert_eq!(u.school_id, "222222");
        assert_eq!(u.profile_id, "333333");
    }

    #[test]
    fn test_parse_parent_user_info() {
        let json = r#"{
            "user_id": "999999",
            "school_id": null,
            "school_name": null,
            "our_school_id": null,
            "profile_id": null,
            "person_kind": "PARENT",
            "full_name": "Петрова Анна",
            "roles": ["PARENT"],
            "students_for_parent": [
                {
                    "profile_id": "444444",
                    "full_name": "Петров Петр",
                    "school_id": "555555"
                }
            ]
        }"#;
        let u: UserInfo = serde_json::from_str(json).unwrap();
        assert!(u.is_parent());
        assert!(!u.is_student());
        assert_eq!(u.school_id, "");
        assert_eq!(u.profile_id, "");
        assert_eq!(u.children().len(), 1);
        assert_eq!(u.children()[0].profile_id, "444444");
        assert_eq!(u.children()[0].school_id, "555555");
        assert_eq!(u.children()[0].full_name, "Петров Петр");
    }
}

