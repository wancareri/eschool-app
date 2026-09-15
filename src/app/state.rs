//! Application state — the central signal hub.

use day::prelude::*;
use eschool_api::entities::*;
use crate::shared::nslog;

/// An official mark set by a teacher.
#[derive(Debug, Clone)]
pub struct OfficialMark {
    pub value: f64,
    pub kind: String,
    pub author: String,
}

const TOKEN_KEY: &str = "auth.token";
const FULL_NAME_KEY: &str = "auth.full_name";
const SCHOOL_NAME_KEY: &str = "auth.school_name";

#[derive(Clone, Copy)]
pub struct AppState {
    // ── auth ─────────────────────────────────────────────────────────────
    pub is_authenticated: Signal<bool>,
    pub loading: Signal<bool>,
    pub error_msg: Signal<String>,
    pub remember_me: Signal<bool>,

    // ── user ─────────────────────────────────────────────────────────────
    pub full_name: Signal<String>,
    pub school_name: Signal<String>,
    pub class_label: Signal<String>,
    pub is_graduating: Signal<bool>,

    // ── theme ────────────────────────────────────────────────────────────
    pub accent_color: Signal<u32>,

    // ── diary ────────────────────────────────────────────────────────────
    pub lessons: Signal<Vec<DaySchedule>>,
    pub lessons_loading: Signal<bool>,
    pub current_week: Signal<String>,
    pub all_weeks: Signal<Vec<WeekActivity>>,
    pub current_week_index: Signal<i32>,
    pub all_marks: Signal<Vec<(String, f64)>>,
    pub loaded_mark_weeks: Signal<std::collections::HashSet<String>>,
    pub current_quarter: Signal<usize>,
    pub quarter_marks: Signal<std::collections::HashMap<String, Vec<f64>>>,
    pub official_marks: Signal<std::collections::HashMap<String, Vec<OfficialMark>>>,
    pub marks_loading: Signal<bool>,
    pub year_quarter_data: Signal<Vec<(String, std::collections::HashMap<String, Vec<f64>>)>>,

    // ── per-quarter storage (index 0..3) ─────────────────────────────────
    pub quarter_all_marks: Signal<Vec<std::collections::HashMap<String, Vec<f64>>>>,
    pub quarter_official_marks: Signal<Vec<std::collections::HashMap<String, Vec<OfficialMark>>>>,

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

        let remember = day::prefs::get("auth.remember_me")
            .map(|v| v == "true")
            .unwrap_or(false);

        let accent = day::prefs::get("app.accent_color")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(crate::shared::colors::DEFAULT_ACCENT);

        let state = Self {
            is_authenticated: Signal::new(has_token),
            loading: Signal::new(false),
            error_msg: Signal::new(String::new()),
            remember_me: Signal::new(remember),
            full_name: Signal::new(day::prefs::get(FULL_NAME_KEY).unwrap_or_default()),
            school_name: Signal::new(day::prefs::get(SCHOOL_NAME_KEY).unwrap_or_default()),
            class_label: Signal::new(String::new()),
            is_graduating: Signal::new(false),
            accent_color: Signal::new(accent),
            lessons: Signal::new(Vec::new()),
            lessons_loading: Signal::new(false),
            current_week: Signal::new(String::new()),
            all_weeks: Signal::new(Vec::new()),
            current_week_index: Signal::new(0),
            all_marks: Signal::new(Vec::new()),
            loaded_mark_weeks: Signal::new(std::collections::HashSet::new()),
            current_quarter: Signal::new(0),
            quarter_marks: Signal::new(std::collections::HashMap::new()),
            official_marks: Signal::new(std::collections::HashMap::new()),
            marks_loading: Signal::new(false),
            year_quarter_data: Signal::new(Vec::new()),
            quarter_all_marks: Signal::new(Vec::new()),
            quarter_official_marks: Signal::new(Vec::new()),
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
