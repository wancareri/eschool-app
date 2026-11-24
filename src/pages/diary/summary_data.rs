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

pub(super) fn quarter_period(q: usize) -> &'static str {
    match q {
        0 => "FIRST_QUARTER",
        1 => "SECOND_QUARTER",
        2 => "THIRD_QUARTER",
        3 => "FOURTH_QUARTER",
        _ => "YEAR",
    }
}

pub(super) fn final_mark(state: AppState, subject: &str, period: &str) -> Option<String> {
    state.final_marks.with(|marks| {
        marks
            .get(subject)
            .and_then(|periods| periods.get(period))
            .cloned()
    })
}

pub(super) fn final_mark_text(mark: Option<String>) -> String {
    match mark {
        Some(v) => match v.parse::<f64>() {
            Ok(f) => format!("{:.0}", f),
            Err(_) => v,
        },
        None => "—".into(),
    }
}

/// The overall quarter result is based on marks already assigned by the
/// teacher, not on every individual lesson mark.  This is the number a parent
/// expects to see beneath the subject list.
pub(super) fn rounded_quarter_final_average(state: AppState, q: usize) -> Option<i32> {
    if q > 3 {
        return None;
    }
    let period = quarter_period(q);
    state.final_marks.with(|marks| {
        rounded_grade_average(
            marks
            .values()
            .filter_map(|periods| periods.get(period))
            .filter_map(|mark| mark.parse::<f64>().ok()),
        )
    })
}

fn rounded_grade_average(grades: impl IntoIterator<Item = f64>) -> Option<i32> {
    let (sum, count) = grades
        .into_iter()
        .fold((0.0, 0usize), |(sum, count), grade| (sum + grade, count + 1));
    (count > 0).then(|| (sum / count as f64).round() as i32)
}

#[cfg(test)]
mod tests {
    use super::rounded_grade_average;

    #[test]
    fn quarter_average_uses_only_assigned_grades_and_rounds_to_a_whole_mark() {
        assert_eq!(rounded_grade_average([6.0, 7.0, 8.0]), Some(7));
        assert_eq!(rounded_grade_average([7.0, 8.0]), Some(8));
        assert_eq!(rounded_grade_average(std::iter::empty()), None);
    }
}
