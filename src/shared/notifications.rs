//! Local push notifications for new grades.

#[cfg(target_os = "ios")]
pub mod ios {
    use crate::shared::nslog;

    pub fn request_permission() {
        nslog::nslog("[Notifications] Permission requested");
    }

    pub fn send_notification(title: &str, body: &str) {
        nslog::nslog(&format!("[Notifications] {title}: {body}"));
    }
}

#[cfg(not(target_os = "ios"))]
pub mod ios {
    pub fn request_permission() {}
    pub fn send_notification(_title: &str, _body: &str) {}
}
