//! Diary data loading — all school data fetching for the diary, schedule, and teachers.

use crate::app::AppState;
use crate::features::auth;
use eschool_api::client::blocking;
use eschool_api::client::endpoints;
use eschool_api::entities::*;
use crate::shared::nslog;

/// Quarter ranges: weeks indices in all_weeks
const QUARTER_RANGES: &[(usize, usize)] = &[(0, 9), (9, 18), (18, 27), (27, 36)];

/// Load ALL school data synchronously. Blocks the UI briefly.
pub fn load_all(state: AppState) {
    nslog::nslog("[Diary] load_all starting");
    let token = match auth::get_token() {
        Some(t) => t,
        _ => {
            nslog::nslog("[Diary] load_all: no token found");
            return;
        }
    };

    state.loading.set(true);
    state.error_msg.set(String::new());

    let mut client = blocking::build_client(&token);

    // 1 — user info (with auto-refresh on 401)
    nslog::nslog("[Diary] Fetching /auth/me...");
    let user = match blocking::api_get::<UserInfo>(&client, endpoints::AUTH_ME) {
        Ok(u) => {
            nslog::nslog(&format!("[Diary] Got user: {} ({})", u.full_name, u.school_name));
            u
        }
        Err(e) if e.starts_with("HTTP 401") => {
            nslog::nslog("[Diary] /auth/me returned 401, attempting token refresh...");
            if let Some(refreshed) = auth::try_refresh_token() {
                client = blocking::build_client(&refreshed);
                match blocking::api_get::<UserInfo>(&client, endpoints::AUTH_ME) {
                    Ok(u) => u,
                    Err(e) => {
                        state.error_msg.set(format!("Auth error: {e}"));
                        state.loading.set(false);
                        return;
                    }
                }
            } else {
                state.error_msg.set("Сессия истекла. Войдите снова.".into());
                state.loading.set(false);
                auth::logout(state);
                return;
            }
        }
        Err(e) => {
            state.error_msg.set(format!("Auth error: {e}"));
            state.loading.set(false);
            return;
        }
    };
    state.full_name.set(user.full_name.clone());
    state.school_name.set(user.school_name.clone());

    // 2 — school year
    let year = blocking::api_get::<SchoolYear>(&client, endpoints::SCHOOL_YEAR).ok();
    let school_period = year.as_ref().map(|y| y.uuid.clone());

    // 3 — classes
    let today = day_piece_datetime::DayDate::today();
    let start_of_year = format!("01.09.{:04}", today.year);
    let end_of_year = format!("31.05.{:04}", today.year + 1);

    let class_body = serde_json::json!({
        "school_period": school_period,
        "from": start_of_year,
        "to": end_of_year,
    });
    let class_id = if let Ok(cbd) = blocking::api_post::<ClassesByDate>(
        &client,
        &endpoints::classes(&user.school_id, &user.profile_id),
        &class_body,
    ) {
        if let Some(c) = cbd.classes.first() {
            let id = c.uuid.clone();
            state.class_label.set(format!("{}{}", c.level, c.label));
            state.is_graduating.set(c.graduating.unwrap_or(false));
            id
        } else {
            fallback_class_id(&client, &user.school_id, &user.profile_id, state)
        }
    } else {
        fallback_class_id(&client, &user.school_id, &user.profile_id, state)
    };

    if class_id.is_empty() {
        state.error_msg.set("Class not found — try opening diary.e-schools.by to sync".into());
        state.loading.set(false);
        return;
    }

    auth::store_ids(
        &user.school_id,
        &class_id,
        &user.profile_id,
        &user.full_name,
        &user.school_name,
    );

    // 4 — week activities → current week → lessons
    load_weeks_and_current(state, &client, &user.school_id, &class_id, &user.profile_id);

    // 5 — bell schedule
    load_bells(state, &client, &user.school_id);

    // 6 — timetable
    load_timetable(state, &client, &user.school_id, &class_id);

    // 7 — subjects with teachers
    load_teachers(state, &client, &user.school_id, &user.profile_id, &class_id);

    state.loading.set(false);
}

fn fallback_class_id(
    client: &reqwest::blocking::Client,
    school_id: &str,
    profile_id: &str,
    state: AppState,
) -> String {
    if let Ok(classes) = blocking::api_get::<Vec<Class>>(
        client,
        &endpoints::classes(school_id, profile_id),
    ) {
        if let Some(c) = classes.first() {
            let id = c.uuid.clone();
            state.class_label.set(format!("{}{}", c.level, c.label));
            state.is_graduating.set(c.graduating.unwrap_or(false));
            id
        } else {
            day::prefs::get("auth.class_id").unwrap_or_default()
        }
    } else {
        day::prefs::get("auth.class_id").unwrap_or_default()
    }
}

