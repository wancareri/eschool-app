//! Diary data loading — all school data fetching for the diary, schedule, and teachers.

use crate::app::AppState;
use crate::app::FinalMarks;
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
    let set_final_marks = state.final_marks.setter();
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
        if let Some(idx) = find_current_week(&cached_weeks, now_ms) {
            let w = &cached_weeks[idx];
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
    if let Some(cached_fm) = cache::load_json::<FinalMarks>("final_marks") {
        nslog::nslog("[Cache] Loading cached final marks");
        set_final_marks.clone().set(cached_fm);
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
            q_all.resize(4, std::collections::HashMap::new());
            set_quarter_all_marks.clone().set(q_all);
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
                let cur = find_current_week(&weeks, now_ms);
                nslog::nslog(&format!("[Diary] current week idx={cur:?}"));

                set_all_weeks.set(weeks.clone());
                cache::save_json("all_weeks", &weeks);

                if let Some(idx) = cur {
                    let week = &weeks[idx];
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
        let mut uuid_to_title: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        if let Ok(subjects) = blocking::api_get::<Vec<SubjectWithTeacher>>(
            &client,
            &endpoints::subjects(&user.school_id, &user.profile_id, &class_id),
        ) {
            uuid_to_title = subjects.iter().map(|s| (s.id.clone(), s.subject_title.clone())).collect();
            set_subjects_teachers.set(subjects.clone());
            cache::save_json("subjects_teachers", &subjects);
        }
        if uuid_to_title.is_empty() {
            // Fall back to the cached list so finals still key correctly offline.
            if let Some(cached) = cache::load_json::<Vec<SubjectWithTeacher>>("subjects_teachers") {
                uuid_to_title = cached.iter().map(|s| (s.id.clone(), s.subject_title.clone())).collect();
            }
        }

        // 8 — final marks (/final/whole): the ONLY trustworthy source for the
        // «Выставл.» column. A lesson mark carries an author and a kind on
        // ordinary grades too, so heuristics over the lessons feed displayed
        // marks no one had ever set as a quarter final.
        if uuid_to_title.is_empty() {
            // Without a subject list the uuids can't be keyed; keep whatever
            // the cache already holds rather than overwriting it with nothing.
            nslog::nslog("[Diary] final marks: no subject map, skipping fetch");
        } else {
            match blocking::api_get_raw(
                &client,
                &endpoints::final_marks(&user.school_id, &class_id, &user.profile_id),
            ) {
                Ok(raw) => match parse_final_marks(&raw, &uuid_to_title) {
                    Some(finals) => {
                        nslog::nslog(&format!("[Diary] final marks for {} subjects", finals.len()));
                        set_final_marks.clone().set(finals.clone());
                        cache::save_json("final_marks", &finals);
                    }
                    None => nslog::nslog("[Diary] final marks: unexpected payload shape"),
                },
                Err(e) => nslog::nslog(&format!("[Diary] final marks failed: {e}")),
            }
        }

        set_loading.set(false);
        set_conn2.set(crate::app::ConnStatus::Connected);
        nslog::nslog("[Diary] load_all completed");

        // Auto-hide the "Connected" indicator after 2 seconds — but only if
        // no newer load took over (it would be Connecting/Connected now).
        std::thread::sleep(std::time::Duration::from_secs(2));
        day::reactive::on_main(move || {
            if let Some(state) = AppState::get_main() {
                if state.conn_status.get() == crate::app::ConnStatus::Connected {
                    set_conn2.set(crate::app::ConnStatus::Idle);
                }
            }
        });
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

/// Warm week_cache[idx] without touching the current index: called at the START of a
/// swipe animation so the incoming page can draw real lessons while it slides in
/// instead of the spinner its `is_loaded` gate shows on a cache miss.
pub fn prefetch_week(state: AppState, idx: i32) {
    if idx < 0 { return; }
    let weeks = state.all_weeks.get();
    if idx as usize >= weeks.len() { return; }
    if state.week_cache.with(|c| c.contains_key(&idx)) { return; }
    let (school_id, class_id, profile_id) = auth::get_stored_ids();
    if school_id.is_empty() || class_id.is_empty() || profile_id.is_empty() { return; }
    let Some(token) = auth::get_token() else { return; };
    let uuid = weeks[idx as usize].uuid.clone();

    std::thread::spawn(move || {
        let client = blocking::build_client(&token);
        match blocking::api_get_raw(&client, &endpoints::lessons(&school_id, &class_id, &profile_id, &uuid)) {
            Ok(raw) => {
                if let Ok(lessons) = serde_json::from_str::<Vec<DaySchedule>>(&raw) {
                    day::reactive::on_main(move || {
                        if let Some(state) = AppState::get_main() {
                            if state.week_cache.with(|c| c.contains_key(&idx)) {
                                return;
                            }
                            let mut wc = state.week_cache.get();
                            wc.insert(idx, lessons);
                            cache::save_json("week_cache", &wc);
                            state.week_cache.set(wc);
                            nslog::nslog(&format!("[Diary] prefetch idx={idx} OK"));
                        }
                    });
                }
            }
            Err(_) => {}
        }
    });
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
                            day::reactive::on_main(move || {
                                if let Some(s) = AppState::get_main() {
                                    if s.conn_status.get() == crate::app::ConnStatus::Connected {
                                        set_c.set(crate::app::ConnStatus::Idle);
                                    }
                                }
                            });
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

/// Extract all marks for a quarter from week_cache (lesson averages only —
/// final marks come from `/final/whole`, see [`crate::app::FinalMarks`]).
pub fn extract_marks_for_quarter(
    cache: &std::collections::HashMap<i32, Vec<DaySchedule>>,
    quarter: usize,
) -> std::collections::HashMap<String, Vec<f64>> {
    let mut marks: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();

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
                        }
                    }
                }
            }
        }
    }

    marks
}

