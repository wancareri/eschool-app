//! Local push notifications for new grades.

#[cfg(target_os = "ios")]
pub mod ios {
    use crate::shared::nslog;

    /// Request notification permission from the user.
    pub fn request_permission() {
        unsafe {
            let center: objc2_user_notifications::UNUserNotificationCenter =
                objc2_user_notifications::UNUserNotificationCenter::currentNotificationCenter();
            let options = objc2_user_notifications::UNAuthorizationOptions::new()
                | objc2_user_notifications::UNAuthorizationOptions::Alert
                | objc2_user_notifications::UNAuthorizationOptions::Sound;

            let block = block2::Rc::new(move |granted: bool, _error: objc2_foundation::NSError| {
                nslog::nslog(&format!("[Notifications] Permission granted: {granted}"));
            });
            center.requestAuthorizationWithOptionsCompletionHandler(options, block);
        }
    }

    /// Send a local notification with the given title and body.
    pub fn send_notification(title: &str, body: &str) {
        unsafe {
            let center = objc2_user_notifications::UNUserNotificationCenter::currentNotificationCenter();

            let content = objc2_user_notifications::UNMutableNotificationContent::new();
            content.setTitle(&objc2_foundation::NSString::from_str(title));
            content.setBody(&objc2_foundation::NSString::from_str(body));
            content.setSound(Some(&objc2_user_notifications::UNNotificationSound::defaultSound()));

            let id = objc2_foundation::NSString::from_str(&format!(
                "grade_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ));

            let request = objc2_user_notifications::UNNotificationRequest::requestWithIdentifierContentTrigger(
                &id,
                &content,
                None,
            );

            center.addNotificationRequestWithCompletionHandler(&request, None);
            nslog::nslog(&format!("[Notifications] Sent: {title} - {body}"));
        }
    }
}

#[cfg(not(target_os = "ios"))]
pub mod ios {
    pub fn request_permission() {}
    pub fn send_notification(_title: &str, _body: &str) {}
}
