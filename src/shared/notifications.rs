//! Local notifications for server-observed grade changes.
#[cfg(target_os = "ios")]
pub mod ios {
    use crate::shared::nslog;
    use block2::{DynBlock, RcBlock};
    use objc2::{AnyThread, define_class, rc::Retained, runtime::ProtocolObject};
    use objc2_foundation::{NSError, NSObject, NSObjectProtocol, NSString};
    use objc2_user_notifications::*;

    define_class!(
        #[unsafe(super(NSObject))]
        #[name = "EschoolNotificationDelegate"]
        struct NotificationDelegate;
        unsafe impl NSObjectProtocol for NotificationDelegate {}
        unsafe impl UNUserNotificationCenterDelegate for NotificationDelegate {
            #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
            fn will_present(
                &self,
                _center: &UNUserNotificationCenter,
                _notification: &UNNotification,
                completion: &DynBlock<dyn Fn(UNNotificationPresentationOptions)>,
            ) {
                completion.call((UNNotificationPresentationOptions::Banner
                    | UNNotificationPresentationOptions::Sound,));
            }
        }
    );
    thread_local! {
        // UNUserNotificationCenter keeps a weak reference to its delegate.
        static DELEGATE: Retained<NotificationDelegate> = unsafe {
            objc2::msg_send![NotificationDelegate::alloc(), init]
        };
    }

    pub fn request_permission() {
        let center = UNUserNotificationCenter::currentNotificationCenter();
        DELEGATE.with(|delegate| center.setDelegate(Some(ProtocolObject::from_ref(&**delegate))));
        let completion = RcBlock::new(|_granted: objc2::runtime::Bool, error: *mut NSError| {
            if !error.is_null() {
                nslog::nslog("[Notifications] Permission request failed");
            }
        });
        center.requestAuthorizationWithOptions_completionHandler(
            UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
            &completion,
        );
    }

    pub fn send_notification(title: &str, body: &str) {
        let content = UNMutableNotificationContent::new();
        content.setTitle(&NSString::from_str(title));
        content.setBody(&NSString::from_str(body));
        content.setSound(Some(&UNNotificationSound::defaultSound()));
        let id = NSString::from_str(&format!("eschool.grade.{}", rand::random::<u64>()));
        let request =
            UNNotificationRequest::requestWithIdentifier_content_trigger(&id, &content, None);
        let completion = RcBlock::new(|error: *mut NSError| {
            if !error.is_null() {
                nslog::nslog("[Notifications] Scheduling failed");
            }
        });
        UNUserNotificationCenter::currentNotificationCenter()
            .addNotificationRequest_withCompletionHandler(&request, Some(&completion));
    }
}

#[cfg(not(target_os = "ios"))]
pub mod ios {
    pub fn request_permission() {}
    pub fn send_notification(_title: &str, _body: &str) {}
}
