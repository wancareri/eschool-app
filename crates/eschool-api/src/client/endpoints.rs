//! API endpoint URL fragments for diary.e-schools.by.

// ── Auth ──────────────────────────────────────────────────────────────
pub const AUTH_ME: &str = "/api/v1/admin/auth/me";
pub const LOGIN_STUDENT: &str = "/api/v1/admin/auth/login/student";
pub const AUTH_LOGIN: &str = "/api/v1/auth/login";
pub const AUTH_REFRESH: &str = "/api/v1/auth/refresh";
pub const IC_LOGIN: &str = "/api/ic/login";

pub fn data_for_login(uuid: &str) -> String {
    format!("/api/v1/admin/auth/data_for_login/{uuid}")
}

pub fn auth_callback(code: &str) -> String {
    format!("/api/v1/admin/auth/callback?code={code}")
}

// ── School year & periods ─────────────────────────────────────────────
pub const SCHOOL_YEAR: &str = "/api/v1/education/diary/school_year";
pub const TIME_ACTIVITIES: &str = "/api/v1/education/diary/time_activities";
pub const WEEK_ACTIVITIES: &str = "/api/v1/education/diary/time_activities/week_activities";

// ── Classes ───────────────────────────────────────────────────────────
pub fn classes(school_id: &str, profile_id: &str) -> String {
    format!("/api/v1/education/diary/schools/{school_id}/students/{profile_id}/classes")
}

// ── Lessons (student diary) ──────────────────────────────────────────
pub fn lessons(school_id: &str, class_id: &str, profile_id: &str, week_uuid: &str) -> String {
    format!(
        "/api/v1/education/diary/schools/{school_id}/classes/{class_id}/students/{profile_id}/lessons?week_activity_uuid={week_uuid}"
    )
}

pub fn attachments(school_id: &str, class_id: &str, profile_id: &str, lesson_uuid: &str) -> String {
    format!(
        "/api/v1/education/diary/schools/{school_id}/classes/{class_id}/students/{profile_id}/lessons/{lesson_uuid}/attachments_and_links"
    )
}

// ── Final marks & signatures ─────────────────────────────────────────
pub fn final_marks(school_id: &str, class_id: &str, profile_id: &str) -> String {
    format!(
        "/api/v1/education/diary/schools/{school_id}/classes/{class_id}/students/{profile_id}/final/whole"
    )
}

pub fn diary_signatures_weekly(school_id: &str, class_id: &str, profile_id: &str) -> String {
    format!(
        "/api/v1/education/diary/schools/{school_id}/classes/{class_id}/students/{profile_id}/diary_signatures/weekly"
    )
}

pub fn diary_signatures_weekly_activity(school_id: &str, class_id: &str, profile_id: &str, activity_id: &str) -> String {
    format!(
        "/api/v1/education/diary/schools/{school_id}/classes/{class_id}/students/{profile_id}/diary_signatures/weekly/activities/{activity_id}"
    )
}

pub fn diary_signatures_final(school_id: &str, class_id: &str, profile_id: &str, period_uuid: &str) -> String {
    format!(
        "/api/v1/education/diary/schools/{school_id}/classes/{class_id}/students/{profile_id}/diary_signatures/final/activities/{period_uuid}"
    )
}

// ── Subjects & teachers ──────────────────────────────────────────────
pub fn educational_subjects(school_id: &str, class_id: &str, profile_id: &str) -> String {
    format!(
        "/api/v1/education/diary/schools/{school_id}/classes/{class_id}/students/{profile_id}/educational_subjects"
    )
}

pub fn subjects(school_id: &str, profile_id: &str, class_id: &str) -> String {
    format!(
        "/api/v1/education/diary/schools/{school_id}/students/{profile_id}/classes/{class_id}/subjects"
    )
}

pub fn employee_fio(school_id: &str, employee_uuid: &str) -> String {
    format!(
        "/api/v1/education/diary/schools/{school_id}/employees/{employee_uuid}/fio"
    )
}

// ── Schedule ──────────────────────────────────────────────────────────
pub fn bells(school_id: &str) -> String {
    format!("/api/v1/education/diary/schools/{school_id}/bells/whole")
}

pub fn timetable(school_id: &str, class_id: &str) -> String {
    format!("/api/v1/education/diary/schools/{school_id}/classes/{class_id}/timetables/whole")
}

pub fn timetable_by_level(school_id: &str, level: u32) -> String {
    format!("/api/v1/education/diary/schools/{school_id}/levels/{level}/timetables/whole")
}

// ── Messages ──────────────────────────────────────────────────────────
pub fn master_messages(school_id: &str, class_id: &str, profile_id: &str) -> String {
    format!(
        "/api/v1/education/diary/schools/{school_id}/classes/{class_id}/students/{profile_id}/master_messages"
    )
}

// ── OAuth config ──────────────────────────────────────────────────────
pub const OAUTH_URL: &str = "https://oauth.rios.unibel.by";
pub const CLIENT_ID: &str = "oauth_diary_echools";
pub const REDIRECT_URI: &str = "https://diary.e-schools.by/api/v1/admin/auth/callback";
pub const SCOPE: &str = "openid profile offline_access organization.write person.write person.write.all person.read persons.read dictionaries.read organization.read";
