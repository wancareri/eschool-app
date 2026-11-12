use crate::app::AppState;

pub(super) fn get_all_subject_names(state: AppState) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for t in state.subjects_teachers.get() {
        if !t.subject_title.is_empty() && !names.contains(&t.subject_title) {
            names.push(t.subject_title);
        }
    }
    for day in state.lessons.get() {
        for slot in &day.slots {
            if !slot.subject_title.is_empty() && !names.contains(&slot.subject_title) {
                names.push(slot.subject_title.clone());
            }
        }
    }
    state.week_cache.with(|cache| {
        for days in cache.values() {
            for day in days {
                for slot in &day.slots {
                    if !slot.subject_title.is_empty() && !names.contains(&slot.subject_title) {
                        names.push(slot.subject_title.clone());
                    }
                }
            }
        }
    });
    for q_map in state.quarter_all_marks.get() {
        for name in q_map.keys() {
            if !name.is_empty() && !names.contains(name) {
                names.push(name.clone());
            }
        }
    }
    for name in state.quarter_marks.get().keys() {
        if !name.is_empty() && !names.contains(name) {
            names.push(name.clone());
        }
    }
    state.final_marks.with(|fm| {
        for name in fm.keys() {
            if !name.is_empty() && !names.contains(name) {
                names.push(name.clone());
            }
        }
    });
    names.sort();
    names.dedup();
    names
}

pub(super) fn get_quarter_marks_map(
    state: AppState,
    q: usize,
) -> std::collections::HashMap<String, Vec<f64>> {
    let q_all = state.quarter_all_marks.get();
    if let Some(m) = q_all.get(q) {
        if !m.is_empty() {
            return m.clone();
        }
    }
    state
        .week_cache
        .with(|c| crate::pages::diary::data::extract_marks_for_quarter(c, &state.week_quarters.get(), q))
}

/// Period key inside `/final/whole` for a quarter column (4 = the year tab).
pub(super) fn quarter_period(q: usize) -> &'static str {
    match q {
        0 => "FIRST_QUARTER",
        1 => "SECOND_QUARTER",
        2 => "THIRD_QUARTER",
        3 => "FOURTH_QUARTER",
        _ => "YEAR",
    }
}

/// The teacher-set final mark for a subject in a period («Выставл.» / «Год»),
/// straight from `/final/whole`. An unset final is a dash — never an average.
pub(super) fn final_mark(state: AppState, subject: &str, period: &str) -> Option<String> {
    state.final_marks.with(|marks| {
        marks
            .get(subject)
            .and_then(|periods| periods.get(period))
            .cloned()
    })
}

/// Renders a final mark: a number as an integer grade, anything else as-is,
/// and an unset final as a dash.
pub(super) fn final_mark_text(mark: Option<String>) -> String {
    match mark {
        Some(v) => match v.parse::<f64>() {
            Ok(f) => format!("{:.0}", f),
            Err(_) => v,
        },
        None => "—".into(),
    }
}
