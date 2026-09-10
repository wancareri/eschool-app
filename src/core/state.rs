//! Global app state — the school data shared across every page.
//!
//! Uses Day's [`Ambient`] pattern so any page can reach it with
//! `ESchoolState::ambient()`. Data is loaded synchronously on startup (if a
//! token is stored) to keep the Signal-threading model simple.

use day::prelude::*;
use crate::core::network::models::*;

const BASE_URL: &str = "https://diary.e-schools.by";

// Preference keys for persistent auth data.
const TOKEN_KEY: &str = "auth.token";
const REFRESH_KEY: &str = "auth.refresh_token";
const SCHOOL_ID_KEY: &str = "auth.school_id";
const PROFILE_ID_KEY: &str = "auth.profile_id";
const CLASS_ID_KEY: &str = "auth.class_id";
const FULL_NAME_KEY: &str = "auth.full_name";
const SCHOOL_NAME_KEY: &str = "auth.school_name";

/// Everything the app knows about the logged-in student. `Copy` because every
/// field is a pointer-sized handle.
#[derive(Clone, Copy)]
pub(crate) struct ESchoolState {
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

    // ── schedule ─────────────────────────────────────────────────────────
    pub bell_times: Signal<Vec<BellTime>>,
    pub timetable_days: Signal<Vec<TimetableDay>>,
    pub schedule_loading: Signal<bool>,

    // ── teachers ─────────────────────────────────────────────────────────
    pub subjects_teachers: Signal<Vec<SubjectWithTeacher>>,
    pub teachers_loading: Signal<bool>,
}

impl Ambient for ESchoolState {
    fn create() -> Self {
        let has_token = day::prefs::get(TOKEN_KEY)
            .map(|t| !t.is_empty())
            .unwrap_or(false);

        let state = Self {
            is_authenticated: Signal::new(has_token),
            loading: Signal::new(false),
            error_msg: Signal::new(String::new()),
            full_name: Signal::new(
                day::prefs::get(FULL_NAME_KEY).unwrap_or_default(),
            ),
            school_name: Signal::new(
                day::prefs::get(SCHOOL_NAME_KEY).unwrap_or_default(),
            ),
            class_label: Signal::new(String::new()),
            lessons: Signal::new(Vec::new()),
            lessons_loading: Signal::new(false),
            current_week: Signal::new(String::new()),
            bell_times: Signal::new(Vec::new()),
            timetable_days: Signal::new(Vec::new()),
            schedule_loading: Signal::new(false),
            subjects_teachers: Signal::new(Vec::new()),
            teachers_loading: Signal::new(false),
        };

        if has_token {
            state.load_all_sync();
        }
        state
    }
}

impl ESchoolState {
    // ── public API ───────────────────────────────────────────────────────

    /// Persist the token pair and kick off a full data load.
    pub(crate) fn save_token(self, access: &str, refresh: &str) {
        day::prefs::set(TOKEN_KEY, access);
        day::prefs::set(REFRESH_KEY, refresh);
        self.is_authenticated.set(true);
        self.load_all_sync();
    }

    /// Login with username and password via OAuth.
    #[cfg(not(target_os = "ios"))]
    pub(crate) fn login_with_password(self, username: &str, password: &str) {
        use crate::core::network::auth::Auth;
        self.loading.set(true);
        self.error_msg.set(String::new());

        let auth = Auth::new();
        match auth.login_blocking(username, password) {
            Ok(token) => {
                day::prefs::set(TOKEN_KEY, &token.access_token);
                day::prefs::set(REFRESH_KEY, &token.refresh_token);
                self.is_authenticated.set(true);
                self.loading.set(false);
                self.load_all_sync();
            }
            Err(e) => {
                self.error_msg.set(format!("Ошибка входа: {e}"));
                self.loading.set(false);
            }
        }
    }

    /// Login with username and password via OAuth (iOS — uses WKWebView).
    #[cfg(target_os = "ios")]
    pub(crate) fn login_with_password(self, username: &str, password: &str) {
        use crate::core::network::auth::Auth;
        use crate::core::network::oauth_web;
        self.loading.set(true);
        self.error_msg.set(String::new());

        let username = username.to_owned();
        let password = password.to_owned();

        std::thread::spawn(move || {
            let auth = Auth::new();
            let result = oauth_web::login_with_web_view(&auth, &username, &password);

            dispatch2::DispatchQueue::main().exec_async(move || {
                match result {
                    Ok(token) => {
                        day::prefs::set(TOKEN_KEY, &token.access_token);
                        day::prefs::set(REFRESH_KEY, &token.refresh_token);
                        self.is_authenticated.set(true);
                        self.loading.set(false);
                        self.load_all_sync();
                    }
                    Err(e) => {
                        self.error_msg.set(format!("Ошибка входа: {e}"));
                        self.loading.set(false);
                    }
                }
            });
        });
    }

