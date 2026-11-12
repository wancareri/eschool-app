//! Diary capability facade. Focused modules own calendar, loading, and marks.
use crate::app::{AppState, ConnStatus, FinalMarks};
use crate::pages::login::auth;
use crate::app::data::cache;
use crate::ios::nslog;
use crate::app::data::session;
use crate::app::format;
use eschool_api::client::{blocking, endpoints};
use eschool_api::entities::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

mod calendar;
mod notes;
use calendar::{academic_year, find_current_week, now_ms, week_quarters};
pub use notes::{delete_note, get_note, load_notes_map, save_note};

type WeekCache = HashMap<i32, Vec<DaySchedule>>;
type Marks = HashMap<String, Vec<f64>>;
static LOAD_REQUEST: AtomicU64 = AtomicU64::new(0);
static MARKS_REQUEST: AtomicU64 = AtomicU64::new(0);
static WEEK_REQUEST: AtomicU64 = AtomicU64::new(0);
static RESPONSE_ORDER: AtomicU64 = AtomicU64::new(0);
static LATEST_WEEK: LazyLock<Mutex<HashMap<String, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn response_order() -> u64 {
    RESPONSE_ORDER.fetch_add(1, Ordering::AcqRel) + 1
}

fn accept_week(uuid: &str, order: u64) -> bool {
    let mut latest = LATEST_WEEK.lock().unwrap();
    let current = latest.entry(uuid.to_string()).or_default();
    if order < *current {
        return false;
    }
    *current = order;
    true
}

fn restore_cache(state: AppState) {
    if let Some(weeks) = cache::load_json("all_weeks") {
        state.all_weeks.set(weeks);
    }
    if let Some(periods) = cache::load_json("time_activities") {
        state.time_activities.set(periods);
    }
    state.week_quarters.set(week_quarters(
        &state.all_weeks.get(),
        &state.time_activities.get(),
    ));
    if let Some(wc) = cache::load_json("week_cache") {
        state.week_cache.set(wc);
    }
    if let Some(idx) = find_current_week(&state.all_weeks.get(), now_ms()) {
        select_week(state, idx);
        state.lessons.set(
            state
                .week_cache
                .with(|c| c.get(&(idx as i32)).cloned().unwrap_or_default()),
        );
    }
    if let Some(v) = cache::load_json("bell_times") {
        state.bell_times.set(v);
    }
    if let Some(v) = cache::load_json("timetable_days") {
        state.timetable_days.set(v);
    }
    if let Some(v) = cache::load_json("subjects_teachers") {
        state.subjects_teachers.set(v);
    }
    if let Some(v) = cache::load_json("final_marks") {
        state.final_marks.set(v);
    }
    if let Some(v) = cache::load("user_full_name") {
        state.full_name.set(v);
    }
    if let Some(v) = cache::load("user_school_name") {
        state.school_name.set(v);
    }
    state.lesson_notes.set(notes::load_notes_map());
    rebuild_marks(state);
}

fn select_week(state: AppState, idx: usize) {
    state.current_week_index.set(idx as i32);
    if let Some(w) = state.all_weeks.with(|w| w.get(idx).cloned()) {
        state.current_week.set(w.summary);
    }
    if let Some(q) = state.week_quarters.with(|q| q.get(idx).copied().flatten()) {
        state.current_quarter.set(q);
    }
}

/// Clear every student-specific value, including modals and pending indicators.
/// The caller invalidates session tickets before invoking this.
pub fn clear_data(state: AppState) {
    state.full_name.set(String::new());
    state.school_name.set(String::new());
    state.class_label.set(String::new());
    state.is_graduating.set(false);
    state.lessons.set(Vec::new());
    state.all_weeks.set(Vec::new());
    state.time_activities.set(Vec::new());
    state.week_quarters.set(Vec::new());
    state.current_week.set(String::new());
    state.current_week_index.set(0);
    state.current_quarter.set(0);
    state.week_cache.set(HashMap::new());
    reset_marks(state);
    state.year_quarter_data.set(Vec::new());
    state.final_marks.set(FinalMarks::new());
    state.bell_times.set(Vec::new());
    state.timetable_days.set(Vec::new());
    state.subjects_teachers.set(Vec::new());
    state.lesson_notes.set(HashMap::new());
    state.lesson_attachments.set(HashMap::new());
    state.attachments_loading.set(Default::default());
    state.downloading_files.set(Default::default());
    state.active_note.set(None);
    state.active_lesson_peek.set(None);
    crate::pages::diary::grade_peek::close();
    state.show_network_modal.set(None);
    state.loading.set(false);
    state.lessons_loading.set(false);
    state.marks_loading.set(false);
    state.schedule_loading.set(false);
    state.teachers_loading.set(false);
    state.error_msg.set(String::new());
    state.conn_status.set(ConnStatus::Idle);
}

