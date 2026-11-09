//! Explicit AutoFill roles for authentication inputs. Apply after Day realizes
//! the UITextField, because its default traits otherwise override our settings.
#[cfg(target_os = "ios")]
pub mod ios {
    use day::prelude::Signal;
    use objc2::rc::{Retained, Weak};
    use objc2::runtime::ProtocolObject;
    use objc2::{MainThreadMarker, MainThreadOnly, Message, define_class};
    use objc2_foundation::{
        NSNotification, NSNotificationCenter, NSObject, NSObjectProtocol, NSString,
    };
    use objc2_ui_kit::{
        NSObjectUIAccessibility, UIAction, UIButton, UIButtonType, UIControlEvents, UIControlState,
        UIEditMenuConfiguration, UIEditMenuInteraction, UIImage, UIReturnKeyType,
        UITextAutocapitalizationType, UITextAutocorrectionType, UITextContentTypeNewPassword,
        UITextContentTypePassword, UITextContentTypeUsername, UITextField, UITextFieldDelegate,
        UITextFieldViewMode, UITextInput, UITextInputTraits, UITextSmartDashesType,
        UITextSmartQuotesType, UITextSpellCheckingType, UIView,
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
        static VALUES: RefCell<Vec<(Weak<UITextField>, Signal<String>)>> = const { RefCell::new(Vec::new()) };
        static RETURN_DELEGATE: RefCell<Option<Retained<ReturnKeyDelegate>>> = const { RefCell::new(None) };
        static OBSERVER: RefCell<Option<Retained<InputModeObserver>>> = const { RefCell::new(None) };
        static LAST_MODE: RefCell<Option<(usize, String)>> = const { RefCell::new(None) };
    }

