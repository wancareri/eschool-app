//! Diary data loading — all school data fetching for the diary, schedule, and teachers.

use crate::app::AppState;
use crate::app::OfficialMark;
use crate::features::auth;
use eschool_api::client::async_client;
use eschool_api::client::endpoints;
use eschool_api::entities::*;
use crate::shared::nslog;

/// Quarter ranges: weeks indices in all_weeks
const QUARTER_RANGES: &[(usize, usize)] = &[(0, 9), (9, 18), (18, 27), (27, 36)];

/// Load ALL school data asynchronously. Does NOT block the UI.
pub fn load_all(state: AppState) {
    nslog::nslog("[Diary] load_all starting");
    let token = match auth::get_token() {
        Some(t) => t,
        _ => {
            nslog::nslog("[Diary] load_all: no token found");
            return;
        }
    };
    nslog::nslog(&format!("[Diary] Token length: {}", token.len()));

    state.loading.set(true);
    state.error_msg.set(String::new());

    day::task(async move {
        let mut client = async_client::build_client(&token);

        // 1 — user info (with auto-refresh on 401)
        nslog::nslog("[Diary] Fetching /auth/me...");
        let user = match async_client::api_get::<UserInfo>(&client, endpoints::AUTH_ME).await {
            Ok(u) => {
                nslog::nslog(&format!("[Diary] Got user: {} ({})", u.full_name, u.school_name));
                u
            }
            Err(e) if e.starts_with("HTTP 401") => {
                nslog::nslog("[Diary] /auth/me returned 401, attempting token refresh...");
                if let Some(refreshed) = auth::try_refresh_token() {
                    client = async_client::build_client(&refreshed);
                    match async_client::api_get::<UserInfo>(&client, endpoints::AUTH_ME).await {
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
        let year = async_client::api_get::<SchoolYear>(&client, endpoints::SCHOOL_YEAR).await.ok();
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
        let class_id = if let Ok(cbd) = async_client::api_post::<ClassesByDate>(
            &client,
            &endpoints::classes(&user.school_id, &user.profile_id),
            &class_body,
        ).await {
            if let Some(c) = cbd.classes.first() {
                let id = c.uuid.clone();
                state.class_label.set(format!("{}{}", c.level, c.label));
                state.is_graduating.set(c.graduating.unwrap_or(false));
                id
            } else {
                fallback_class_id(&client, &user.school_id, &user.profile_id, state).await
            }
        } else {
            fallback_class_id(&client, &user.school_id, &user.profile_id, state).await
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

        // Initialize per-quarter storage
        {
            let mut q_all = state.quarter_all_marks.get();
            let mut q_off = state.quarter_official_marks.get();
            q_all.resize(4, std::collections::HashMap::new());
            q_off.resize(4, std::collections::HashMap::new());
            state.quarter_all_marks.set(q_all);
            state.quarter_official_marks.set(q_off);
        }

        // 4 — week activities → current week → lessons
        load_weeks_and_current(state, &client, &user.school_id, &class_id, &user.profile_id).await;

        // 5 — bell schedule
        load_bells(state, &client, &user.school_id).await;

        // 6 — timetable
        load_timetable(state, &client, &user.school_id, &class_id).await;

        // 7 — subjects with teachers
        load_teachers(state, &client, &user.school_id, &user.profile_id, &class_id).await;

        state.loading.set(false);
        nslog::nslog("[Diary] load_all completed");
    });
}

async fn fallback_class_id(
    client: &reqwest::Client,
    school_id: &str,
    profile_id: &str,
    state: AppState,
) -> String {
    if let Ok(classes) = async_client::api_get::<Vec<Class>>(
        client,
        &endpoints::classes(school_id, profile_id),
    ).await {
        if let Some(c) = classes.first() {
            let id = c.uuid.clone();
            state.class_label.set(format!("{}{}", c.level, c.label));
            state.is_graduating.set(c.graduating.unwrap_or(false));
            id
        } else {
            crate::shared::secure::load("auth.class_id").unwrap_or_default()
        }
    } else {
        crate::shared::secure::load("auth.class_id").unwrap_or_default()
    }
}

async fn load_weeks_and_current(
    state: AppState,
    client: &reqwest::Client,
    _school_id: &str,
    _class_id: &str,
    _profile_id: &str,
) {
    state.lessons_loading.set(true);
    match async_client::api_get::<Vec<WeekActivity>>(client, endpoints::WEEK_ACTIVITIES).await {
        Ok(weeks) => {
            nslog::nslog(&format!("[Diary] Got {} weeks", weeks.len()));
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let cur = weeks.iter().enumerate()
                .find(|(_, w)| w.start_ts <= now_ms && w.end_ts >= now_ms);

            if let Some((idx, week)) = cur {
                let _week_uuid = week.uuid.clone();
                let week_summary = week.summary.clone();
                state.all_weeks.set(weeks);
                state.current_week_index.set(idx as i32);
                state.current_week.set(week_summary);
                state.current_quarter.set(quarter_for_index(idx));

                // Load current week
                load_week(state, idx as i32);
            }
        }
        Err(e) => nslog::nslog(&format!("[Diary] Week activities failed: {e}")),
    }
    state.lessons_loading.set(false);
}

async fn load_bells(state: AppState, client: &reqwest::Client, school_id: &str) {
    state.schedule_loading.set(true);
    if let Ok(bells) = async_client::api_get::<Vec<BellSchedule>>(client, &endpoints::bells(school_id)).await {
        if let Some(schedule) = bells.first() {
            if let Some(day) = schedule.days_of_week.first() {
                state.bell_times.set(day.time_of_bells.clone());
            }
        }
    }
    state.schedule_loading.set(false);
}

async fn load_timetable(state: AppState, client: &reqwest::Client, school_id: &str, class_id: &str) {
    if let Ok(tables) = async_client::api_get::<Vec<Timetable>>(client, &endpoints::timetable(school_id, class_id)).await {
        if let Some(t) = tables.first() {
            state.timetable_days.set(t.days_of_week.clone());
        }
    }
}

async fn load_teachers(state: AppState, client: &reqwest::Client, school_id: &str, profile_id: &str, class_id: &str) {
    state.teachers_loading.set(true);
    if let Ok(subjects) = async_client::api_get::<Vec<SubjectWithTeacher>>(
        client,
        &endpoints::subjects(school_id, profile_id, class_id),
    ).await {
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

    let week = &weeks[idx];
    state.current_week_index.set(idx as i32);
    state.current_week.set(week.summary.clone());
    state.current_quarter.set(quarter_for_index(idx));
    state.lessons_loading.set(true);

    let client = async_client::build_client(&token);
    let week_uuid = week.uuid.clone();

    day::task(async move {
        match async_client::api_get_raw(&client, &endpoints::lessons(&school_id, &class_id, &profile_id, &week_uuid)).await {
            Ok(raw) => {
                match serde_json::from_str::<Vec<DaySchedule>>(&raw) {
                    Ok(lessons) => {
                        state.lessons.set(lessons);
                        crate::shared::widget::update_widget_data(state);
                    }
                    Err(e) => {
                        nslog::nslog(&format!("[Diary] load_week parse failed: {e}"));
                        let col = e.column();
                        if col > 0 && col < raw.len() {
                            let start = col.saturating_sub(200);
                            let end = (col + 200).min(raw.len());
                            nslog::nslog(&format!("[Diary] JSON around col {}: ...{}...", col, &raw[start..end]));
                        }
                    }
                }
            }
            Err(e) => nslog::nslog(&format!("[Diary] load_week request failed: {e}")),
        }
        state.lessons_loading.set(false);
    });
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

    state.current_quarter.set(quarter);
    state.marks_loading.set(true);

    // Pre-initialize this quarter's storage so the UI shows loading state
    {
        let mut q_all = state.quarter_all_marks.get();
        let mut q_off = state.quarter_official_marks.get();
        if q_all.len() <= quarter {
            q_all.resize(quarter + 1, std::collections::HashMap::new());
            q_off.resize(quarter + 1, std::collections::HashMap::new());
        }
        state.quarter_all_marks.set(q_all);
        state.quarter_official_marks.set(q_off);
    }

    let client = async_client::build_client(&token);
    let actual_end = end.min(weeks.len());
    let week_uuids: Vec<String> = (start..actual_end)
        .filter_map(|idx| weeks.get(idx).map(|w| w.uuid.clone()))
        .collect();

    day::task(async move {
        let mut all_marks: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
        let mut all_official: std::collections::HashMap<String, Vec<OfficialMark>> = std::collections::HashMap::new();

        for week_uuid in &week_uuids {
            match async_client::api_get_raw(&client, &endpoints::lessons(&school_id, &class_id, &profile_id, week_uuid)).await {
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
                                    if let Some(ref lm) = slot.lesson_mark {
                                        let has_author = lm.author.as_ref()
                                            .map(|a| !a.is_empty())
                                            .unwrap_or(false);
                                        if has_author {
                                            all_official.entry(slot.subject_title.clone())
                                                .or_default()
                                                .push(OfficialMark {
                                                    value: mark_val,
                                                    kind: lm.kind.clone().unwrap_or_default(),
                                                    author: lm.author.clone().unwrap_or_default(),
                                                });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => nslog::nslog(&format!("[Diary] load_quarter week failed: {e}")),
            }
        }

        // Store per-quarter
        {
            let mut q_all = state.quarter_all_marks.get();
            let mut q_off = state.quarter_official_marks.get();
            if q_all.len() <= quarter {
                q_all.resize(quarter + 1, std::collections::HashMap::new());
                q_off.resize(quarter + 1, std::collections::HashMap::new());
            }
            q_all[quarter] = all_marks.clone();
            q_off[quarter] = all_official.clone();
            state.quarter_all_marks.set(q_all);
            state.quarter_official_marks.set(q_off);
        }

        state.quarter_marks.set(all_marks);
        state.official_marks.set(all_official);
        state.marks_loading.set(false);
        nslog::nslog(&format!("[Diary] Quarter {} loaded", quarter + 1));
    });
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

    state.current_quarter.set(4);
    state.marks_loading.set(true);

    let client = async_client::build_client(&token);
    let quarter_labels = ["I четверть", "II четверть", "III четверть", "IV четверть"];

    let mut q_weeks: Vec<Vec<String>> = Vec::new();
    for q in 0..4 {
        let (start, end) = QUARTER_RANGES[q];
        let actual_end = end.min(weeks.len());
        let uuids: Vec<String> = (start..actual_end)
            .filter_map(|idx| weeks.get(idx).map(|w| w.uuid.clone()))
            .collect();
        q_weeks.push(uuids);
    }

    day::task(async move {
        let mut year_data: Vec<(String, std::collections::HashMap<String, Vec<f64>>)> = Vec::new();
        let mut all_marks: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
        let mut all_official: std::collections::HashMap<String, Vec<OfficialMark>> = std::collections::HashMap::new();

        let mut q_all_marks: Vec<std::collections::HashMap<String, Vec<f64>>> = Vec::new();
        let mut q_official_marks: Vec<std::collections::HashMap<String, Vec<OfficialMark>>> = Vec::new();

        for q in 0..4 {
            let mut q_marks: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
            let mut q_off: std::collections::HashMap<String, Vec<OfficialMark>> = std::collections::HashMap::new();

            for week_uuid in &q_weeks[q] {
                match async_client::api_get_raw(&client, &endpoints::lessons(&school_id, &class_id, &profile_id, week_uuid)).await {
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
                                        if let Some(ref lm) = slot.lesson_mark {
                                            let has_author = lm.author.as_ref()
                                                .map(|a| !a.is_empty())
                                                .unwrap_or(false);
                                            if has_author {
                                                let om = OfficialMark {
                                                    value: mark_val,
                                                    kind: lm.kind.clone().unwrap_or_default(),
                                                    author: lm.author.clone().unwrap_or_default(),
                                                };
                                                q_off.entry(slot.subject_title.clone())
                                                    .or_default()
                                                    .push(om.clone());
                                                all_official.entry(slot.subject_title.clone())
                                                    .or_default()
                                                    .push(om);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => nslog::nslog(&format!("[Diary] load_year q{} week failed: {e}", q + 1)),
                }
            }

            year_data.push((quarter_labels[q].to_string(), q_marks.clone()));
            q_all_marks.push(q_marks);
            q_official_marks.push(q_off);
        }

        state.quarter_all_marks.set(q_all_marks);
        state.quarter_official_marks.set(q_official_marks);

        state.year_quarter_data.set(year_data);
        state.quarter_marks.set(all_marks);
        state.official_marks.set(all_official);
        state.marks_loading.set(false);
        nslog::nslog("[Diary] Year loaded with per-quarter data");
    });
}

/// Reset marks when switching quarters.
pub fn reset_marks(state: AppState) {
    state.all_marks.set(Vec::new());
    state.loaded_mark_weeks.set(std::collections::HashSet::new());
    state.quarter_marks.set(std::collections::HashMap::new());
    state.quarter_all_marks.set(Vec::new());
    state.quarter_official_marks.set(Vec::new());
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
