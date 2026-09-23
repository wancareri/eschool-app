//! Background grade checker — detects new marks and sends local notifications.

use crate::app::AppState;
use crate::shared::{nslog, notifications};
use day::prelude::Ambient;

/// Check for new grades by comparing current state with stored marks.
pub fn check_new_grades(state: AppState) {
    nslog::nslog("[GradeCheck] Checking for new grades...");

    let current_marks: Vec<String> = state.all_marks.get()
        .iter()
        .map(|(uuid, _)| uuid.clone())
        .collect();

    let stored_json = day::prefs::get("grade_check.last_marks").unwrap_or_default();
    let stored_marks: Vec<String> = serde_json::from_str(&stored_json)
        .unwrap_or_default();

    // Find new marks (in current but not in stored)
    let new_count = current_marks.iter()
        .filter(|m| !stored_marks.contains(m))
        .count();

    if new_count > 0 {
        let message = format!("Новых оценок: {new_count}");
        notifications::ios::send_notification("Eschool", &message);
        nslog::nslog(&format!("[GradeCheck] Found {new_count} new marks"));
    }

    // Store current marks
    if let Ok(json) = serde_json::to_string(&current_marks) {
        day::prefs::set("grade_check.last_marks", &json);
    }
}

/// Start background grade checking (runs every 15 minutes).
pub fn start_grade_checker() {
    nslog::nslog("[GradeCheck] Starting background checker (every 15 min)");

    std::thread::spawn(|| {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(15 * 60));

            day::reactive::on_main(|| {
                if let Some(state) = AppState::get_main() {
                    if state.is_authenticated.get() && !state.all_marks.get().is_empty() {
                        check_new_grades(state);
                    }
                }
            });
        }
    });
}
