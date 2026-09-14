//! API endpoint URL fragments.

pub const AUTH_ME: &str = "/api/v1/admin/auth/me";
pub const SCHOOL_YEAR: &str = "/api/v1/education/diary/school_year";
pub const WEEK_ACTIVITIES: &str = "/api/v1/education/diary/time_activities/week_activities";

pub fn classes(school_id: &str, profile_id: &str) -> String {
    format!("/api/v1/education/diary/schools/{school_id}/students/{profile_id}/classes")
}

pub fn lessons(school_id: &str, class_id: &str, profile_id: &str, week_uuid: &str) -> String {
    format!(
        "/api/v1/education/diary/schools/{school_id}/classes/{class_id}/students/{profile_id}/lessons?week_activity_uuid={week_uuid}"
    )
}

pub fn bells(school_id: &str) -> String {
    format!("/api/v1/education/diary/schools/{school_id}/bells/whole")
}

pub fn timetable(school_id: &str, class_id: &str) -> String {
    format!("/api/v1/education/diary/schools/{school_id}/classes/{class_id}/timetables/whole")
}

pub fn subjects(school_id: &str, profile_id: &str, class_id: &str) -> String {
    format!("/api/v1/education/diary/schools/{school_id}/students/{profile_id}/classes/{class_id}/subjects")
}