struct Loaded {
    user: UserInfo,
    child_idx: usize,
    school: String,
    profile: String,
    name: String,
    school_name: String,
    class: Class,
    weeks: Vec<WeekActivity>,
    periods: Vec<TimeActivity>,
    current: Option<(usize, Vec<DaySchedule>)>,
    bells: Option<Vec<BellTime>>,
    timetable: Option<Vec<TimetableDay>>,
    teachers: Option<Vec<SubjectWithTeacher>>,
    finals: Option<FinalMarks>,
    partial_error: Option<String>,
}

pub fn load_all(state: AppState) {
    let Some(token) = auth::get_token() else {
        return;
    };
    let ticket = session::ticket();
    let request = LOAD_REQUEST.fetch_add(1, Ordering::AcqRel) + 1;
    let order = response_order();
    let old_profile = auth::get_stored_ids().2;
    let selected = state.active_child_index.get();
    if state.all_weeks.with(|w| w.is_empty()) {
        restore_cache(state);
    }
    state.loading.set(true);
    state.lessons_loading.set(true);
    state.schedule_loading.set(true);
    state.teachers_loading.set(true);
    state.error_msg.set(String::new());
    state.conn_status.set(ConnStatus::Connecting);
    std::thread::spawn(move || {
        let result = fetch_all(&token, selected, &old_profile);
        ticket.publish(move |state| {
            if LOAD_REQUEST.load(Ordering::Acquire) != request {
                return;
            }
            state.loading.set(false);
            state.lessons_loading.set(false);
            state.schedule_loading.set(false);
            state.teachers_loading.set(false);
            match result {
                Ok(data) => {
                    // Keep a user's selection during a refresh, but return to
                    // today after a new profile has been selected.
                    let selected_uuid = state.all_weeks.with(|w| {
                        w.get(state.current_week_index.get() as usize)
                            .map(|w| w.uuid.clone())
                    });
                    let selected_quarter = state.current_quarter.get();
                    let had_selection = selected_uuid.is_some();
                    let profile_changed = auth::get_stored_ids().2 != data.profile;
                    if profile_changed {
                        session::invalidate();
                        clear_data(state);
                    }
                    auth::store_ids(
                        &data.school,
                        &data.class.uuid,
                        &data.profile,
                        &data.name,
                        &data.school_name,
                    );
                    if profile_changed {
                        restore_cache(state);
                    }
                    state.is_parent.set(data.user.is_parent());
                    state.parent_children.set(data.user.children().to_vec());
                    state.active_child_index.set(data.child_idx);
                    state.full_name.set(data.name.clone());
                    state.school_name.set(data.school_name.clone());
                    state
                        .class_label
                        .set(format!("{}{}", data.class.level, data.class.label));
                    state
                        .is_graduating
                        .set(data.class.graduating.unwrap_or(false));
                    cache::save("user_full_name", &data.name);
                    cache::save("user_school_name", &data.school_name);
                    // Index keys are valid only for this exact ordered list.
                    let old_weeks = state.all_weeks.get();
                    let remapped: WeekCache = state.week_cache.with(|c| {
                        data.weeks
                            .iter()
                            .enumerate()
                            .filter_map(|(i, w)| {
                                let old = old_weeks.iter().position(|v| v.uuid == w.uuid)?;
                                Some((i as i32, c.get(&(old as i32))?.clone()))
                            })
                            .collect()
                    });
                    state.week_cache.set(remapped);
                    state.all_weeks.set(data.weeks.clone());
                    if !data.periods.is_empty() {
                        state.time_activities.set(data.periods.clone());
                    }
                    state
                        .week_quarters
                        .set(week_quarters(&data.weeks, &state.time_activities.get()));
                    cache::save_json("all_weeks", &data.weeks);
                    cache::save_json("time_activities", &state.time_activities.get());
                    let fetched_idx = data.current.as_ref().map(|(idx, _)| *idx);
                    if let Some((idx, days)) = data.current {
                        if accept_week(&data.weeks[idx].uuid, order) {
                            state.week_cache.update(|wc| {
                                wc.insert(idx as i32, days.clone());
                            });
                            super::grade_checker::observe_week(&data.weeks[idx].uuid, &days);
                        }
                    }
                    cache::save_json("week_cache", &state.week_cache.get());
                    let idx = selected_uuid
                        .filter(|_| !profile_changed)
                        .and_then(|uuid| data.weeks.iter().position(|w| w.uuid == uuid))
                        .or_else(|| find_current_week(&data.weeks, now_ms()));
                    if let Some(idx) = idx {
                        select_week(state, idx);
                        if had_selection && !profile_changed {
                            state.current_quarter.set(selected_quarter);
                        }
                        state.lessons.set(
                            state
                                .week_cache
                                .with(|c| c.get(&(idx as i32)).cloned().unwrap_or_default()),
                        );
                    }
                    if let Some(v) = data.bells {
                        cache::save_json("bell_times", &v);
                        state.bell_times.set(v);
                    }
                    if let Some(v) = data.timetable {
                        cache::save_json("timetable_days", &v);
                        state.timetable_days.set(v);
                    }
                    if let Some(v) = data.teachers {
                        cache::save_json("subjects_teachers", &v);
                        state.subjects_teachers.set(v);
                    }
                    if let Some(v) = data.finals {
                        cache::save_json("final_marks", &v);
                        state.final_marks.set(v);
                    }
                    rebuild_marks(state);
                    if let Some(e) = data.partial_error {
                        state.error_msg.set(e);
                        state.conn_status.set(ConnStatus::Error);
                    } else {
                        connected(state);
                    }
                    if let Some(idx) = idx {
                        // Refresh the viewed week even when it was cached; load_all
                        // fetched today's week, which may be a different one.
                        if fetched_idx != Some(idx) {
                            request_week(state, idx as i32, true);
                        }
                        prefetch_week(state, idx as i32 - 1);
                        prefetch_week(state, idx as i32 + 1);
                    }
                }
                Err(e) => {
                    nslog::nslog(&format!("[Diary] {e}"));
                    state.error_msg.set(e);
                    state
                        .conn_status
                        .set(if state.week_cache.with(|c| !c.is_empty()) {
                            ConnStatus::Offline
                        } else {
                            ConnStatus::Error
                        });
                }
            }
        });
    });
}