/// Dated lesson marks for ONE subject in a quarter — `(date, raw, parsed)`,
/// date-sorted. Each entry is one lesson slot: the date from its day, the mark
/// as the API spelled it (`"5"`, `"5/4"`), the parsed values for averages.
/// Powers the Итоги peek strip; averages elsewhere use
/// [`extract_marks_for_quarter`], which aggregates the same slots without dates.
pub fn extract_marks_dated(
    cache: &std::collections::HashMap<i32, Vec<DaySchedule>>,
    quarter: usize,
    subject: &str,
) -> Vec<(u64, String, Vec<f64>)> {
    let (start, end) = if quarter < 4 {
        QUARTER_RANGES[quarter]
    } else {
        (0, 36)
    };

    let mut out: Vec<(u64, String, Vec<f64>)> = Vec::new();
    for idx in start..end {
        if let Some(days) = cache.get(&(idx as i32)) {
            for day in days {
                for slot in &day.slots {
                    if slot.subject_title != subject {
                        continue;
                    }
                    let Some(ref lm) = slot.lesson_mark else { continue };
                    let Some(ref m_str) = lm.mark else { continue };
                    let parsed = utils::parse_marks(m_str);
                    if parsed.is_empty() {
                        continue;
                    }
                    out.push((day.date, m_str.clone(), parsed));
                }
            }
        }
    }
    out.sort_by_key(|e| e.0);
    out
}