fn load_weeks_and_current(
    state: AppState,
    client: &reqwest::blocking::Client,
    school_id: &str,
    class_id: &str,
    profile_id: &str,
) {
    state.lessons_loading.set(true);
    match blocking::api_get::<Vec<WeekActivity>>(client, endpoints::WEEK_ACTIVITIES) {
        Ok(weeks) => {
            nslog::nslog(&format!("[Diary] Got {} weeks", weeks.len()));
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let cur = weeks.iter().enumerate()
                .find(|(_, w)| w.start_ts <= now_ms && w.end_ts >= now_ms);

            if let Some((idx, week)) = cur {
                let week_uuid = week.uuid.clone();
                let week_summary = week.summary.clone();
                state.all_weeks.set(weeks);
                state.current_week_index.set(idx as i32);
                state.current_week.set(week_summary);
                state.current_quarter.set(quarter_for_index(idx));

                // Load first week of current quarter
                let q = quarter_for_index(idx);
                let (start, _) = QUARTER_RANGES[q];
                load_week(state, start as i32);
            }
        }
        Err(e) => nslog::nslog(&format!("[Diary] Week activities failed: {e}")),
    }
    state.lessons_loading.set(false);
}

fn load_bells(state: AppState, client: &reqwest::blocking::Client, school_id: &str) {
    state.schedule_loading.set(true);
    if let Ok(bells) = blocking::api_get::<Vec<BellSchedule>>(client, &endpoints::bells(school_id)) {
        if let Some(schedule) = bells.first() {
            if let Some(day) = schedule.days_of_week.first() {
                state.bell_times.set(day.time_of_bells.clone());
            }
        }
    }
    state.schedule_loading.set(false);
}

fn load_timetable(state: AppState, client: &reqwest::blocking::Client, school_id: &str, class_id: &str) {
    if let Ok(tables) = blocking::api_get::<Vec<Timetable>>(client, &endpoints::timetable(school_id, class_id)) {
        if let Some(t) = tables.first() {
            state.timetable_days.set(t.days_of_week.clone());
        }
    }
}

fn load_teachers(state: AppState, client: &reqwest::blocking::Client, school_id: &str, profile_id: &str, class_id: &str) {
    state.teachers_loading.set(true);
    if let Ok(subjects) = blocking::api_get::<Vec<SubjectWithTeacher>>(
        client,
        &endpoints::subjects(school_id, profile_id, class_id),
    ) {
        state.subjects_teachers.set(subjects);
    }
    state.teachers_loading.set(false);
}

/// Load lessons for a specific week by index in all_weeks.
pub fn load_week(state: AppState, new_index: i32) {
    let weeks = state.all_weeks.get();
    let idx = new_index as usize;
    if idx >= weeks.len() { return; }

    let (school_id, class_id, profile_id) = auth::get_stored_ids();
    if school_id.is_empty() || class_id.is_empty() || profile_id.is_empty() { return; }

    let token = match auth::get_token() {
        Some(t) => t,
        _ => return,
    };
    let client = blocking::build_client(&token);

    let week = &weeks[idx];
    state.current_week_index.set(idx as i32);
    state.current_week.set(week.summary.clone());
    state.current_quarter.set(quarter_for_index(idx));
    state.lessons_loading.set(true);

    match blocking::api_get_raw(&client, &endpoints::lessons(&school_id, &class_id, &profile_id, &week.uuid)) {
        Ok(raw) => {
            match serde_json::from_str::<Vec<DaySchedule>>(&raw) {
                Ok(lessons) => {
                    state.lessons.set(lessons);
                }
                Err(e) => nslog::nslog(&format!("[Diary] load_week parse failed: {e}")),
            }
        }
        Err(e) => nslog::nslog(&format!("[Diary] load_week request failed: {e}")),
    }
    state.lessons_loading.set(false);
}

/// Load all weeks for a quarter and accumulate marks.
pub fn load_quarter(state: AppState, quarter: usize) {
    if quarter > 3 { return; }

    let weeks = state.all_weeks.get();
    let (start, end) = QUARTER_RANGES[quarter];
    if start >= weeks.len() { return; }

    let (school_id, class_id, profile_id) = auth::get_stored_ids();
    if school_id.is_empty() || class_id.is_empty() || profile_id.is_empty() { return; }

    let token = match auth::get_token() {
        Some(t) => t,
        _ => return,
    };
    let client = blocking::build_client(&token);

    state.current_quarter.set(quarter);
    state.marks_loading.set(true);
    state.quarter_marks.set(std::collections::HashMap::new());

    let mut all_marks: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();

    let actual_end = end.min(weeks.len());
    for idx in start..actual_end {
        let week = &weeks[idx];
        match blocking::api_get_raw(&client, &endpoints::lessons(&school_id, &class_id, &profile_id, &week.uuid)) {
            Ok(raw) => {
                if let Ok(days) = serde_json::from_str::<Vec<DaySchedule>>(&raw) {
                    for day in &days {
                        for slot in &day.slots {
                            if let Some(mark_val) = slot.lesson_mark.as_ref()
                                .and_then(|m| m.mark.as_ref())
                                .and_then(|m| m.parse::<f64>().ok())
                            {
                                all_marks.entry(slot.subject_title.clone())
                                    .or_default()
                                    .push(mark_val);
                            }
                        }
                    }
                }
            }
            Err(e) => nslog::nslog(&format!("[Diary] load_quarter week {idx} failed: {e}")),
        }
    }

    state.quarter_marks.set(all_marks);
    state.marks_loading.set(false);
    nslog::nslog(&format!("[Diary] Quarter {} loaded", quarter + 1));
}

