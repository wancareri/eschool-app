//! Application state — the central signal hub.

use day::prelude::*;
use eschool_api::entities::*;
use crate::shared::{nslog, secure};

/// Connection status for the Telegram-style indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnStatus {
    /// No activity — hide indicator
    Idle,
    /// Connecting / loading data
    Connecting,
    /// Successfully connected (show briefly, then fade)
    Connected,
    /// Working offline from cache
    Offline,
    /// Error occurred
    Error,
}

/// An official mark set by a teacher.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    pub biometric_ok: Signal<bool>,

    // ── PIN lock ────────────────────────────────────────────────────────
    pub pin_lock_active: Signal<bool>,
    pub pin_input: Signal<String>,
    pub pin_error: Signal<bool>,

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
    pub week_cache: Signal<std::collections::HashMap<i32, Vec<DaySchedule>>>,
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

    // ── dev ────────────────────────────────────────────────────────────────
    pub log_version: Signal<u64>,

    // ── teachers ─────────────────────────────────────────────────────────
    pub subjects_teachers: Signal<Vec<SubjectWithTeacher>>,
    pub teachers_loading: Signal<bool>,

    // ── connection status ──────────────────────────────────────────────────
    pub conn_status: Signal<ConnStatus>,
}

impl Ambient for AppState {
    fn create() -> Self {
        nslog::nslog("[App] AppState::create()");
        let has_token = secure::load(TOKEN_KEY).is_some();

        let remember = secure::load("auth.remember_me")
            .map(|v| v == "true")
            .unwrap_or(false);

        let accent = day::prefs::get("app.accent_color")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(crate::shared::colors::DEFAULT_ACCENT);

        let biometric_lock = has_token
            && crate::shared::biometric::is_available()
            && crate::shared::biometric::is_enabled();

        let pin_lock = has_token && crate::shared::pin::is_enabled();
        let lock_active = biometric_lock || pin_lock;

        let state = Self {
            // Lock active → start unauthenticated (Face ID / PIN will unlock)
            is_authenticated: Signal::new(has_token && !lock_active),
            loading: Signal::new(false),
            error_msg: Signal::new(String::new()),
            remember_me: Signal::new(remember),
            biometric_ok: Signal::new(false),
            pin_lock_active: Signal::new(pin_lock),
            pin_input: Signal::new(String::new()),
            pin_error: Signal::new(false),
            full_name: Signal::new(secure::load(FULL_NAME_KEY).unwrap_or_default()),
            school_name: Signal::new(secure::load(SCHOOL_NAME_KEY).unwrap_or_default()),
            class_label: Signal::new(String::new()),
            is_graduating: Signal::new(false),
            accent_color: Signal::new(accent),
            lessons: Signal::new(Vec::new()),
            lessons_loading: Signal::new(false),
            current_week: Signal::new(String::new()),
            all_weeks: Signal::new(Vec::new()),
            current_week_index: Signal::new(0),
            week_cache: Signal::new(std::collections::HashMap::new()),
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
            conn_status: Signal::new(ConnStatus::Idle),
            log_version: Signal::new(0u64),
        };

        state.register_main();

        if has_token && !lock_active {
            crate::features::diary::load_all(state);
        } else if biometric_lock {
            nslog::nslog("[App] Biometric lock active, waiting for Face ID");
        } else if pin_lock {
            nslog::nslog("[App] PIN lock active, waiting for PIN");
        }
        state
    }
}

thread_local! {
    static MAIN_APP_STATE: std::cell::Cell<Option<AppState>> = const { std::cell::Cell::new(None) };
}

impl AppState {
    pub fn register_main(self) {
        MAIN_APP_STATE.with(|c| c.set(Some(self)));
    }

    pub fn get_main() -> Option<AppState> {
        MAIN_APP_STATE.with(|c| c.get())
    }
}
