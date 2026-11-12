//! Detect changed marks only after a successful server response for that week.
use crate::app::data::cache;
use eschool_api::entities::DaySchedule;
use std::collections::HashMap;

type Snapshot = HashMap<String, String>;
type History = HashMap<String, Snapshot>;
const HISTORY_KEY: &str = "grade_check.last_marks";

fn snapshot(days: &[DaySchedule]) -> Snapshot {
    days.iter()
        .flat_map(|d| {
            d.slots.iter().filter_map(move |slot| {
                let mark = slot.lesson_mark.as_ref()?.mark.as_ref()?;
                if crate::app::format::parse_marks(mark).is_empty() {
                    return None;
                }
                let id = if slot.lesson_uuid.is_empty() {
                    format!(
                        "{}:{}:{}:{}",
                        d.date, slot.number, slot.subject_id, slot.subject_title
                    )
                } else {
                    slot.lesson_uuid.clone()
                };
                Some((id, mark.clone()))
            })
        })
        .collect()
}

fn changed_count(previous: Option<&Snapshot>, current: &Snapshot) -> usize {
    // First observation is a baseline, including a previously unseen old week.
    previous
        .map(|old| {
            current
                .iter()
                .filter(|(id, value)| old.get(*id) != Some(*value))
                .count()
        })
        .unwrap_or(0)
}

/// Must run in the session-checked UI callback, with the selected cache namespace.
pub fn observe_week(uuid: &str, days: &[DaySchedule]) {
    let mut history: History = cache::load_json(HISTORY_KEY).unwrap_or_default();
    let current = snapshot(days);
    let count = changed_count(history.get(uuid), &current);
    if count > 0 {
        if day_part_local_notify::is_supported() {
            if let Err(error) = day_part_local_notify::Notification::new("ESchool")
                .body(format!("Новых или обновлённых оценок: {count}"))
                .channel("grades")
                .post()
            {
                crate::ios::nslog::nslog(&format!("[GradeCheck] Notification failed: {error}"));
            }
        }
    }
    history.insert(uuid.to_string(), current);
    cache::save_json(HISTORY_KEY, &history);
}


fn request_notification_access() {
    use std::sync::atomic::Ordering;
    static REQUESTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if REQUESTED.swap(true, Ordering::AcqRel) || !day_part_local_notify::is_supported() {
        return;
    }
    day_part_local_notify::Channel::new(
        "grades",
        day_part_local_notify::Importance::Default,
    )
    .name("Оценки")
    .register();
    use day_part_permissions::{Permission, Status};
    match day_part_permissions::status(Permission::Notifications) {
        Status::Granted => {}
        Status::Prompt | Status::Unknown
            if day_part_permissions::can_prompt(Permission::Notifications) => {
            day_part_permissions::request(Permission::Notifications, |_| {});
        }
        _ => {}
    }
}

static CHECKER_STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Refresh while the process is running. iOS may suspend this thread in the
/// background; this is not an OS background-fetch entitlement or remote push.
pub fn start_grade_checker() {
    use std::sync::atomic::Ordering;
    if CHECKER_STARTED.swap(true, Ordering::AcqRel) {
        return;
    }
    request_notification_access();
    std::thread::spawn(|| {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(15 * 60));
            day::reactive::on_main(|| {
                if let Some(state) = crate::app::AppState::get_main() {
                    if state.is_authenticated.get() && !state.loading.get() {
                        crate::pages::diary::data::load_all(state);
                    }
                }
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn baseline_navigation_and_repeated_refresh_do_not_notify() {
        let old = Snapshot::from([("lesson".into(), "8".into())]);
        assert_eq!(changed_count(None, &old), 0);
        assert_eq!(changed_count(Some(&old), &old), 0);
        let mut current = old.clone();
        current.insert("second".into(), "10".into());
        assert_eq!(changed_count(Some(&old), &current), 1);
        current.insert("lesson".into(), "9".into());
        assert_eq!(changed_count(Some(&old), &current), 2);
        assert_eq!(changed_count(Some(&current), &Snapshot::new()), 0);
    }
}