    /// Wipe auth state and cached data.
    pub(crate) fn logout(self) {
        for key in [TOKEN_KEY, REFRESH_KEY, SCHOOL_ID_KEY, PROFILE_ID_KEY,
                     CLASS_ID_KEY, FULL_NAME_KEY, SCHOOL_NAME_KEY] {
            day::prefs::set(key, "");
        }
        self.is_authenticated.set(false);
        self.full_name.set(String::new());
        self.school_name.set(String::new());
        self.class_label.set(String::new());
        self.lessons.set(Vec::new());
        self.bell_times.set(Vec::new());
        self.timetable_days.set(Vec::new());
        self.subjects_teachers.set(Vec::new());
    }

    // ── data loading (synchronous — runs on the main thread) ────────────

    /// Load ALL school data synchronously. This blocks the UI briefly but
    /// avoids the `Signal: !Send` constraint entirely.
    fn load_all_sync(self) {
        let token = match day::prefs::get(TOKEN_KEY) {
            Some(t) if !t.is_empty() => t,
            _ => return,
        };

        self.loading.set(true);
        self.error_msg.set(String::new());

        let client = build_client(&token);

        // 1 — user info
        let user = match api_get::<UserInfo>(&client, "/api/v1/admin/auth/me") {
            Ok(u) => u,
            Err(e) => {
                self.error_msg.set(format!("Auth error: {e}"));
                self.loading.set(false);
                return;
            }
        };
        self.full_name.set(user.full_name.clone());
        self.school_name.set(user.school_name.clone());
        day::prefs::set(SCHOOL_ID_KEY, &user.school_id);
        day::prefs::set(PROFILE_ID_KEY, &user.profile_id);
        day::prefs::set(FULL_NAME_KEY, &user.full_name);
        day::prefs::set(SCHOOL_NAME_KEY, &user.school_name);

        // 2 — school year
        let year = api_get::<SchoolYear>(
            &client,
            "/api/v1/education/diary/school_year",
        ).ok();
        let school_period = year.as_ref().map(|y| y.uuid.clone());

        // 3 — classes (POST to discover the student's class)
        // Try with a wide date range to cover the whole school year
        let today = day_piece_datetime::DayDate::today();
        let start_of_year = format!("01.09.{:04}", today.year);
        let end_of_year = format!("31.05.{:04}", today.year + 1);
        let today_str = format!("{:02}.{:02}.{:04}", today.day, today.month, today.year);

        let class_body = serde_json::json!({
            "school_period": school_period,
            "from": start_of_year,
            "to": end_of_year,
        });
        let class_id = if let Ok(cbd) = api_post::<ClassesByDate>(
            &client,
            &format!(
                "/api/v1/education/diary/schools/{}/students/{}/classes",
                user.school_id, user.profile_id
            ),
            &class_body,
        ) {
            if let Some(c) = cbd.classes.first() {
                let id = c.uuid.clone();
                self.class_label.set(format!("{}{}", c.level, c.label));
                day::prefs::set(CLASS_ID_KEY, &id);
                id
            } else {
                // Fallback: try GET endpoint
                if let Ok(classes) = api_get::<Vec<Class>>(
                    &client,
                    &format!(
                        "/api/v1/education/diary/schools/{}/students/{}/classes",
                        user.school_id, user.profile_id
                    ),
                ) {
                    if let Some(c) = classes.first() {
                        let id = c.uuid.clone();
                        self.class_label.set(format!("{}{}", c.level, c.label));
                        day::prefs::set(CLASS_ID_KEY, &id);
                        id
                    } else {
                        day::prefs::get(CLASS_ID_KEY).unwrap_or_default()
                    }
                } else {
                    day::prefs::get(CLASS_ID_KEY).unwrap_or_default()
                }
            }
        } else {
            // POST failed, try GET as fallback
            if let Ok(classes) = api_get::<Vec<Class>>(
                &client,
                &format!(
                    "/api/v1/education/diary/schools/{}/students/{}/classes",
                    user.school_id, user.profile_id
                ),
            ) {
                if let Some(c) = classes.first() {
                    let id = c.uuid.clone();
                    self.class_label.set(format!("{}{}", c.level, c.label));
                    day::prefs::set(CLASS_ID_KEY, &id);
                    id
                } else {
                    day::prefs::get(CLASS_ID_KEY).unwrap_or_default()
                }
            } else {
                day::prefs::get(CLASS_ID_KEY).unwrap_or_default()
            }
        };

        if class_id.is_empty() {
            self.error_msg.set("Class not found — try opening diary.e-schools.by to sync".into());
            self.loading.set(false);
            return;
        }

        // 4 — week activities → current week → lessons
        self.lessons_loading.set(true);
        if let Ok(weeks) = api_get::<Vec<WeekActivity>>(
            &client,
            "/api/v1/education/diary/time_activities/week_activities",
        ) {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let cur = weeks.iter().find(|w| w.start_ts <= now_ms && w.end_ts >= now_ms);

            if let Some(week) = cur {
                self.current_week.set(week.summary.clone());
                if let Ok(lessons) = api_get::<Vec<DaySchedule>>(
                    &client,
                    &format!(
                        "/api/v1/education/diary/schools/{}/classes/{}/students/{}/lessons?week_activity_uuid={}",
                        user.school_id, class_id, user.profile_id, week.uuid
                    ),
                ) {
                    self.lessons.set(lessons);
                }
            }
        }
        self.lessons_loading.set(false);

        // 5 — bell schedule
        self.schedule_loading.set(true);
        if let Ok(bells) = api_get::<Vec<BellSchedule>>(
            &client,
            &format!(
                "/api/v1/education/diary/schools/{}/bells/whole",
                user.school_id
            ),
        ) {
            if let Some(schedule) = bells.first() {
                if let Some(day) = schedule.days_of_week.first() {
                    self.bell_times.set(day.time_of_bells.clone());
                }
            }
        }

        // 6 — timetable
        if let Ok(tables) = api_get::<Vec<Timetable>>(
            &client,
            &format!(
                "/api/v1/education/diary/schools/{}/classes/{}/timetables/whole",
                user.school_id, class_id
            ),
        ) {
            if let Some(t) = tables.first() {
                self.timetable_days.set(t.days_of_week.clone());
            }
        }
        self.schedule_loading.set(false);

        // 7 — subjects with teachers
        self.teachers_loading.set(true);
        if let Ok(subjects) = api_get::<Vec<SubjectWithTeacher>>(
            &client,
            &format!(
                "/api/v1/education/diary/schools/{}/students/{}/classes/{}/subjects",
                user.school_id, user.profile_id, class_id
            ),
        ) {
            self.subjects_teachers.set(subjects);
        }
        self.teachers_loading.set(false);

        self.loading.set(false);
    }
}