    fn traits(field: &UITextField, role: Role) {
        let content = unsafe {
            match role {
                Role::Username => UITextContentTypeUsername,
                Role::Password => UITextContentTypePassword,
                Role::NewPassword => UITextContentTypeNewPassword,
            }
        };
        if field.textContentType().to_string() != content.to_string() {
            field.setTextContentType(Some(content));
        }
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
        if field.autocorrectionType() != UITextAutocorrectionType::Yes {
            field.setAutocorrectionType(UITextAutocorrectionType::Yes);
        }
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

    fn bind_value(view: &UIView, value: Signal<String>) {
        let Some(field) = find_field(view) else {
            return;
        };
        VALUES.with(|values| {
            let mut values = values.borrow_mut();
            values.retain(|(weak, _)| weak.load().is_some_and(|old| !std::ptr::eq(&*old, &*field)));
            values.push((Weak::new(&*field), value));
        });
    }

    pub fn username(view: &UIView, value: Signal<String>) {
        configure(view, Role::Username);
        bind_value(view, value);
        if let Some(field) = find_field(view) {
            install_return_delegate(&field);
            install_edit_menu_button(&field);
        }
    }

    pub fn capture_password(view: &UIView, value: Signal<String>) {
        configure(view, Role::Password);
        bind_value(view, value);
        // Separate capture field: never overlays the username input or takes
        // focus, and it is omitted from accessibility navigation.
        view.setUserInteractionEnabled(false);
        if let Some(mtm) = MainThreadMarker::new() {
            view.setAccessibilityElementsHidden(true, mtm);
            if let Some(field) = find_field(view) {
                field.setTextColor(Some(&objc2_ui_kit::UIColor::clearColor()));
                field.setBackgroundColor(Some(&objc2_ui_kit::UIColor::clearColor()));
                field.setAccessibilityElementsHidden(true, mtm);
                field.setIsAccessibilityElement(false, mtm);
            }
        }
    }

    /// Read before replacing the username page: credential providers may set
    /// native text without sending Day's editingChanged control event.
    pub fn sync_values() {
        let updates = VALUES.with(|values| {
            let mut values = values.borrow_mut();
            values.retain(|(weak, _)| weak.load().is_some());
            values
                .iter()
                .filter_map(|(weak, value)| {
                    Some((
                        *value,
                        weak.load()?
                            .text()
                            .map(|s| s.to_string())
                            .unwrap_or_default(),
                    ))
                })
                .collect::<Vec<_>>()
        });
        for (value, text) in updates {
            if value.get_untracked() != text {
                value.set(text);
            }
        }
    }

    pub fn refresh_active() {
        dispatch2::DispatchQueue::main().exec_async(|| {
            LAST_MODE.with(|last| *last.borrow_mut() = None);
            restore_current_traits();
        });
    }

    /// Toggle masking on ONE field. Replacing a secure field with a plain field
    /// loses focus, selection, and the password role used by Password AutoFill.
    pub fn password(view: &UIView, shown: Signal<bool>, value: Signal<String>) {
        configure(view, Role::Password);
        bind_value(view, value);
        let Some(field) = find_field(view) else {
            return;
        };
        // Realize the captured value before UIKit starts the push/focus transfer.
        field.setText(Some(&NSString::from_str(&value.get_untracked())));
        install_return_delegate(&field);
        install_edit_menu_button(&field);
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

    /// A stable entry to UIKit's text-edit menu when QuickType is collapsed.
    /// UIKit supplies its own AutoFill commands; no private Passwords selector.
    fn install_edit_menu_button(field: &UITextField) {
        if field.rightView().is_some() {
            return;
        }
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let button = UIButton::buttonWithType(UIButtonType::System, mtm);
        button.setFrame(objc2_core_foundation::CGRect::new(
            objc2_core_foundation::CGPoint::new(0.0, 0.0),
            objc2_core_foundation::CGSize::new(36.0, 36.0),
        ));
        let image = UIImage::systemImageNamed(&NSString::from_str("key"));
        button.setImage_forState(image.as_deref(), UIControlState::Normal);
        let weak = Weak::new(field);
        let handler = block2::RcBlock::new(move |_: std::ptr::NonNull<UIAction>| {
            let Some(field) = weak.load() else { return };
            field.becomeFirstResponder();
            let bound = dispatch2::MainThreadBound::new(Weak::new(&*field), mtm);
            dispatch2::DispatchQueue::main().exec_async(move || {
                let Some(mtm) = MainThreadMarker::new() else {
                    return;
                };
                let Some(field) = bound.get(mtm).load() else {
                    return;
                };
                if !field.isFirstResponder() || field.window().is_none() {
                    return;
                }
                let existing = field.interactions().iter().find_map(|interaction| {
                    let object: &objc2::runtime::AnyObject = interaction.as_ref();
                    object
                        .downcast_ref::<UIEditMenuInteraction>()
                        .map(|menu| menu.retain())
                });
                let menu = existing.unwrap_or_else(|| {
                    let menu = UIEditMenuInteraction::initWithDelegate(
                        UIEditMenuInteraction::alloc(mtm),
                        None,
                    );
                    field.addInteraction(ProtocolObject::from_ref(&*menu));
                    menu
                });
                let bounds = field.bounds();
                let configuration = unsafe {
                    UIEditMenuConfiguration::configurationWithIdentifier_sourcePoint(
                        None,
                        objc2_core_foundation::CGPoint::new(bounds.size.width / 2.0, 0.0),
                        mtm,
                    )
                };
                menu.presentEditMenuWithConfiguration(&configuration);
            });
        });
        let action = unsafe {
            UIAction::actionWithTitle_image_identifier_handler(
                &NSString::from_str(""),
                None,
                None,
                block2::RcBlock::as_ptr(&handler),
                mtm,
            )
        };
        button.addAction_forControlEvents(&action, UIControlEvents::TouchUpInside);
        let weak = Weak::new(&*button);
        day::reactive::bind(
            move || crate::res::str::auth_autofill().format(),
            move |label| {
                if let Some(button) = weak.load() {
                    button.setAccessibilityLabel(Some(&NSString::from_str(label)), mtm);
                }
            },
        );
        field.setRightView(Some(&button));
        field.setRightViewMode(UITextFieldViewMode::Always);
    }

    pub fn reset() {
        // Drop native bindings before clearing signals so old credentials cannot
        // be read back by a queued input-mode notification after logout.
        VALUES.with(|values| values.borrow_mut().clear());
        FIELDS.with(|fields| fields.borrow_mut().clear());
        LAST_MODE.with(|mode| *mode.borrow_mut() = None);
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "EschoolAuthReturnDelegate"]
        struct ReturnKeyDelegate;
        unsafe impl NSObjectProtocol for ReturnKeyDelegate {}
        unsafe impl UITextFieldDelegate for ReturnKeyDelegate {
            #[unsafe(method(textFieldShouldReturn:))]
            fn should_return(&self, field: &UITextField) -> bool {
                // Submit without UIKit's automatic resign/reopen cycle between
                // username and password. The incoming field takes focus itself.
                field.sendActionsForControlEvents(UIControlEvents::EditingDidEndOnExit);
                false
            }
        }
    );

    fn install_return_delegate(field: &UITextField) {
        // Do not replace a delegate supplied by another component.
        if field.delegate().is_some() {
            return;
        }
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        RETURN_DELEGATE.with(|slot| {
            let mut slot = slot.borrow_mut();
            let delegate = slot.get_or_insert_with(|| unsafe {
                objc2::msg_send![ReturnKeyDelegate::alloc(mtm), init]
            });
            field.setDelegate(Some(ProtocolObject::from_ref(&**delegate)));
        });
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
        // Input-mode changes do not require rebuilding the keyboard. Doing so
        // interrupts the credential suggestion session on multilingual keyboards.
        if changed {
            traits(&field, role);
        }
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "EschoolInputModeObserver"]
        struct InputModeObserver;
        unsafe impl NSObjectProtocol for InputModeObserver {}
        impl InputModeObserver {
            #[unsafe(method(textChanged:))]
            fn text_changed(&self, notification: &NSNotification) {
                let Some(object) = notification.object() else { return };
                let Some(field) = object.downcast_ref::<UITextField>() else { return };
                let value = VALUES.with(|values| values.borrow().iter().find_map(|(weak, value)| {
                    weak.load().filter(|known| std::ptr::eq(&**known, field)).map(|_| *value)
                }));
                if let Some(value) = value {
                    let text = field.text().map(|s| s.to_string()).unwrap_or_default();
                    if value.get_untracked() != text { value.set(text); }
                }
            }

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
                    objc2::sel!(textChanged:),
                    Some(objc2_ui_kit::UITextFieldTextDidChangeNotification),
                    None,
                );
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
