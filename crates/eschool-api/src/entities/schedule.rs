use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BellTime {
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    pub number: u32,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BellDay {
    pub uuid: Option<String>,
    pub days: Vec<u32>,
    pub time_of_bells: Vec<BellTime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BellSchedule {
    pub uuid: String,
    pub title: String,
    pub shift: u32,
    #[serde(default)]
    pub levels: Vec<u32>,
    #[serde(default)]
    pub period: Option<String>,
    pub days_of_week: Vec<BellDay>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimetableSlot {
    pub uuid: String,
    #[serde(default)]
    pub summary: Option<String>,
    pub number: u32,
    #[serde(default)]
    pub room_id: Option<String>,
    pub lesson_template_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimetableTimeSlot {
    pub time_of_bells: BellTime,
    pub slots: Vec<TimetableSlot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimetableDay {
    pub day_of_week: u32,
    pub timetable_slots: Vec<TimetableTimeSlot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timetable {
    pub uuid: String,
    pub schedule_of_bells_uuid: String,
    pub class_id: String,
    pub class_level: u32,
    #[serde(default)]
    pub period: Option<String>,
    pub days_of_week: Vec<TimetableDay>,
}