fn fetch_all(token: &str, selected: usize, stored_profile: &str) -> Result<Loaded, String> {
    let mut client = blocking::build_client(token);
    let user = match blocking::get_user_me(&client) {
        Ok(u) => u,
        Err(e) if e.starts_with("HTTP 401") => {
            let token = auth::try_refresh_token()
                .ok_or("Сессия не обновлена. Проверьте соединение или войдите заново.")?;
            client = blocking::build_client(&token);
            blocking::get_user_me(&client)?
        }
        Err(e) => return Err(e),
    };
    let children = user.children();
    let child_idx = children
        .iter()
        .position(|c| c.profile_id == stored_profile)
        .unwrap_or(selected.min(children.len().saturating_sub(1)));
    let (school, profile, name, school_name) = if user.is_parent() {
        let child = children
            .get(child_idx)
            .ok_or("В аккаунте нет доступных учеников")?;
        let name = blocking::get_school_by_id(&client, &child.school_id)
            .ok()
            .flatten()
            .and_then(|s| s.short_name.or(s.short_name_be))
            .unwrap_or_else(|| user.school_name.clone());
        (
            child.school_id.clone(),
            child.profile_id.clone(),
            child.full_name.clone(),
            name,
        )
    } else {
        (
            user.school_id.clone(),
            user.profile_id.clone(),
            user.full_name.clone(),
            user.school_name.clone(),
        )
    };
    let year = blocking::api_get::<SchoolYear>(&client, endpoints::SCHOOL_YEAR).ok();
    let today = day::datetime::DayDate::today();
    let (start, end) = year
        .as_ref()
        .map(|y| (y.start_year, y.end_year))
        .unwrap_or_else(|| academic_year(today.year as u32, today.month as u32));
    let body = serde_json::json!({"school_period": year.as_ref().map(|y| &y.uuid), "from": format!("01.09.{start:04}"), "to": format!("31.05.{end:04}")});
    let classes =
        blocking::api_post::<ClassesByDate>(&client, &endpoints::classes(&school, &profile), &body)
            .map(|c| c.classes)
            .or_else(|_| {
                blocking::api_get::<Vec<Class>>(&client, &endpoints::classes(&school, &profile))
            })?;
    let class = classes.into_iter().next().ok_or("Класс не найден")?;
    let weeks = blocking::api_get::<Vec<WeekActivity>>(&client, endpoints::WEEK_ACTIVITIES)?;
    let periods = blocking::api_get::<Vec<TimeActivity>>(&client, endpoints::TIME_ACTIVITIES)
        .unwrap_or_default();
    let mut partial_error = None;
    let current = find_current_week(&weeks, now_ms()).and_then(|idx| {
        match fetch_week(&client, &school, &class.uuid, &profile, &weeks[idx].uuid) {
            Ok(days) => Some((idx, days)),
            Err(e) => {
                partial_error = Some(e);
                None
            }
        }
    });
    let mut optional = |error: String| {
        if partial_error.is_none() {
            partial_error = Some(error);
        }
    };
    let bells = blocking::api_get::<Vec<BellSchedule>>(&client, &endpoints::bells(&school))
        .map(|v| {
            v.into_iter()
                .next()
                .and_then(|s| s.days_of_week.into_iter().next())
                .map(|d| d.time_of_bells)
                .unwrap_or_default()
        })
        .map_err(&mut optional)
        .ok();
    let timetable =
        blocking::api_get::<Vec<Timetable>>(&client, &endpoints::timetable(&school, &class.uuid))
            .map(|v| {
                v.into_iter()
                    .next()
                    .map(|t| t.days_of_week)
                    .unwrap_or_default()
            })
            .map_err(&mut optional)
            .ok();
    let teachers = blocking::api_get::<Vec<SubjectWithTeacher>>(
        &client,
        &endpoints::subjects(&school, &profile, &class.uuid),
    )
    .map_err(&mut optional)
    .ok();
    let titles: HashMap<String, String> = teachers
        .as_ref()
        .map(|v| {
            v.iter()
                .map(|s| (s.id.clone(), s.subject_title.clone()))
                .collect()
        })
        .unwrap_or_default();
    let finals = if titles.is_empty() {
        None
    } else {
        blocking::api_get_raw(
            &client,
            &endpoints::final_marks(&school, &class.uuid, &profile),
        )
        .map_err(&mut optional)
        .ok()
        .and_then(|raw| parse_final_marks(&raw, &titles))
    };
    Ok(Loaded {
        user,
        child_idx,
        school,
        profile,
        name,
        school_name,
        class,
        weeks,
        periods,
        current,
        bells,
        timetable,
        teachers,
        finals,
        partial_error,
    })
}