/// Parse `/final/whole` into `{subject title → {period → mark}}`.
///
/// The payload keys marks by subject uuid:
/// `{ "marks": { "<uuid>": { "FIRST_QUARTER": {"mark":"5"}, …, "YEAR": {…} } } }`,
/// so uuids are mapped to titles through the subjects list; entries for
/// subjects we don't know are skipped.
fn parse_final_marks(
    raw: &str,
    uuid_to_title: &std::collections::HashMap<String, String>,
) -> Option<FinalMarks> {
    let v: serde_json::Value = serde_json::from_str(raw).ok()?;
    let marks = v.get("marks")?.as_object()?;
    let mut out: FinalMarks = std::collections::HashMap::new();
    for (uuid, periods) in marks {
        let Some(title) = uuid_to_title.get(uuid) else { continue };
        let Some(periods) = periods.as_object() else { continue };
        for (period, cell) in periods {
            let mark = cell
                .get("mark")
                .and_then(|m| m.as_str())
                .or_else(|| cell.as_str())
                .unwrap_or_default();
            if mark.is_empty() {
                continue;
            }
            out.entry(title.clone())
                .or_default()
                .insert(period.clone(), mark.to_string());
        }
    }
    Some(out)
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
    let set_quarter_marks = state.quarter_marks.setter();
    let set_conn = state.conn_status.setter();

    set_current_quarter.set(quarter);
    set_marks_loading.set(true);
    set_conn.set(crate::app::ConnStatus::Connecting);

    // 1. Immediately extract any existing marks from week_cache so UI displays instantly!
    let cached_marks = state.week_cache.with(|c| {
        extract_marks_for_quarter(c, quarter)
    });
    let mut q_all = state.quarter_all_marks.get();
    if q_all.len() <= quarter {
        q_all.resize(quarter + 1, std::collections::HashMap::new());
    }
    if !cached_marks.is_empty() {
        q_all[quarter] = cached_marks.clone();
        set_quarter_all_marks.set(q_all.clone());
        set_quarter_marks.set(cached_marks);
    }

    let actual_end = end.min(weeks.len());
    let week_info: Vec<(usize, String)> = (start..actual_end)
        .filter_map(|idx| weeks.get(idx).map(|w| (idx, w.uuid.clone())))
        .collect();

    nslog::nslog(&format!("[Diary] load_quarter q={} weeks={}", quarter + 1, week_info.len()));

    let init_q_all = state.quarter_all_marks.get();

    std::thread::spawn(move || {
        let client = blocking::build_client(&token);
        let mut all_marks: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();

        if let Some(existing) = init_q_all.get(quarter) {
            all_marks = existing.clone();
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
                                cache::save_json("week_cache", &wc);
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
            if q_all.len() <= quarter {
                q_all.resize(quarter + 1, std::collections::HashMap::new());
            }
            q_all[quarter] = all_marks.clone();
            set_quarter_all_marks.set(q_all.clone());
            cache::save_json("quarter_all_marks", &q_all);
        }

        set_quarter_marks.set(all_marks);

        day::reactive::on_main(|| {
            if let Some(state) = AppState::get_main() {
                state.marks_loading.setter().set(false);
                if state.conn_status.get() == crate::app::ConnStatus::Connecting {
                    let set_c = state.conn_status.setter();
                    set_c.set(crate::app::ConnStatus::Connected);
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        day::reactive::on_main(move || {
                            if let Some(s) = AppState::get_main() {
                                if s.conn_status.get() == crate::app::ConnStatus::Connected {
                                    set_c.set(crate::app::ConnStatus::Idle);
                                }
                            }
                        });
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
    let set_year_quarter_data = state.year_quarter_data.setter();
    let set_quarter_marks = state.quarter_marks.setter();
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
                extract_marks_for_quarter(c, q)
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

        let mut q_all_marks: Vec<std::collections::HashMap<String, Vec<f64>>> = Vec::new();

        for q in 0..4 {
            let mut q_marks: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();

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
                                    cache::save_json("week_cache", &wc);
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
        }

        set_quarter_all_marks.set(q_all_marks.clone());
        set_year_quarter_data.set(year_data);
        set_quarter_marks.set(all_marks);
        cache::save_json("quarter_all_marks", &q_all_marks);

        day::reactive::on_main(|| {
            if let Some(state) = AppState::get_main() {
                state.marks_loading.setter().set(false);
                if state.conn_status.get() == crate::app::ConnStatus::Connecting {
                    let set_c = state.conn_status.setter();
                    set_c.set(crate::app::ConnStatus::Connected);
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        day::reactive::on_main(move || {
                            if let Some(s) = AppState::get_main() {
                                if s.conn_status.get() == crate::app::ConnStatus::Connected {
                                    set_c.set(crate::app::ConnStatus::Idle);
                                }
                            }
                        });
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

/// School API timestamps may arrive as seconds or milliseconds — everything
/// compares in ms, so anything below ~2001-09 in magnitude is seconds.
fn normalize_ts(ts: u64) -> u64 {
    if ts < 1_000_000_000_000 { ts * 1000 } else { ts }
}

/// The week the app opens on: the one containing `now_ms`; when none does
/// (a gap between school years, a stale cache), the most recent week that has
/// already started — never a silent fall back to week 0.
pub fn find_current_week(weeks: &[WeekActivity], now_ms: u64) -> Option<usize> {
    weeks
        .iter()
        .position(|w| {
            let start = normalize_ts(w.start_ts);
            let end = normalize_ts(w.end_ts);
            start <= now_ms && now_ms <= end
        })
        .or_else(|| {
            weeks
                .iter()
                .enumerate()
                .filter(|(_, w)| normalize_ts(w.start_ts) <= now_ms)
                .map(|(i, _)| i)
                .last()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn week(start_ts: u64, end_ts: u64) -> WeekActivity {
        WeekActivity {
            uuid: String::new(),
            summary: String::new(),
            time_activity_id: String::new(),
            start_ts,
            end_ts,
        }
    }

    #[test]
    fn seconds_and_milliseconds_both_match() {
        let now = 1_790_000_000_000u64;
        let in_secs = vec![week((now - 1_000) / 1_000, (now + 1_000) / 1_000)];
        assert_eq!(find_current_week(&in_secs, now), Some(0));
        let in_millis = vec![week(now - 1_000_000, now + 1_000_000)];
        assert_eq!(find_current_week(&in_millis, now), Some(0));
    }

    #[test]
    fn falls_back_to_latest_started_week_in_a_gap() {
        let now = 1_790_000_000_000u64;
        let day = 86_400_000u64;
        let weeks = vec![
            week(now - 20 * day, now - 14 * day),
            week(now - 13 * day, now - 7 * day),
            week(now + day, now + 2 * day),
        ];
        assert_eq!(find_current_week(&weeks, now), Some(1));
    }

    #[test]
    fn future_only_list_and_empty_list() {
        let now = 1_790_000_000_000u64;
        let day = 86_400_000u64;
        let future = vec![week(now + day, now + 2 * day)];
        assert_eq!(find_current_week(&future, now), None);
        assert_eq!(find_current_week(&[], now), None);
    }
}
