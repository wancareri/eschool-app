//! Widget data provider — writes schedule data to shared UserDefaults for the iOS widget.

use crate::app::AppState;

/// Write today's schedule to shared UserDefaults (App Group) for the widget.
pub fn update_widget_data(_state: AppState) {
    #[cfg(target_os = "ios")]
    {
        use objc2::AnyThread;
        use crate::shared::nslog;

        let lessons = _state.lessons.get();
        let bell_times = _state.bell_times.get();

        // Build lesson data for widget
        let today = day_piece_datetime::DayDate::today();
        let today_epoch = today.to_epoch_days() as u64;

        let widget_lessons: Vec<WidgetLesson> = lessons.iter()
            .filter(|d| d.date == today_epoch)
            .flat_map(|d| {
                d.slots.iter().filter_map(|slot| {
                    let subject = if slot.subject.is_empty() { None } else { Some(&slot.subject) }?;
                    let time = bell_times.get(slot.number as usize - 1)
                        .map(|b| b.time_start.clone())
                        .unwrap_or_default();
                    let teacher = slot.teacher.clone().unwrap_or_default();

                    Some(WidgetLesson {
                        subject: subject.clone(),
                        time,
                        teacher,
                    })
                })
            })
            .collect();

        // Serialize to JSON
        if let Ok(json) = serde_json::to_string(&widget_lessons) {
            unsafe {
                use objc2_foundation::{NSString, NSUserDefaults};

                let suite_name = NSString::from_str("group.by.eschool.app.shared");
                let defaults = NSUserDefaults::alloc()
                    .initWithSuiteName(Some(&suite_name));

                let key = NSString::from_str("widget.schedule");
                let value = NSString::from_str(&json);
                defaults.setObjectForKey(Some(&*value), Some(&*key));

                nslog::nslog(&format!("[Widget] Updated schedule: {} lessons", widget_lessons.len()));
            }
        }
    }
}

/// Lesson data for the widget.
#[derive(serde::Serialize, serde::Deserialize)]
struct WidgetLesson {
    subject: String,
    time: String,
    teacher: String,
}