fn fetch_week(
    client: &reqwest::blocking::Client,
    school: &str,
    class: &str,
    profile: &str,
    week: &str,
) -> Result<Vec<DaySchedule>, String> {
    blocking::api_get(client, &endpoints::lessons(school, class, profile, week))
}

fn connected(state: AppState) {
    // Do not hide a failed sibling request with a successful neighbor prefetch.
    if matches!(
        state.conn_status.get(),
        ConnStatus::Error | ConnStatus::Offline
    ) {
        return;
    }
    state.conn_status.set(ConnStatus::Connected);
    let ticket = session::ticket();
    day::reactive::on_main_delayed(2000, move || {
        if ticket.is_current() {
            if let Some(state) = AppState::get_main() {
                if state.conn_status.get() == ConnStatus::Connected {
                    state.conn_status.set(ConnStatus::Idle);
                }
            }
        }
    });
}

pub fn prefetch_week(state: AppState, idx: i32) {
    if idx < 0 || state.week_cache.with(|c| c.contains_key(&idx)) {
        return;
    }
    request_week(state, idx, false);
}

pub fn load_week(state: AppState, idx: i32) {
    if idx < 0 || idx as usize >= state.all_weeks.with(|w| w.len()) {
        return;
    }
    select_week(state, idx as usize);
    state.conn_status.set(ConnStatus::Connecting);
    rebuild_marks(state);
    state.lessons.set(
        state
            .week_cache
            .with(|c| c.get(&idx).cloned().unwrap_or_default()),
    );
    request_week(state, idx, true);
    prefetch_week(state, idx - 1);
    prefetch_week(state, idx + 1);
}