// ── private helpers ──────────────────────────────────────────────────────

fn build_client(token: &str) -> reqwest::blocking::Client {
    use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    // e-schools.by uses raw JWT without "Bearer" prefix
    if let Ok(v) = HeaderValue::from_str(token) {
        headers.insert(AUTHORIZATION, v);
    }
    reqwest::blocking::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}

fn api_get<T: serde::de::DeserializeOwned>(
    client: &reqwest::blocking::Client,
    path: &str,
) -> Result<T, String> {
    let url = if path.starts_with("http") {
        path.to_string()
    } else {
        format!("{BASE_URL}{path}")
    };
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    resp.json::<T>()
        .map_err(|e| format!("parse failed: {e}"))
}

fn api_post<T: serde::de::DeserializeOwned>(
    client: &reqwest::blocking::Client,
    path: &str,
    body: &serde_json::Value,
) -> Result<T, String> {
    let url = format!("{BASE_URL}{path}");
    let resp = client
        .post(&url)
        .json(body)
        .send()
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    resp.json::<T>()
        .map_err(|e| format!("parse failed: {e}"))
}

// ── date formatting ─────────────────────────────────────────────────────

/// Weekday name from a 1-based day index (1 = Monday).
pub(crate) fn weekday_name(dow: u32) -> &'static str {
    match dow {
        1 => "Понедельник",
        2 => "Вторник",
        3 => "Среда",
        4 => "Четверг",
        5 => "Пятница",
        6 => "Суббота",
        _ => "Воскресенье",
    }
}

/// Render a timestamp (epoch millis) as "DD.MM".
pub(crate) fn format_date_short(ts: u64) -> String {
    let epoch_days = (ts / 86_400_000) as i64;
    let d = day_piece_datetime::DayDate::from_epoch_days(epoch_days);
    format!("{:02}.{:02}", d.day, d.month)
}

/// Full header: "Понедельник, 07.09".
pub(crate) fn format_date_header(dow: u32, ts: u64) -> String {
    format!("{}, {}", weekday_name(dow), format_date_short(ts))
}

/// Map a numeric mark string (e.g. "8") to a colour.
pub(crate) fn grade_color(mark: &str) -> day::prelude::Color {
    use crate::core::colors;
    match mark.trim().parse::<u32>() {
        Ok(9..=10) => colors::GRADE_EXCELLENT,
        Ok(7..=8)  => colors::GRADE_GOOD,
        Ok(5..=6)  => colors::GRADE_SATISFACTORY,
        Ok(3..=4)  => colors::GRADE_POOR,
        Ok(1..=2)  => colors::GRADE_FAILING,
        _          => colors::GRAY_400,
    }
}