/// Load all weeks for the entire year, storing marks per-quarter and total.
pub fn load_year(state: AppState) {
    let weeks = state.all_weeks.get();
    if weeks.is_empty() { return; }

    let (school_id, class_id, profile_id) = auth::get_stored_ids();
    if school_id.is_empty() || class_id.is_empty() || profile_id.is_empty() { return; }

    let token = match auth::get_token() {
        Some(t) => t,
        _ => return,
    };
    let client = blocking::build_client(&token);

    state.current_quarter.set(4); // 4 = year
    state.marks_loading.set(true);
    state.quarter_marks.set(std::collections::HashMap::new());
    state.year_quarter_data.set(Vec::new());

    let quarter_labels = ["I четверть", "II четверть", "III четверть", "IV четверть"];
    let mut year_data: Vec<(String, std::collections::HashMap<String, Vec<f64>>)> = Vec::new();
    let mut all_marks: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();

    for q in 0..4 {
        let (start, end) = QUARTER_RANGES[q];
        let actual_end = end.min(weeks.len());
        let mut q_marks: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();

        for idx in start..actual_end {
            let week = &weeks[idx];
            match blocking::api_get_raw(&client, &endpoints::lessons(&school_id, &class_id, &profile_id, &week.uuid)) {
                Ok(raw) => {
                    if let Ok(days) = serde_json::from_str::<Vec<DaySchedule>>(&raw) {
                        for day in &days {
                            for slot in &day.slots {
                                if let Some(mark_val) = slot.lesson_mark.as_ref()
                                    .and_then(|m| m.mark.as_ref())
                                    .and_then(|m| m.parse::<f64>().ok())
                                {
                                    q_marks.entry(slot.subject_title.clone())
                                        .or_default()
                                        .push(mark_val);
                                    all_marks.entry(slot.subject_title.clone())
                                        .or_default()
                                        .push(mark_val);
                                }
                            }
                        }
                    }
                }
                Err(e) => nslog::nslog(&format!("[Diary] load_year q{} week {idx} failed: {e}", q + 1)),
            }
        }

        year_data.push((quarter_labels[q].to_string(), q_marks));
    }

    state.year_quarter_data.set(year_data);
    state.quarter_marks.set(all_marks);
    state.marks_loading.set(false);
    nslog::nslog("[Diary] Year loaded with per-quarter data");
}

/// Extract numeric marks from lessons and append to all_marks (legacy).
fn update_marks(state: AppState, lessons: &[DaySchedule], week_uuid: &str) {
    let mut loaded = state.loaded_mark_weeks.get();
    if loaded.contains(week_uuid) {
        return;
    }
    loaded.insert(week_uuid.to_string());
    state.loaded_mark_weeks.set(loaded);

    let new_marks: Vec<(String, f64)> = lessons.iter()
        .flat_map(|d| d.slots.iter())
        .filter_map(|s| {
            let mark_val = s.lesson_mark.as_ref()?
                .mark.as_ref()?
                .parse::<f64>().ok()?;
            Some((s.subject_title.clone(), mark_val))
        })
        .collect();
    let mut marks = state.all_marks.get();
    marks.extend(new_marks);
    state.all_marks.set(marks);
}

/// Reset marks when switching quarters.
pub fn reset_marks(state: AppState) {
    state.all_marks.set(Vec::new());
    state.loaded_mark_weeks.set(std::collections::HashSet::new());
    state.quarter_marks.set(std::collections::HashMap::new());
}

/// Get week indices for a quarter.
pub fn quarter_week_indices(quarter: usize) -> std::ops::Range<usize> {
    let (start, end) = QUARTER_RANGES[quarter];
    start..end
}

/// Determine which quarter a week index belongs to.
pub fn quarter_for_index(idx: usize) -> usize {
    if idx < 9 { 0 } else if idx < 18 { 1 } else if idx < 27 { 2 } else { 3 }
}