fn request_week(state: AppState, idx: i32, visible: bool) {
    let Some(week) = state.all_weeks.with(|w| w.get(idx as usize).cloned()) else {
        return;
    };
    let Some(token) = auth::get_token() else {
        return;
    };
    let (school, class, profile) = auth::get_stored_ids();
    if school.is_empty() || class.is_empty() || profile.is_empty() {
        return;
    }
    let ticket = session::ticket();
    let order = response_order();
    accept_week(&week.uuid, order);
    let request = if visible {
        WEEK_REQUEST.fetch_add(1, Ordering::AcqRel) + 1
    } else {
        WEEK_REQUEST.load(Ordering::Acquire)
    };
    if visible {
        state
            .lessons_loading
            .set(!state.week_cache.with(|c| c.contains_key(&idx)));
        if !matches!(
            state.conn_status.get(),
            ConnStatus::Error | ConnStatus::Offline
        ) {
            state.conn_status.set(ConnStatus::Connecting);
        }
    }
    std::thread::spawn(move || {
        let result = fetch_week(
            &blocking::build_client(&token),
            &school,
            &class,
            &profile,
            &week.uuid,
        );
        ticket.publish(move |state| {
            // A refreshed list may have changed the week at this index.
            if !state
                .all_weeks
                .with(|w| w.get(idx as usize).is_some_and(|w| w.uuid == week.uuid))
            {
                return;
            }
            let active = visible
                && WEEK_REQUEST.load(Ordering::Acquire) == request
                && state.current_week_index.get() == idx;
            match result {
                Ok(days) => {
                    if !accept_week(&week.uuid, order) {
                        if active {
                            state.lessons_loading.set(false);
                        }
                        return;
                    }
                    super::grade_checker::observe_week(&week.uuid, &days);
                    state.week_cache.update(|wc| {
                        wc.insert(idx, days.clone());
                    });
                    cache::save_json("week_cache", &state.week_cache.get());
                    if state.current_week_index.get() == idx {
                        state.lessons.set(days);
                    }
                    rebuild_marks(state);
                    if active {
                        connected(state);
                    }
                }
                Err(e) => {
                    if active {
                        state.error_msg.set(e);
                        state.conn_status.set(ConnStatus::Error);
                    }
                }
            }
            if active {
                state.lessons_loading.set(false);
            }
        });
    });
}

pub fn quarter_week_indices(state: AppState, quarter: usize) -> Vec<usize> {
    state.week_quarters.with(|v| {
        v.iter()
            .enumerate()
            .filter_map(|(i, q)| (*q == Some(quarter)).then_some(i))
            .collect()
    })
}

pub fn extract_marks_for_quarter(
    cache: &WeekCache,
    quarters: &[Option<usize>],
    quarter: usize,
) -> Marks {
    let mut marks = Marks::new();
    for (idx, days) in cache {
        if quarter < 4 && quarters.get(*idx as usize).copied().flatten() != Some(quarter) {
            continue;
        }
        for day in days {
            for slot in &day.slots {
                if let Some(raw) = slot.lesson_mark.as_ref().and_then(|m| m.mark.as_deref()) {
                    marks
                        .entry(slot.subject_title.clone())
                        .or_default()
                        .extend(format::parse_marks(raw));
                }
            }
        }
    }
    marks
}

pub fn extract_marks_dated(
    cache: &WeekCache,
    quarters: &[Option<usize>],
    quarter: usize,
    subject: &str,
) -> Vec<(u64, String, Vec<f64>)> {
    let mut out = Vec::new();
    for (idx, days) in cache {
        if quarter < 4 && quarters.get(*idx as usize).copied().flatten() != Some(quarter) {
            continue;
        }
        for day in days {
            for slot in &day.slots {
                if slot.subject_title != subject {
                    continue;
                }
                let Some(raw) = slot.lesson_mark.as_ref().and_then(|m| m.mark.as_ref()) else {
                    continue;
                };
                let parsed = format::parse_marks(raw);
                if !parsed.is_empty() {
                    out.push((day.date, raw.clone(), parsed));
                }
            }
        }
    }
    out.sort_by_key(|e| e.0);
    out
}

