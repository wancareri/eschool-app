//! Diary data loading — all school data fetching for the diary, schedule, and teachers.

use crate::app::AppState;
use crate::app::OfficialMark;
use crate::features::auth;
use eschool_api::client::blocking;
use eschool_api::client::endpoints;
use eschool_api::entities::*;
use crate::shared::{cache, nslog, utils};

/// Quarter ranges: weeks indices in all_weeks
const QUARTER_RANGES: &[(usize, usize)] = &[(0, 9), (9, 18), (18, 27), (27, 36)];

/// Load ALL school data on a background thread. UI stays responsive.
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

    let set_loading = state.loading.setter();
    let set_error = state.error_msg.setter();
    let set_full_name = state.full_name.setter();
    let set_school_name = state.school_name.setter();
    let set_class_label = state.class_label.setter();
    let set_is_graduating = state.is_graduating.setter();
    let set_all_weeks = state.all_weeks.setter();
    let set_current_week_index = state.current_week_index.setter();
    let set_current_week = state.current_week.setter();
    let set_current_quarter = state.current_quarter.setter();
    let set_bell_times = state.bell_times.setter();
    let set_timetable_days = state.timetable_days.setter();
    let set_subjects_teachers = state.subjects_teachers.setter();
    let set_lessons = state.lessons.setter();
    let set_quarter_all_marks = state.quarter_all_marks.setter();
    let set_quarter_official_marks = state.quarter_official_marks.setter();
    let set_lessons_loading = state.lessons_loading.setter();
    let set_is_authenticated = state.is_authenticated.setter();
    let set_conn = state.conn_status.setter();

    set_loading.set(true);
    set_error.set(String::new());

    // Load cached data immediately so UI shows stale data while refreshing
    let mut cached_cur_idx: Option<i32> = None;
    if let Some(cached_weeks) = cache::load_json::<Vec<WeekActivity>>("all_weeks") {
        nslog::nslog("[Cache] Loading cached weeks");
        set_all_weeks.set(cached_weeks.clone());
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        if let Some((idx, w)) = cached_weeks.iter().enumerate()
            .find(|(_, w)| w.start_ts <= now_ms && w.end_ts >= now_ms)
        {
            set_current_week_index.set(idx as i32);
            set_current_week.set(w.summary.clone());
            set_current_quarter.set(quarter_for_index(idx));
            cached_cur_idx = Some(idx as i32);
        }
    }
    if let Some(cached_wc) = cache::load_json::<std::collections::HashMap<i32, Vec<DaySchedule>>>("week_cache") {
        nslog::nslog(&format!("[Cache] Loading cached week_cache ({} weeks)", cached_wc.len()));
        if let Some(ci) = cached_cur_idx {
            if let Some(ls) = cached_wc.get(&ci) {
                set_lessons.set(ls.clone());
            }
        }
        state.week_cache.set(cached_wc);
    }
    if let Some(cached_lessons) = cache::load_json::<Vec<DaySchedule>>("lessons") {
        nslog::nslog("[Cache] Loading cached lessons");
        if state.lessons.get().is_empty() {
            set_lessons.set(cached_lessons.clone());
            if let Some(ci) = cached_cur_idx {
                let mut wc = state.week_cache.get();
                wc.insert(ci, cached_lessons);
                state.week_cache.set(wc);
            }
        }
    }
    if let Some(cached_bells) = cache::load_json::<Vec<BellTime>>("bell_times") {
        nslog::nslog("[Cache] Loading cached bells");
        set_bell_times.set(cached_bells);
    }
    if let Some(cached_timetable) = cache::load_json::<Vec<TimetableDay>>("timetable_days") {
        nslog::nslog("[Cache] Loading cached timetable");
        set_timetable_days.set(cached_timetable);
    }
    if let Some(cached_teachers) = cache::load_json::<Vec<SubjectWithTeacher>>("subjects_teachers") {
        nslog::nslog("[Cache] Loading cached teachers");
        set_subjects_teachers.set(cached_teachers);
    }
    let cur_q = state.current_quarter.get();
    if let Some(cached_q_all) = cache::load_json::<Vec<std::collections::HashMap<String, Vec<f64>>>>("quarter_all_marks") {
        if let Some(m) = cached_q_all.get(cur_q) {
            state.quarter_marks.set(m.clone());
        }
        set_quarter_all_marks.clone().set(cached_q_all);
    }
    if let Some(cached_q_off) = cache::load_json::<Vec<std::collections::HashMap<String, Vec<OfficialMark>>>>("quarter_official_marks") {
        if let Some(m) = cached_q_off.get(cur_q) {
            state.official_marks.set(m.clone());
        }
        set_quarter_official_marks.clone().set(cached_q_off);
    }
    if let Some(name) = cache::load("user_full_name") { set_full_name.set(name); }
    if let Some(school) = cache::load("user_school_name") { set_school_name.set(school); }

    set_conn.set(crate::app::ConnStatus::Connecting);

    let set_conn2 = state.conn_status.setter();
    std::thread::spawn(move || {
        set_conn2.set(crate::app::ConnStatus::Connecting);
        let mut client = blocking::build_client(&token);

        // 1 — user info (with auto-refresh on 401)
        nslog::nslog("[Diary] Fetching /auth/me...");
        let user = match blocking::api_get::<UserInfo>(&client, endpoints::AUTH_ME) {
            Ok(u) => {
                nslog::nslog(&format!("[Diary] Got user: {} ({})", u.full_name, u.school_name));
                u
            }
            Err(e) if e.starts_with("HTTP 401") => {
                nslog::nslog("[Diary] /auth/me 401, refreshing...");
                if let Some(refreshed) = auth::try_refresh_token() {
                    client = blocking::build_client(&refreshed);
                    match blocking::api_get::<UserInfo>(&client, endpoints::AUTH_ME) {
                        Ok(u) => u,
                        Err(e) => {
                            set_error.set(format!("Auth error: {e}"));
                            set_loading.set(false);
                            set_conn2.set(crate::app::ConnStatus::Error);
                            return;
                        }
                    }
                } else {
                    set_error.set("Сессия истекла. Войдите снова.".into());
                    set_loading.set(false);
                    set_conn2.set(crate::app::ConnStatus::Error);
                    // Inline logout using setters (AppState is !Send, can't move into thread)
                    auth::logout_keys();
                    set_is_authenticated.set(false);
                    return;
                }
            }
            Err(e) => {
                set_error.set(format!("Auth error: {e}"));
                set_loading.set(false);
                set_conn2.set(crate::app::ConnStatus::Error);
                return;
            }
        };
        set_full_name.set(user.full_name.clone());
        set_school_name.set(user.school_name.clone());
        cache::save("user_full_name", &user.full_name);
        cache::save("user_school_name", &user.school_name);

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
                set_class_label.set(format!("{}{}", c.level, c.label));
                set_is_graduating.set(c.graduating.unwrap_or(false));
                id
            } else {
                fallback_class_id(&client, &user.school_id, &user.profile_id)
            }
        } else {
            fallback_class_id(&client, &user.school_id, &user.profile_id)
        };

        if class_id.is_empty() {
            set_error.set("Class not found — try opening diary.e-schools.by to sync".into());
            set_loading.set(false);
            set_conn2.set(crate::app::ConnStatus::Error);
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
            let mut q_all: Vec<std::collections::HashMap<String, Vec<f64>>> = Vec::new();
            let mut q_off: Vec<std::collections::HashMap<String, Vec<OfficialMark>>> = Vec::new();
            q_all.resize(4, std::collections::HashMap::new());
            q_off.resize(4, std::collections::HashMap::new());
            set_quarter_all_marks.clone().set(q_all);
            set_quarter_official_marks.clone().set(q_off);
        }

        // 4 — week activities → current week → lessons
        nslog::nslog("[Diary] Loading week activities...");
        set_lessons_loading.clone().set(true);
        match blocking::api_get::<Vec<WeekActivity>>(&client, endpoints::WEEK_ACTIVITIES) {
            Ok(weeks) => {
                nslog::nslog(&format!("[Diary] Got {} weeks", weeks.len()));
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                let cur = weeks.iter().enumerate()
                    .find(|(_, w)| w.start_ts <= now_ms && w.end_ts >= now_ms);

                set_all_weeks.set(weeks.clone());
                cache::save_json("all_weeks", &weeks);

                if let Some((idx, week)) = cur {
                    let week_summary = week.summary.clone();
                    set_current_week_index.set(idx as i32);
                    set_current_week.set(week_summary);
                    set_current_quarter.set(quarter_for_index(idx));

                    // Load current week lessons
                    let school_id = user.school_id.clone();
                    let cid = class_id.clone();
                    let pid = user.profile_id.clone();
                    let week_uuid = week.uuid.clone();
                    match blocking::api_get_raw(&client, &endpoints::lessons(&school_id, &cid, &pid, &week_uuid)) {
                        Ok(raw) => {
                            match serde_json::from_str::<Vec<DaySchedule>>(&raw) {
                                Ok(lessons) => {
                                    set_lessons.set(lessons.clone());
                                    cache::save_json("lessons", &lessons);
                                    let ci = idx as i32;
                                    let ls = lessons;
                                    day::reactive::on_main(move || {
                                        if let Some(state) = AppState::get_main() {
                                            let mut wc = state.week_cache.get();
                                            wc.insert(ci, ls);
                                            cache::save_json("week_cache", &wc);
                                            state.week_cache.set(wc);
                                        }
                                    });

                                    // Prefetch neighbor weeks (idx - 1 and idx + 1) immediately on startup
                                    for di in [-1i32, 1i32] {
                                        let ni = idx as i32 + di;
                                        if ni >= 0 && (ni as usize) < weeks.len() {
                                            let nuuid = weeks[ni as usize].uuid.clone();
                                            if let Ok(raw) = blocking::api_get_raw(&client, &endpoints::lessons(&school_id, &cid, &pid, &nuuid)) {
                                                if let Ok(nls) = serde_json::from_str::<Vec<DaySchedule>>(&raw) {
                                                    nslog::nslog(&format!("[Diary] startup prefetch week idx={ni} OK ({} days)", nls.len()));
                                                    day::reactive::on_main(move || {
                                                        if let Some(state) = AppState::get_main() {
                                                            let mut wc = state.week_cache.get();
                                                            wc.insert(ni, nls);
                                                            cache::save_json("week_cache", &wc);
                                                            state.week_cache.set(wc);
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => nslog::nslog(&format!("[Diary] parse failed: {e}")),
                            }
                        }
                        Err(e) => nslog::nslog(&format!("[Diary] lessons request failed: {e}")),
                    }
                }
            }
            Err(e) => nslog::nslog(&format!("[Diary] Week activities failed: {e}")),
        }
        set_lessons_loading.set(false);

        // 5 — bell schedule
        nslog::nslog("[Diary] Loading bells...");
        if let Ok(bells) = blocking::api_get::<Vec<BellSchedule>>(&client, &endpoints::bells(&user.school_id)) {
            if let Some(schedule) = bells.first() {
                if let Some(day) = schedule.days_of_week.first() {
                    set_bell_times.set(day.time_of_bells.clone());
                    cache::save_json("bell_times", &day.time_of_bells);
                }
            }
        }

        // 6 — timetable
        if let Ok(tables) = blocking::api_get::<Vec<Timetable>>(&client, &endpoints::timetable(&user.school_id, &class_id)) {
            if let Some(t) = tables.first() {
                set_timetable_days.set(t.days_of_week.clone());
                cache::save_json("timetable_days", &t.days_of_week);
            }
        }

        // 7 — subjects with teachers
        nslog::nslog("[Diary] Loading teachers...");
        if let Ok(subjects) = blocking::api_get::<Vec<SubjectWithTeacher>>(
            &client,
            &endpoints::subjects(&user.school_id, &user.profile_id, &class_id),
        ) {
            set_subjects_teachers.set(subjects.clone());
            cache::save_json("subjects_teachers", &subjects);
        }

        set_loading.set(false);
        set_conn2.set(crate::app::ConnStatus::Connected);
        nslog::nslog("[Diary] load_all completed");

        // Auto-hide the "Connected" indicator after 2 seconds
        std::thread::sleep(std::time::Duration::from_secs(2));
        set_conn2.set(crate::app::ConnStatus::Idle);
    });
}

fn fallback_class_id(
    client: &reqwest::blocking::Client,
    school_id: &str,
    profile_id: &str,
) -> String {
    if let Ok(classes) = blocking::api_get::<Vec<Class>>(
        client,
        &endpoints::classes(school_id, profile_id),
    ) {
        if let Some(c) = classes.first() {
            c.uuid.clone()
        } else {
            crate::shared::secure::load("auth.class_id").unwrap_or_default()
        }
    } else {
        crate::shared::secure::load("auth.class_id").unwrap_or_default()
    }
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
    let week_summary = week.summary.clone();
    let week_uuid = week.uuid.clone();

    let set_current_week_index = state.current_week_index.setter();
    let set_current_week = state.current_week.setter();
    let set_current_quarter = state.current_quarter.setter();
    let set_lessons = state.lessons.setter();
    let set_lessons_loading = state.lessons_loading.setter();
    let set_conn = state.conn_status.setter();

    // Instant-serve from week_cache when present (without cloning whole HashMap)
    let cached = state.week_cache.with(|c| c.get(&(idx as i32)).cloned());
    let mut need_fetch = true;

    day::reactive::batch(|| {
        set_current_week_index.set(idx as i32);
        set_current_week.set(week_summary);
        set_current_quarter.set(quarter_for_index(idx));

        if let Some(lessons) = cached {
            set_lessons.set(lessons);
            set_lessons_loading.set(false);
            if state.conn_status.get() == crate::app::ConnStatus::Connecting {
                set_conn.set(crate::app::ConnStatus::Connected);
            }
            need_fetch = false;
            nslog::nslog(&format!("[Diary] load_week idx={idx} served from cache"));
        } else {
            set_lessons.set(Vec::new());
            set_lessons_loading.set(true);
            set_conn.set(crate::app::ConnStatus::Connecting);
            nslog::nslog(&format!("[Diary] load_week idx={idx} uuid={week_uuid}"));
        }
    });

    // Neighbor prefetch: idx±1 not already in cache
    let mut prefetch: Vec<(i32, String)> = Vec::new();
    state.week_cache.with(|cache| {
        for di in [-1i32, 1i32] {
            let ni = idx as i32 + di;
            if ni < 0 || ni as usize >= weeks.len() {
                continue;
            }
            if cache.contains_key(&ni) {
                continue;
            }
            if let Some(w) = weeks.get(ni as usize) {
                prefetch.push((ni, w.uuid.clone()));
            }
        }
    });

    let cur_uuid = week_uuid;
    let cur_i = idx as i32;
    let need_current = need_fetch;
    let set_lessons = state.lessons.setter();
    let set_lessons_loading = state.lessons_loading.setter();
    let set_conn = state.conn_status.setter();

    std::thread::spawn(move || {
        let client = blocking::build_client(&token);

        if need_current {
            match blocking::api_get_raw(&client, &endpoints::lessons(&school_id, &class_id, &profile_id, &cur_uuid)) {
                Ok(raw) => {
                    match serde_json::from_str::<Vec<DaySchedule>>(&raw) {
                        Ok(lessons) => {
                            nslog::nslog(&format!("[Diary] load_week idx={cur_i} OK ({} days)", lessons.len()));
                            let ci = cur_i;
                            let ls = lessons.clone();
                            day::reactive::on_main(move || {
                                if let Some(state) = AppState::get_main() {
                                    if state.current_week_index.get() == ci {
                                        state.lessons.setter().set(ls.clone());
                                    }
                                    let mut wc = state.week_cache.get();
                                    wc.insert(ci, ls);
                                    cache::save_json("week_cache", &wc);
                                    state.week_cache.set(wc);
                                } else {
                                    set_lessons.set(ls);
                                }
                            });
                        }
                        Err(e) => {
                            nslog::nslog(&format!("[Diary] load_week parse failed: {e}"));
                            set_conn.set(crate::app::ConnStatus::Error);
                        }
                    }
                }
                Err(e) => {
                    nslog::nslog(&format!("[Diary] load_week request failed: {e}"));
                    set_conn.set(crate::app::ConnStatus::Error);
                }
            }

            day::reactive::on_main(move || {
                set_lessons_loading.set(false);
                if let Some(state) = AppState::get_main() {
                    state.lessons_loading.setter().set(false);
                    if state.conn_status.get() == crate::app::ConnStatus::Connecting {
                        let set_c = state.conn_status.setter();
                        set_c.set(crate::app::ConnStatus::Connected);
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            set_c.set(crate::app::ConnStatus::Idle);
                        });
                    }
                }
            });
        }

        // Quiet neighbor prefetch
        for (ni, nuuid) in prefetch {
            match blocking::api_get_raw(&client, &endpoints::lessons(&school_id, &class_id, &profile_id, &nuuid)) {
                Ok(raw) => {
                    if let Ok(ls) = serde_json::from_str::<Vec<DaySchedule>>(&raw) {
                        day::reactive::on_main(move || {
                            if let Some(state) = AppState::get_main() {
                                if state.current_week_index.get() == ni {
                                    state.lessons.setter().set(ls.clone());
                                }
                                let mut wc = state.week_cache.get();
                                wc.insert(ni, ls);
                                cache::save_json("week_cache", &wc);
                                state.week_cache.set(wc);
                            }
                        });
                    }
                }
                Err(e) => nslog::nslog(&format!("[Diary] prefetch {ni} failed: {e}")),
            }
        }
    });
}

/// Extract all marks for a quarter from week_cache.
pub fn extract_marks_for_quarter(
    cache: &std::collections::HashMap<i32, Vec<DaySchedule>>,
    quarter: usize,
) -> (
    std::collections::HashMap<String, Vec<f64>>,
    std::collections::HashMap<String, Vec<OfficialMark>>,
) {
    let mut marks: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
    let mut official: std::collections::HashMap<String, Vec<OfficialMark>> = std::collections::HashMap::new();

    let (start, end) = if quarter < 4 {
        QUARTER_RANGES[quarter]
    } else {
        (0, 36)
    };

    for idx in start..end {
        if let Some(days) = cache.get(&(idx as i32)) {
            for day in days {
                for slot in &day.slots {
                    if let Some(ref lm) = slot.lesson_mark {
                        if let Some(ref m_str) = lm.mark {
                            let parsed = utils::parse_marks(m_str);
                            for val in &parsed {
                                marks.entry(slot.subject_title.clone())
                                    .or_default()
                                    .push(*val);
                            }
                            if let Some(&first_val) = parsed.first() {
                                let has_author = lm.author.as_ref().map(|a| !a.is_empty()).unwrap_or(false);
                                if has_author || lm.kind.is_some() {
                                    official.entry(slot.subject_title.clone())
                                        .or_default()
                                        .push(OfficialMark {
                                            value: first_val,
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
    }

    (marks, official)
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

    let set_current_quarter = state.current_quarter.setter();
    let set_marks_loading = state.marks_loading.setter();
    let set_quarter_all_marks = state.quarter_all_marks.setter();
    let set_quarter_official_marks = state.quarter_official_marks.setter();
    let set_quarter_marks = state.quarter_marks.setter();
    let set_official_marks = state.official_marks.setter();
    let set_conn = state.conn_status.setter();

    set_current_quarter.set(quarter);
    set_marks_loading.set(true);
    set_conn.set(crate::app::ConnStatus::Connecting);

    // 1. Immediately extract any existing marks from week_cache so UI displays instantly!
    let (cached_marks, cached_official) = state.week_cache.with(|c| {
        extract_marks_for_quarter(c, quarter)
    });
    let mut q_all = state.quarter_all_marks.get();
    let mut q_off = state.quarter_official_marks.get();
    if q_all.len() <= quarter {
        q_all.resize(quarter + 1, std::collections::HashMap::new());
        q_off.resize(quarter + 1, std::collections::HashMap::new());
    }
    if !cached_marks.is_empty() {
        q_all[quarter] = cached_marks.clone();
        q_off[quarter] = cached_official.clone();
        set_quarter_all_marks.set(q_all.clone());
        set_quarter_official_marks.set(q_off.clone());
        set_quarter_marks.set(cached_marks);
        set_official_marks.set(cached_official);
    }

    let actual_end = end.min(weeks.len());
    let week_info: Vec<(usize, String)> = (start..actual_end)
        .filter_map(|idx| weeks.get(idx).map(|w| (idx, w.uuid.clone())))
        .collect();

    nslog::nslog(&format!("[Diary] load_quarter q={} weeks={}", quarter + 1, week_info.len()));

    let init_q_all = state.quarter_all_marks.get();
    let init_q_off = state.quarter_official_marks.get();

    std::thread::spawn(move || {
        let client = blocking::build_client(&token);
        let mut all_marks: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
        let mut all_official: std::collections::HashMap<String, Vec<OfficialMark>> = std::collections::HashMap::new();

        if let Some(existing) = init_q_all.get(quarter) {
            all_marks = existing.clone();
        }
        if let Some(existing_off) = init_q_off.get(quarter) {
            all_official = existing_off.clone();
        }

        for (week_idx, week_uuid) in &week_info {
            match blocking::api_get_raw(&client, &endpoints::lessons(&school_id, &class_id, &profile_id, week_uuid)) {
                Ok(raw) => {
                    if let Ok(days) = serde_json::from_str::<Vec<DaySchedule>>(&raw) {
                        let w_idx = *week_idx as i32;
                        let days_clone = days.clone();
                        day::reactive::on_main(move || {
                            if let Some(state) = AppState::get_main() {
                                let mut wc = state.week_cache.get();
                                wc.insert(w_idx, days_clone);
                                state.week_cache.set(wc);
                            }
                        });

                        for day in &days {
                            for slot in &day.slots {
                                if let Some(ref lm) = slot.lesson_mark {
                                    if let Some(ref mark_str) = lm.mark {
                                        let parsed = utils::parse_marks(mark_str);
                                        for val in &parsed {
                                            all_marks.entry(slot.subject_title.clone())
                                                .or_default()
                                                .push(*val);
                                        }
                                        if let Some(&first_val) = parsed.first() {
                                            let has_author = lm.author.as_ref().map(|a| !a.is_empty()).unwrap_or(false);
                                            if has_author || lm.kind.is_some() {
                                                all_official.entry(slot.subject_title.clone())
                                                    .or_default()
                                                    .push(OfficialMark {
                                                        value: first_val,
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
                }
                Err(e) => nslog::nslog(&format!("[Diary] load_quarter week failed: {e}")),
            }
        }

        // Store per-quarter
        {
            let mut q_all = init_q_all.clone();
            let mut q_off = init_q_off.clone();
            if q_all.len() <= quarter {
                q_all.resize(quarter + 1, std::collections::HashMap::new());
                q_off.resize(quarter + 1, std::collections::HashMap::new());
            }
            q_all[quarter] = all_marks.clone();
            q_off[quarter] = all_official.clone();
            set_quarter_all_marks.set(q_all.clone());
            set_quarter_official_marks.set(q_off.clone());
            cache::save_json("quarter_all_marks", &q_all);
            cache::save_json("quarter_official_marks", &q_off);
        }

        set_quarter_marks.set(all_marks);
        set_official_marks.set(all_official);

        day::reactive::on_main(|| {
            if let Some(state) = AppState::get_main() {
                state.marks_loading.setter().set(false);
                if state.conn_status.get() == crate::app::ConnStatus::Connecting {
                    let set_c = state.conn_status.setter();
                    set_c.set(crate::app::ConnStatus::Connected);
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        set_c.set(crate::app::ConnStatus::Idle);
                    });
                }
            }
        });

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

    let set_current_quarter = state.current_quarter.setter();
    let set_marks_loading = state.marks_loading.setter();
    let set_quarter_all_marks = state.quarter_all_marks.setter();
    let set_quarter_official_marks = state.quarter_official_marks.setter();
    let set_year_quarter_data = state.year_quarter_data.setter();
    let set_quarter_marks = state.quarter_marks.setter();
    let set_official_marks = state.official_marks.setter();
    let set_conn = state.conn_status.setter();

    set_current_quarter.set(4);
    set_marks_loading.set(true);
    set_conn.set(crate::app::ConnStatus::Connecting);

    let quarter_labels = ["I четверть", "II четверть", "III четверть", "IV четверть"];

    // Seed year_quarter_data immediately from quarter_all_marks or week_cache
    let mut initial_yqd: Vec<(String, std::collections::HashMap<String, Vec<f64>>)> = Vec::new();
    let current_q_all = state.quarter_all_marks.get();
    for q in 0..4 {
        let m = current_q_all.get(q).filter(|m| !m.is_empty()).cloned().unwrap_or_else(|| {
            state.week_cache.with(|c| {
                let (km, _) = extract_marks_for_quarter(c, q);
                km
            })
        });
        initial_yqd.push((quarter_labels[q].to_string(), m));
    }
    set_year_quarter_data.clone().set(initial_yqd);

    let mut q_weeks: Vec<Vec<(usize, String)>> = Vec::new();
    for q in 0..4 {
        let (start, end) = QUARTER_RANGES[q];
        let actual_end = end.min(weeks.len());
        let uuids: Vec<(usize, String)> = (start..actual_end)
            .filter_map(|idx| weeks.get(idx).map(|w| (idx, w.uuid.clone())))
            .collect();
        q_weeks.push(uuids);
    }

    nslog::nslog(&format!("[Diary] load_year total_weeks={}", q_weeks.iter().map(|v| v.len()).sum::<usize>()));

    std::thread::spawn(move || {
        let client = blocking::build_client(&token);
        let mut year_data: Vec<(String, std::collections::HashMap<String, Vec<f64>>)> = Vec::new();
        let mut all_marks: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
        let mut all_official: std::collections::HashMap<String, Vec<OfficialMark>> = std::collections::HashMap::new();

        let mut q_all_marks: Vec<std::collections::HashMap<String, Vec<f64>>> = Vec::new();
        let mut q_official_marks: Vec<std::collections::HashMap<String, Vec<OfficialMark>>> = Vec::new();

        for q in 0..4 {
            let mut q_marks: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
            let mut q_off: std::collections::HashMap<String, Vec<OfficialMark>> = std::collections::HashMap::new();

            for (week_idx, week_uuid) in &q_weeks[q] {
                match blocking::api_get_raw(&client, &endpoints::lessons(&school_id, &class_id, &profile_id, week_uuid)) {
                    Ok(raw) => {
                        if let Ok(days) = serde_json::from_str::<Vec<DaySchedule>>(&raw) {
                            let w_idx = *week_idx as i32;
                            let days_clone = days.clone();
                            day::reactive::on_main(move || {
                                if let Some(state) = AppState::get_main() {
                                    let mut wc = state.week_cache.get();
                                    wc.insert(w_idx, days_clone);
                                    state.week_cache.set(wc);
                                }
                            });

                            for day in &days {
                                for slot in &day.slots {
                                    if let Some(ref lm) = slot.lesson_mark {
                                        if let Some(ref mark_str) = lm.mark {
                                            let parsed = utils::parse_marks(mark_str);
                                            for val in &parsed {
                                                q_marks.entry(slot.subject_title.clone())
                                                    .or_default()
                                                    .push(*val);
                                                all_marks.entry(slot.subject_title.clone())
                                                    .or_default()
                                                    .push(*val);
                                            }
                                            if let Some(&first_val) = parsed.first() {
                                                let has_author = lm.author.as_ref().map(|a| !a.is_empty()).unwrap_or(false);
                                                if has_author || lm.kind.is_some() {
                                                    let om = OfficialMark {
                                                        value: first_val,
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
                    }
                    Err(e) => nslog::nslog(&format!("[Diary] load_year q{} week failed: {e}", q + 1)),
                }
            }

            year_data.push((quarter_labels[q].to_string(), q_marks.clone()));
            q_all_marks.push(q_marks);
            q_official_marks.push(q_off);
        }

        set_quarter_all_marks.set(q_all_marks.clone());
        set_quarter_official_marks.set(q_official_marks.clone());
        set_year_quarter_data.set(year_data);
        set_quarter_marks.set(all_marks);
        set_official_marks.set(all_official);
        cache::save_json("quarter_all_marks", &q_all_marks);
        cache::save_json("quarter_official_marks", &q_official_marks);

        day::reactive::on_main(|| {
            if let Some(state) = AppState::get_main() {
                state.marks_loading.setter().set(false);
                if state.conn_status.get() == crate::app::ConnStatus::Connecting {
                    let set_c = state.conn_status.setter();
                    set_c.set(crate::app::ConnStatus::Connected);
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        set_c.set(crate::app::ConnStatus::Idle);
                    });
                }
            }
        });

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
