use serde::{Deserialize, Serialize};

// Auth models
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub school_id: String,
    pub school_name: String,
    pub our_school_id: String,
    pub profile_id: String,
    pub person_kind: String,
    pub full_name: String,
    pub roles: Vec<String>,
    pub position: Option<String>,
    pub students_for_parent: Option<String>,
    pub parent_code: Option<String>,
}

// School year models
#[derive(Debug, Serialize, Deserialize)]
pub struct SchoolYear {
    pub uuid: String,
    pub start_year: u32,
    pub end_year: u32,
    pub label: String,
    pub start_ts: u64,
    pub end_ts: u64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeActivity {
    pub uuid: String,
    pub school_year: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub start_date: u64,
    pub end_date: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeekActivity {
    pub uuid: String,
    pub summary: String,
    pub time_activity_id: String,
    pub start_ts: u64,
    pub end_ts: u64,
}

// Class models
#[derive(Debug, Serialize, Deserialize)]
pub struct Class {
    pub uuid: String,
    pub label: String,
    pub level: u32,
    pub school: Option<String>,
    pub dt_disbanded: Option<String>,
    pub graduating_year: Option<u32>,
    pub class_number: String,
    pub graduating: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassesByDate {
    pub classes_by_date: std::collections::HashMap<String, Vec<String>>,
    pub classes: Vec<Class>,
}

// Lesson models
#[derive(Debug, Serialize, Deserialize)]
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
    pub has_attachments_or_links: bool,
    pub homework_source_id: Option<String>,
    pub lesson_mark: Option<String>,
    pub substitution: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaySchedule {
    pub day_of_week: u32,
    pub date: u64,
    pub slots: Vec<LessonSlot>,
}

// Subject models
#[derive(Debug, Serialize, Deserialize)]
pub struct Subject {
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub group_name: String,
}

// Bell schedule models
#[derive(Debug, Serialize, Deserialize)]
pub struct BellTime {
    pub number: u32,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BellDay {
    pub uuid: Option<String>,
    pub days: Vec<u32>,
    pub time_of_bells: Vec<BellTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BellSchedule {
    pub uuid: String,
    pub title: String,
    pub shift: u32,
    pub levels: Vec<u32>,
    pub period: String,
    pub days_of_week: Vec<BellDay>,
}

// Timetable models
#[derive(Debug, Serialize, Deserialize)]
pub struct TimetableSlot {
    pub uuid: String,
    pub summary: String,
    pub number: u32,
    pub room_id: String,
    pub lesson_template_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimetableTimeSlot {
    pub time_of_bells: BellTime,
    pub slots: Vec<TimetableSlot>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimetableDay {
    pub day_of_week: u32,
    pub timetable_slots: Vec<TimetableTimeSlot>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Timetable {
    pub uuid: String,
    pub schedule_of_bells_uuid: String,
    pub class_id: String,
    pub class_level: u32,
    pub period: String,
    pub days_of_week: Vec<TimetableDay>,
}

// Final grades models
#[derive(Debug, Serialize, Deserialize)]
pub struct FinalGrades {
    pub marks: serde_json::Value,
    pub behaviour: serde_json::Value,
    pub diary_signature: serde_json::Value,
}

// Planning models
#[derive(Debug, Serialize, Deserialize)]
pub struct PlanningTemplate {
    pub uuid: String,
    pub study_hours: u32,
    pub study_group: StudyGroup,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudyGroup {
    pub name: String,
    pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SchoolSubject {
    pub uuid: String,
    pub summary: String,
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanningSubject {
    pub uuid: String,
    pub partition_type: String,
    pub school_subject: SchoolSubject,
    pub templates: Vec<PlanningTemplate>,
    pub pass_exam: Option<String>,
}

// Premise models
#[derive(Debug, Serialize, Deserialize)]
pub struct PremiseType {
    pub uuid: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Premise {
    pub uuid: String,
    pub audience_number: String,
    pub audience_name: String,
    pub hull: Option<String>,
    pub premise_type: PremiseType,
}

// Meal models
#[derive(Debug, Serialize, Deserialize)]
pub struct MealType {
    pub uuid: String,
    pub name: String,
    pub ordinal: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EatingJournalRequest {
    pub school_external_uuid: String,
    pub class_internal_uuid: String,
    pub dt_eating_from: String,
    pub dt_eating_to: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EatingJournalResponse {
    pub school_external_uuid: String,
    pub class_internal_uuid: String,
    pub dt_eating_from: String,
    pub dt_eating_to: String,
    pub student_nutrition_info: Vec<serde_json::Value>,
    pub class_eating_day_info: Vec<serde_json::Value>,
    pub student_eating_marks: Vec<serde_json::Value>,
}

// Pagination models
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub number: u32,
    pub size: u32,
    pub total_pages: u32,
    pub total_elements: u32,
    pub content: Vec<T>,
    pub first: bool,
    pub last: bool,
}

// Supplementary lesson models
#[derive(Debug, Serialize, Deserialize)]
pub struct SupplementaryLessonCollector {
    pub excused_absence_count: u32,
    pub unexcused_absence_count: u32,
    pub total_absence_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupplementaryLessonDiary {
    pub lessons: Vec<serde_json::Value>,
    pub collector: SupplementaryLessonCollector,
}
