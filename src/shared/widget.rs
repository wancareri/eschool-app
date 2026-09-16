//! Widget data provider — writes schedule data to shared UserDefaults for the iOS widget.

use crate::app::AppState;

/// Write today's schedule to shared UserDefaults (App Group) for the widget.
pub fn update_widget_data(_state: AppState) {
    #[cfg(target_os = "ios")]
    {
        use crate::shared::nslog;
        use eschool_api::entities::*;

        let lessons = _state.lessons.get();
        let bell_times = _state.bell_times.get();

        // Build lesson data for widget
        let today = day_piece_datetime::DayDate::today();
        let today_str = today.to_string();

        let widget_lessons: Vec<WidgetLesson> = lessons.iter()
            .filter(|d| d.date == today_str)
            .flat_map(|d| {
                d.lessons.iter().filter_map(|slot| {
                    let subject = slot.subject.as_ref()?;
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
                let suite_name = objc2_foundation::NSString::from_str("group.by.eschool.app.shared");
                let defaults = objc2_foundation::NSUserDefaults::alloc()
                    .initWithSuiteName(&suite_name);

                let key = objc2_foundation::NSString::from_str("widget.schedule");
                let value = objc2_foundation::NSString::from_str(&json);
                defaults.setObjectForKey(Some(&value), Some(&key));

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
