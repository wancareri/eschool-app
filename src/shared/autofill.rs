//! Explicit AutoFill roles for authentication inputs. Apply after Day realizes
//! the UITextField, because its default traits otherwise override our settings.
#[cfg(target_os = "ios")]
pub mod ios {
    use day::prelude::Signal;
    use objc2::rc::{Retained, Weak};
    use objc2::{MainThreadMarker, MainThreadOnly, Message, define_class};
    use objc2_foundation::{
        NSNotification, NSNotificationCenter, NSObject, NSObjectProtocol, NSString,
    };
    use objc2_ui_kit::{
        UIReturnKeyType, UITextAutocapitalizationType, UITextAutocorrectionType, UITextField,
        UITextInput, UITextInputTraits, UITextSmartDashesType, UITextSmartQuotesType,
        UITextSpellCheckingType, UIView,
    };
    use std::cell::RefCell;

    #[derive(Clone, Copy)]
    pub enum Role {
        Username,
        Password,
        NewPassword,
    }

    thread_local! {
        static FIELDS: RefCell<Vec<(Weak<UITextField>, Role)>> = const { RefCell::new(Vec::new()) };
        static OBSERVER: RefCell<Option<Retained<InputModeObserver>>> = const { RefCell::new(None) };
        static LAST_MODE: RefCell<Option<(usize, String)>> = const { RefCell::new(None) };
    }

    fn traits(field: &UITextField, role: Role) {
        let content = match role {
            Role::Username => "username",
            Role::Password => "password",
            Role::NewPassword => "newPassword",
        };
        field.setTextContentType(Some(&NSString::from_str(content)));
        if let Role::NewPassword = role {
            if let Some(mtm) = MainThreadMarker::new() {
                // The parent API accepts 12–15 characters and requires a letter
                // in each case, a digit and a permitted special character.
                let descriptor = NSString::from_str(
                    "minlength: 12; maxlength: 15; required: lower; required: upper; required: digit; required: [!]; allowed: lower, upper, digit, [!];",
                );
                let rules = objc2_ui_kit::UITextInputPasswordRules::passwordRulesWithDescriptor(
                    &descriptor,
                    mtm,
                );
                field.setPasswordRules(Some(&rules));
            }
        }

        field.setAutocapitalizationType(UITextAutocapitalizationType::None);
        // Day sets No during creation, after setSecureTextEntry. Configure here
        // instead of globally changing every text field in the application.
        field.setAutocorrectionType(UITextAutocorrectionType::Yes);
        field.setSpellCheckingType(UITextSpellCheckingType::No);
        field.setSmartQuotesType(UITextSmartQuotesType::No);
        field.setSmartDashesType(UITextSmartDashesType::No);
        field.setReturnKeyType(match role {
            Role::Username => UIReturnKeyType::Next,
            Role::Password => UIReturnKeyType::Go,
            Role::NewPassword => UIReturnKeyType::Done,
        });
        field.inputAssistantItem().setAllowsHidingShortcuts(false);
    }

    fn find_field(view: &UIView) -> Option<Retained<UITextField>> {
        if let Some(field) = view.downcast_ref::<UITextField>() {
            return Some(field.retain());
        }
        view.subviews().iter().find_map(|child| find_field(&child))
    }

    pub fn configure(view: &UIView, role: Role) {
        let Some(field) = find_field(view) else {
            return;
        };
        traits(&field, role);
        FIELDS.with(|all| {
            let mut all = all.borrow_mut();
            all.retain(|(weak, _)| weak.load().is_some_and(|f| !std::ptr::eq(&*f, &*field)));
            all.push((Weak::new(&*field), role));
        });
        install_observer();
    }

    /// Toggle masking on ONE field. Replacing a secure field with a plain field
    /// loses focus, selection, and the password role used by Password AutoFill.
    pub fn password(view: &UIView, shown: Signal<bool>) {
        configure(view, Role::Password);
        let Some(field) = find_field(view) else {
            return;
        };
        let weak = Weak::new(&*field);
        day::reactive::bind(
            move || shown.get(),
            move |&shown| {
                let Some(field) = weak.load() else { return };
                if field.isSecureTextEntry() == !shown {
                    return;
                }
                let text = field.text();
                let selection = field.selectedTextRange();
                field.setSecureTextEntry(!shown);
                field.setText(text.as_deref());
                if let Some(selection) = selection {
                    field.setSelectedTextRange(Some(&selection));
                }
                traits(&field, Role::Password);
            },
        );
    }

    fn restore_current_traits() {
        let current = FIELDS.with(|all| {
            let mut all = all.borrow_mut();
            all.retain(|(weak, _)| weak.load().is_some());
            all.iter().find_map(|(weak, role)| {
                let field = weak.load()?;
                field.isFirstResponder().then_some((field, *role))
            })
        });
        let Some((field, role)) = current else { return };
        let language = field
            .textInputMode()
            .and_then(|mode| mode.primaryLanguage())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let key = (Retained::as_ptr(&field) as usize, language);
        let changed = LAST_MODE.with(|last| {
            if last.borrow().as_ref() == Some(&key) {
                false
            } else {
                *last.borrow_mut() = Some(key);
                true
            }
        });
        traits(&field, role);
        if changed {
            field.reloadInputViews();
        }
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "EschoolInputModeObserver"]
        struct InputModeObserver;
        unsafe impl NSObjectProtocol for InputModeObserver {}
        impl InputModeObserver {
            #[unsafe(method(inputModeChanged:))]
            fn input_mode_changed(&self, _notification: &NSNotification) {
                // UIKit has to finish changing input modes before traits and
                // the assistant bar can be reconciled. Never resign/refocus.
                dispatch2::DispatchQueue::main().exec_async(restore_current_traits);
            }
        }
    );

    fn install_observer() {
        OBSERVER.with(|slot| {
            if slot.borrow().is_some() {
                return;
            }
            let Some(mtm) = MainThreadMarker::new() else {
                return;
            };
            let observer: Retained<InputModeObserver> =
                unsafe { objc2::msg_send![InputModeObserver::alloc(mtm), init] };
            let center = NSNotificationCenter::defaultCenter();
            unsafe {
                center.addObserver_selector_name_object(
                    &observer,
                    objc2::sel!(inputModeChanged:),
                    Some(objc2_ui_kit::UITextInputCurrentInputModeDidChangeNotification),
                    None,
                );
                center.addObserver_selector_name_object(
                    &observer,
                    objc2::sel!(inputModeChanged:),
                    Some(objc2_ui_kit::UITextFieldTextDidBeginEditingNotification),
                    None,
                );
            }
            *slot.borrow_mut() = Some(observer);
        });
    }
}

/// Used by the new-password and confirmation inputs on both platforms.
pub fn new_password(node: day::RNode) {
    #[cfg(target_os = "ios")]
    let _ = day_uikit::with_native(node, |view, _, _| {
        ios::configure(view, ios::Role::NewPassword)
    });
    #[cfg(not(target_os = "ios"))]
    let _ = node;
}

pub fn current_password(node: day::RNode) {
    #[cfg(target_os = "ios")]
    let _ = day_uikit::with_native(node, |view, _, _| ios::configure(view, ios::Role::Password));
    #[cfg(not(target_os = "ios"))]
    let _ = node;
}