fn rebuild_marks(state: AppState) {
    let quarters = state.week_quarters.get();
    let q_all: Vec<_> = state.week_cache.with(|c| {
        (0..4)
            .map(|q| extract_marks_for_quarter(c, &quarters, q))
            .collect()
    });
    let q = state.current_quarter.get();
    let marks = if q < 4 {
        q_all[q].clone()
    } else {
        state
            .week_cache
            .with(|c| extract_marks_for_quarter(c, &quarters, 4))
    };
    state.all_marks.set(
        marks
            .iter()
            .flat_map(|(subject, values)| values.iter().map(move |value| (subject.clone(), *value)))
            .collect(),
    );
    state.quarter_marks.set(marks);
    state.year_quarter_data.set(
        q_all
            .iter()
            .enumerate()
            .map(|(q, m)| {
                (
                    format!("{} четверть", ["I", "II", "III", "IV"][q]),
                    m.clone(),
                )
            })
            .collect(),
    );
    state.quarter_all_marks.set(q_all.clone());
    cache::save_json("quarter_all_marks", &q_all);
}

pub fn load_quarter(state: AppState, quarter: usize) {
    if quarter > 3 {
        return;
    }
    state.current_quarter.set(quarter);
    let indices = quarter_week_indices(state, quarter);
    if indices.is_empty() {
        state.quarter_marks.set(Marks::new());
        MARKS_REQUEST.fetch_add(1, Ordering::AcqRel);
        state.marks_loading.set(false);
        state
            .error_msg
            .set("Не удалось определить учебные недели четверти по календарю школы".into());
        state.conn_status.set(ConnStatus::Error);
        return;
    }
    load_mark_weeks(state, indices);
}

pub fn load_year(state: AppState) {
    state.current_quarter.set(4);
    load_mark_weeks(state, (0..state.all_weeks.with(|w| w.len())).collect());
}

fn load_mark_weeks(state: AppState, indices: Vec<usize>) {
    let Some(token) = auth::get_token() else {
        return;
    };
    let (school, class, profile) = auth::get_stored_ids();
    if class.is_empty() {
        return;
    }
    let ticket = session::ticket();
    let request = MARKS_REQUEST.fetch_add(1, Ordering::AcqRel) + 1;
    let weeks = state.all_weeks.get();
    let info: Vec<_> = indices
        .into_iter()
        .filter_map(|i| {
            weeks.get(i).map(|w| {
                let order = response_order();
                accept_week(&w.uuid, order);
                (i, w.uuid.clone(), order)
            })
        })
        .collect();
    rebuild_marks(state);
    state.marks_loading.set(true);
    state.conn_status.set(ConnStatus::Connecting);
    std::thread::spawn(move || {
        let client = blocking::build_client(&token);
        let mut last_error = None;
        for (idx, uuid, order) in info {
            if !ticket.is_current() || MARKS_REQUEST.load(Ordering::Acquire) != request {
                return;
            }
            match fetch_week(&client, &school, &class, &profile, &uuid) {
                Ok(days) => ticket.publish(move |state| {
                    if !state
                        .all_weeks
                        .with(|w| w.get(idx).is_some_and(|w| w.uuid == uuid))
                    {
                        return;
                    }
                    if !accept_week(&uuid, order) {
                        return;
                    }
                    super::grade_checker::observe_week(&uuid, &days);
                    state.week_cache.update(|wc| {
                        wc.insert(idx as i32, days);
                    });
                    cache::save_json("week_cache", &state.week_cache.get());
                    rebuild_marks(state);
                }),
                Err(e) => last_error = Some(e),
            }
        }
        ticket.publish(move |state| {
            if MARKS_REQUEST.load(Ordering::Acquire) != request {
                return;
            }
            rebuild_marks(state);
            state.marks_loading.set(false);
            if let Some(e) = last_error {
                state.error_msg.set(e);
                state.conn_status.set(ConnStatus::Error);
            } else {
                connected(state);
            }
        });
    });
}

pub fn reset_marks(state: AppState) {
    state.all_marks.set(Vec::new());
    state.loaded_mark_weeks.set(Default::default());
    state.quarter_marks.set(Marks::new());
    state.quarter_all_marks.set(Vec::new());
}

