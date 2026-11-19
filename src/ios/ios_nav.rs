#[cfg(feature = "uikit")]
pub mod ios_impl {
    use objc2::MainThreadMarker;
    use objc2::rc::Retained;
    use objc2_foundation::{NSArray, NSString};
    use objc2_ui_kit::{UIImage, UIView, UIViewController};

    pub fn install_auth_menu_icons() {
        use std::sync::{Once, atomic::Ordering};
        static INSTALL: Once = Once::new();
        INSTALL.call_once(|| {
            let class = objc2::class!(UIMenu);
            let Some(method) = class.class_method(objc2::sel!(menuWithTitle:children:)) else {
                return;
            };
            AUTH_MENU_ORIGINAL.store(method.implementation() as *mut (), Ordering::Release);
            unsafe {
                method.set_implementation(std::mem::transmute(auth_menu as *const ()));
            }
        });
    }

    static AUTH_MENU_ORIGINAL: std::sync::atomic::AtomicPtr<()> =
        std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

    unsafe extern "C-unwind" fn auth_menu(
        class: *const objc2::runtime::AnyClass,
        selector: objc2::runtime::Sel,
        title: &NSString,
        children: &NSArray<objc2_ui_kit::UIMenuElement>,
    ) -> *mut objc2_ui_kit::UIMenu {
        use objc2_ui_kit::{UIMenu, UIMenuOptions};
        let label = title.to_string();
        let language = matches!(label.as_str(), "Language" | "Язык" | "Мова")
            && children
                .firstObject()
                .is_some_and(|child| child.title().to_string() == "RU · Русский");
        let appearance = matches!(label.as_str(), "Appearance" | "Оформление" | "Афармленне")
            && children.len() == 3
            && children.iter().all(|child| child.image().is_some());
        if (language || appearance)
            && let Some(mtm) = MainThreadMarker::new()
        {
            let icon = if language {
                "globe"
            } else {
                "circle.lefthalf.filled"
            };
            let image = UIImage::systemImageNamed(&NSString::from_str(icon));
            let menu = UIMenu::menuWithTitle_image_identifier_options_children(
                title,
                image.as_deref(),
                None,
                UIMenuOptions::empty(),
                children,
                mtm,
            );
            return Retained::autorelease_return(menu);
        }
        let original: unsafe extern "C-unwind" fn(
            *const objc2::runtime::AnyClass,
            objc2::runtime::Sel,
            &NSString,
            &NSArray<objc2_ui_kit::UIMenuElement>,
        ) -> *mut UIMenu = unsafe {
            std::mem::transmute(AUTH_MENU_ORIGINAL.load(std::sync::atomic::Ordering::Acquire))
        };
        unsafe { original(class, selector, title, children) }
    }

    pub fn bind_auth_page_title(view: &UIView, title: fn() -> String) {
        let weak = objc2::rc::Weak::new(view);
        day::reactive::watch(
            move || day::locale().get(),
            move |_, _| {
                let Some(view) = weak.load() else { return };
                let mut responder = view.nextResponder();
                while let Some(current) = responder {
                    if let Some(vc) = current.downcast_ref::<UIViewController>() {
                        vc.navigationItem()
                            .setTitle(Some(&NSString::from_str(&title())));
                        break;
                    }
                    responder = current.nextResponder();
                }
            },
        );
    }
}
