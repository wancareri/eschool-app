//! Application state — the central signal hub.

use day::prelude::*;
use eschool_api::entities::*;
use crate::shared::nslog;

const TOKEN_KEY: &str = "auth.token";
const FULL_NAME_KEY: &str = "auth.full_name";
const SCHOOL_NAME_KEY: &str = "auth.school_name";

#[derive(Clone, Copy)]
pub struct AppState {
    // ── auth ─────────────────────────────────────────────────────────────
    pub is_authenticated: Signal<bool>,
    pub loading: Signal<bool>,
    pub error_msg: Signal<String>,

    // ── user ─────────────────────────────────────────────────────────────
    pub full_name: Signal<String>,
    pub school_name: Signal<String>,
    pub class_label: Signal<String>,

    // ── diary ────────────────────────────────────────────────────────────
    pub lessons: Signal<Vec<DaySchedule>>,
    pub lessons_loading: Signal<bool>,
    pub current_week: Signal<String>,
    pub all_weeks: Signal<Vec<WeekActivity>>,
    pub current_week_index: Signal<i32>,
    pub all_marks: Signal<Vec<(String, f64)>>,
    pub loaded_mark_weeks: Signal<std::collections::HashSet<String>>,

    // ── schedule ─────────────────────────────────────────────────────────
    pub bell_times: Signal<Vec<BellTime>>,
    pub timetable_days: Signal<Vec<TimetableDay>>,
    pub schedule_loading: Signal<bool>,

    // ── teachers ─────────────────────────────────────────────────────────
    pub subjects_teachers: Signal<Vec<SubjectWithTeacher>>,
    pub teachers_loading: Signal<bool>,
}

impl Ambient for AppState {
    fn create() -> Self {
        nslog::nslog("[App] AppState::create()");
        let has_token = day::prefs::get(TOKEN_KEY)
            .map(|t| !t.is_empty())
            .unwrap_or(false);

        let state = Self {
            is_authenticated: Signal::new(has_token),
            loading: Signal::new(false),
            error_msg: Signal::new(String::new()),
            full_name: Signal::new(day::prefs::get(FULL_NAME_KEY).unwrap_or_default()),
            school_name: Signal::new(day::prefs::get(SCHOOL_NAME_KEY).unwrap_or_default()),
            class_label: Signal::new(String::new()),
            lessons: Signal::new(Vec::new()),
            lessons_loading: Signal::new(false),
            current_week: Signal::new(String::new()),
            all_weeks: Signal::new(Vec::new()),
            current_week_index: Signal::new(0),
            all_marks: Signal::new(Vec::new()),
            loaded_mark_weeks: Signal::new(std::collections::HashSet::new()),
            bell_times: Signal::new(Vec::new()),
            timetable_days: Signal::new(Vec::new()),
            schedule_loading: Signal::new(false),
            subjects_teachers: Signal::new(Vec::new()),
            teachers_loading: Signal::new(false),
        };

        if has_token {
            crate::features::diary::load_all(state);
        }
        state
    }
}