pub fn switch_child(state: AppState, child_idx: usize) {
    let Some(child) = state.parent_children.with(|c| c.get(child_idx).cloned()) else {
        return;
    };
    if child_idx == state.active_child_index.get() {
        return;
    }
    session::invalidate();
    clear_data(state);
    state.active_child_index.set(child_idx);
    // Select the new namespace BEFORE restoring any cache. Never reuse the
    // previous child's class ID as a fallback.
    auth::store_ids(
        &child.school_id,
        "",
        &child.profile_id,
        &child.full_name,
        "",
    );
    load_all(state);
}

fn parse_final_marks(
    raw: &str,
    uuid_to_title: &std::collections::HashMap<String, String>,
) -> Option<FinalMarks> {
    let v: serde_json::Value = serde_json::from_str(raw).ok()?;
    let marks = v.get("marks")?.as_object()?;
    let mut out: FinalMarks = std::collections::HashMap::new();
    for (uuid, periods) in marks {
        let Some(title) = uuid_to_title.get(uuid) else {
            continue;
        };
        let Some(periods) = periods.as_object() else {
            continue;
        };
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
    fn marked_days(raw: &str) -> Vec<DaySchedule> {
        serde_json::from_value(serde_json::json!([{
            "day_of_week": 1, "date": 1794171600000_u64,
            "slots": [{"lesson_uuid":"lesson", "subject_title":"Математика", "lesson_mark":{"mark":raw}}]
        }])).unwrap()
    }

    #[test]
    fn refreshing_a_week_replaces_marks_instead_of_accumulating() {
        let mut cache = WeekCache::from([(0, marked_days("8"))]);
        assert_eq!(
            extract_marks_for_quarter(&cache, &[Some(0)], 0)["Математика"],
            vec![8.0]
        );
        cache.insert(0, marked_days("8/10"));
        assert_eq!(
            extract_marks_for_quarter(&cache, &[Some(0)], 0)["Математика"],
            vec![8.0, 10.0]
        );
        // The same refresh is idempotent; a corrected/deleted grade replaces it.
        cache.insert(0, marked_days("8/10"));
        assert_eq!(
            extract_marks_for_quarter(&cache, &[Some(0)], 0)["Математика"],
            vec![8.0, 10.0]
        );
        cache.insert(0, marked_days("9"));
        assert_eq!(
            extract_marks_for_quarter(&cache, &[Some(0)], 0)["Математика"],
            vec![9.0]
        );
        assert!(extract_marks_for_quarter(&cache, &[Some(1)], 0).is_empty());
    }

    #[test]
    fn academic_year_spans_calendar_year_boundary() {
        assert_eq!(academic_year(2027, 2), (2026, 2027));
        assert_eq!(academic_year(2027, 8), (2026, 2027));
        assert_eq!(academic_year(2027, 9), (2027, 2028));
    }

    #[test]
    fn unequal_quarters_follow_period_ids_and_year_includes_late_weeks() {
        let mut weeks = Vec::new();
        for (q, count) in [8, 7, 11, 12].into_iter().enumerate() {
            for _ in 0..count {
                let mut w = week(1_790_000_000 + weeks.len() as u64 * 604800, 1_900_000_000);
                w.time_activity_id = format!("q{q}");
                weeks.push(w);
            }
        }
        let quarters = week_quarters(&weeks, &[]);
        assert_eq!(quarters[7], Some(0));
        assert_eq!(quarters[8], Some(1));
        assert_eq!(quarters[15], Some(2));
        assert_eq!(quarters[37], Some(3));
        let cache = WeekCache::from([(37, marked_days("10"))]);
        assert_eq!(
            extract_marks_for_quarter(&cache, &quarters, 3)["Математика"],
            vec![10.0]
        );
        assert_eq!(
            extract_marks_for_quarter(&cache, &quarters, 4)["Математика"],
            vec![10.0]
        );
        assert!(extract_marks_for_quarter(&cache, &quarters, 0).is_empty());
        assert_eq!(week_quarters(&[week(1, 2)], &[]), vec![None]);
    }

    #[test]
    fn older_week_response_cannot_overwrite_a_newer_one() {
        assert!(accept_week("test-response-order", 100));
        assert!(accept_week("test-response-order", 102));
        assert!(!accept_week("test-response-order", 101));
    }
}
