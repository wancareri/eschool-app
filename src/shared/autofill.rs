//! Authentication UIKit supplements and diagnostics. Day owns credential roles,
//! Return labels and same-field secure masking through its declarative InputTraits.

#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CredentialTrait {
    ContentType,
    Autocorrection,
    SpellChecking,
    InlinePrediction,
    SmartQuotes,
    SmartDashes,
    SmartInsertDelete,
    GrammarChecking,
}

#[cfg(any(target_os = "ios", test))]
fn reconcile_credential_traits(
    mut matches: impl FnMut(CredentialTrait) -> Option<bool>,
    mut restore: impl FnMut(CredentialTrait),
) -> usize {
    let mut repaired = 0;
    for kind in [
        CredentialTrait::ContentType,
        CredentialTrait::Autocorrection,
        CredentialTrait::SpellChecking,
        CredentialTrait::InlinePrediction,
        CredentialTrait::SmartQuotes,
        CredentialTrait::SmartDashes,
        CredentialTrait::SmartInsertDelete,
        CredentialTrait::GrammarChecking,
    ] {
        // None means the OS does not support this trait. Never send a setter
        // for an unsupported or unchanged trait: preserve the AutoFill session.
        if matches(kind) == Some(false) {
            restore(kind);
            repaired += 1;
        }
    }
    repaired
}

/// Deliver the navigation focus request after UIKit's completion callback.
#[cfg(any(target_os = "ios", test))]
fn complete_navigation_focus(
    cancelled: bool,
    keyboard_visible: bool,
    post: impl FnOnce(Box<dyn FnOnce()>),
    focus: impl FnOnce() + 'static,
) {
    if cancelled {
        return;
    }
    if keyboard_visible {
        focus();
    } else {
        post(Box::new(focus));
    }
}

#[cfg(target_os = "ios")]
pub mod ios {
    use day::prelude::Signal;
    use objc2::rc::{Retained, Weak};
    use objc2::runtime::{ProtocolObject, Sel};
    use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class};
    use objc2_foundation::{
        NSNotification, NSNotificationCenter, NSObject, NSObjectProtocol, NSString,
    };
    use objc2_ui_kit::{
        NSObjectUIAccessibility, UIControlEvents, UITextAutocorrectionType,
        UITextContentTypeNewPassword, UITextContentTypePassword, UITextContentTypeUsername,
        UITextField, UITextFieldDelegate, UITextInlinePredictionType,
        UITextInputTraits,
        UITextSmartDashesType, UITextSmartInsertDeleteType, UITextSmartQuotesType,
        UITextSpellCheckingType, UIView,
    };
    use std::cell::RefCell;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Role {
        Username,
        Password,
        NewPassword,
    }

    thread_local! {
        static FIELDS: RefCell<Vec<(Weak<UITextField>, Role)>> = const { RefCell::new(Vec::new()) };
        static VALUES: RefCell<Vec<(Weak<UITextField>, Signal<String>)>> = const { RefCell::new(Vec::new()) };
        static OBSERVER: RefCell<Option<Retained<InputModeObserver>>> = const { RefCell::new(None) };
    }

    // iOS 27 public UITextInputTraits API, absent from objc2-ui-kit 0.3.2.
    // UITextGrammarCheckingTypeNo = 1 (NSInteger); verified against the native
    // binding: https://learn.microsoft.com/dotnet/api/uikit.uitextgrammarcheckingtype
    // Guard BOTH selectors so deployment on iOS 16 remains supported.
    const GRAMMAR_CHECKING_NO: isize = 1;

    fn grammar_checking(field: &UITextField) -> Option<isize> {
        (field.respondsToSelector(objc2::sel!(grammarCheckingType))
            && field.respondsToSelector(objc2::sel!(setGrammarCheckingType:)))
            .then(|| unsafe { objc2::msg_send![field, grammarCheckingType] })
    }

    fn credential_traits(field: &UITextField, role: Role) -> usize {
        use super::CredentialTrait::*;
        let content_type = unsafe {
            match role {
                Role::Username => UITextContentTypeUsername,
                Role::Password => UITextContentTypePassword,
                Role::NewPassword => UITextContentTypeNewPassword,
            }
        };
        super::reconcile_credential_traits(
            |kind| match kind {
                ContentType => Some(&*field.textContentType() == content_type),
                Autocorrection => Some(field.autocorrectionType() == UITextAutocorrectionType::No),
                SpellChecking => Some(field.spellCheckingType() == UITextSpellCheckingType::No),
                SmartQuotes => Some(field.smartQuotesType() == UITextSmartQuotesType::No),
                SmartDashes => Some(field.smartDashesType() == UITextSmartDashesType::No),
                SmartInsertDelete => Some(field.smartInsertDeleteType() == UITextSmartInsertDeleteType::No),
                GrammarChecking => grammar_checking(field).map(|value| value == GRAMMAR_CHECKING_NO),
                InlinePrediction => {
                    // Both selectors are required, including on the iOS 16 target.
                    (field.respondsToSelector(objc2::sel!(inlinePredictionType))
                        && field.respondsToSelector(objc2::sel!(setInlinePredictionType:)))
                        .then(|| field.inlinePredictionType() == UITextInlinePredictionType::No)
                }
            },
            |kind| match kind {
                ContentType => field.setTextContentType(Some(content_type)),
                Autocorrection => field.setAutocorrectionType(UITextAutocorrectionType::No),
                SpellChecking => field.setSpellCheckingType(UITextSpellCheckingType::No),
                SmartQuotes => field.setSmartQuotesType(UITextSmartQuotesType::No),
                SmartDashes => field.setSmartDashesType(UITextSmartDashesType::No),
                SmartInsertDelete => field.setSmartInsertDeleteType(UITextSmartInsertDeleteType::No),
                GrammarChecking => unsafe {
                    let _: () = objc2::msg_send![field, setGrammarCheckingType: GRAMMAR_CHECKING_NO];
                },
                InlinePrediction => field.setInlinePredictionType(UITextInlinePredictionType::No),
            },
        )
    }

    fn supplemental_traits(field: &UITextField, role: Role) {
        // Day applies InputPurpose at mount and caches it. Its equality guard
        // cannot repair a later native change while the purpose stays the same.
        credential_traits(field, role);
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

        let assistant = field.inputAssistantItem();
        if assistant.allowsHidingShortcuts() {
            assistant.setAllowsHidingShortcuts(false);
        }
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
        supplemental_traits(&field, role);
        trace_traits(&field, role, "configured");
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
        reconcile_active_traits("app-language");
    }

    /// Register the same field that Day configures and masks declaratively.
    pub fn password(view: &UIView, value: Signal<String>) {
        configure(view, Role::Password);
        bind_value(view, value);
        let Some(field) = find_field(view) else {
            return;
        };
        install_return_delegate(&field);
    }

    /// Focus only after UIKit has finished positioning the destination page.
    /// Focusing a moving page makes the keyboard inherit the horizontal push.
    pub fn focus_after_navigation(role: Role, allowed: impl Fn() -> bool + 'static) {
        use std::rc::Rc;
        fn enqueue(role: Role, allowed: Rc<dyn Fn() -> bool>, retry: bool) {
            let Some(mtm) = MainThreadMarker::new() else {
                return;
            };
            if !allowed() {
                return;
            }
            let field = FIELDS.with(|fields| {
                fields.borrow().iter().rev().find_map(|(weak, known)| {
                    let field = weak.load()?;
                    (*known == role && field.isUserInteractionEnabled()).then_some(field)
                })
            });
            let bound =
                dispatch2::MainThreadBound::new((field.map(|f| Weak::new(&*f)), allowed), mtm);
            dispatch2::DispatchQueue::main().exec_async(move || {
                let Some(mtm) = MainThreadMarker::new() else {
                    return;
                };
                let (weak, allowed) = bound.get(mtm);
                if !allowed() {
                    return;
                }
                let field = weak.as_ref().and_then(Weak::load);
                let Some(field) = field.filter(|f| f.window().is_some()) else {
                    // Day may still be applying this turn's navigation patch.
                    if retry {
                        enqueue(role, allowed.clone(), false);
                    }
                    return;
                };
                let weak = Weak::new(&*field);
                let allowed = allowed.clone();
                let finish = move || {
                    if !allowed() {
                        return;
                    }
                    let Some(field) = weak.load() else { return };
                    if field.window().is_none()
                        || !field.isUserInteractionEnabled()
                        || field.isFirstResponder()
                    {
                        return;
                    }
                    // A completed or interrupted pop can precede Day's path
                    // update. Never focus the outgoing native controller.
                    let mut responder = field.nextResponder();
                    while let Some(current) = responder {
                        if let Some(vc) = current.downcast_ref::<objc2_ui_kit::UIViewController>() {
                            if let Some(nav) = vc.navigationController() {
                                if !nav
                                    .topViewController()
                                    .is_some_and(|top| std::ptr::eq(&*top, vc))
                                {
                                    return;
                                }
                            }
                            break;
                        }
                        responder = current.nextResponder();
                    }
                    field.layoutIfNeeded();
                    trace_traits(&field, role, "navigation-focus-before");
                    let focus = block2::RcBlock::new(|| {
                        field.becomeFirstResponder();
                    });
                    // Keep an already visible keyboard in place when changing
                    // from username to secure input; allow the initial rise.
                    if crate::shared::keyboard::get_keyboard_height()
                        > crate::shared::keyboard::get_safe_area_bottom()
                    {
                        UIView::performWithoutAnimation(&focus, mtm);
                    } else {
                        field.becomeFirstResponder();
                    }
                    trace_traits(&field, role, "navigation-focus-after");
                };
                let finish = Rc::new(finish);
                let complete = Rc::new(move |cancelled| {
                    let keyboard_visible = crate::shared::keyboard::get_keyboard_height()
                        > crate::shared::keyboard::get_safe_area_bottom();
                    crate::shared::nslog::nslog(&format!(
                        "[AutoFill] event=navigation-complete cancelled={cancelled} keyboard_visible={keyboard_visible}"
                    ));
                    let finish = finish.clone();
                    super::complete_navigation_focus(cancelled, keyboard_visible, move |focus| {
                        // Exit the transition callback before opening a new keyboard
                        // session. Keep native objects confined to the main thread.
                        let bound = dispatch2::MainThreadBound::new(focus, mtm);
                        dispatch2::DispatchQueue::main().exec_async(move || {
                            if let Some(mtm) = MainThreadMarker::new() {
                                bound.into_inner(mtm)();
                            }
                        });
                    }, move || finish());
                });
                let mut responder = field.nextResponder();
                while let Some(current) = responder {
                    if let Some(vc) = current.downcast_ref::<objc2_ui_kit::UIViewController>() {
                        let coordinator = vc.transitionCoordinator().or_else(|| {
                            vc.navigationController()
                                .and_then(|nav| nav.transitionCoordinator())
                        });
                        if let Some(coordinator) = coordinator {
                            use objc2_ui_kit::{
                                UIViewControllerTransitionCoordinator,
                                UIViewControllerTransitionCoordinatorContext,
                            };
                            let on_complete = complete.clone();
                            let completion = block2::RcBlock::new(
                                move |context: std::ptr::NonNull<
                                    ProtocolObject<
                                        dyn UIViewControllerTransitionCoordinatorContext,
                                    >,
                                >| {
                                    on_complete(unsafe { context.as_ref().isCancelled() });
                                },
                            );
                            if !coordinator
                                .animateAlongsideTransition_completion(None, Some(&completion))
                            {
                                complete(false);
                            }
                            return;
                        }
                        break;
                    }
                    responder = current.nextResponder();
                }
                complete(false);
            });
        }
        enqueue(role, Rc::new(allowed), true);
    }

    pub fn reset() {
        // Drop native bindings before clearing signals so old credentials cannot
        // be read back by a queued input-mode notification after logout.
        VALUES.with(|values| values.borrow_mut().clear());
        FIELDS.with(|fields| fields.borrow_mut().clear());
    }

    struct ReturnKeyIvars {
        original: Option<Retained<ProtocolObject<dyn UITextFieldDelegate>>>,
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "EschoolAuthReturnDelegate"]
        #[ivars = ReturnKeyIvars]
        struct ReturnKeyDelegate;
        unsafe impl NSObjectProtocol for ReturnKeyDelegate {}
        unsafe impl UITextFieldDelegate for ReturnKeyDelegate {
            #[unsafe(method(textFieldShouldReturn:))]
            fn should_return(&self, field: &UITextField) -> bool {
                // Emit exactly one submit without UIKit's resign/reopen cycle.
                // Day's target/action still dispatches the existing on_submit.
                field.sendActionsForControlEvents(UIControlEvents::EditingDidEndOnExit);
                false
            }
        }
        impl ReturnKeyDelegate {
            #[unsafe(method(respondsToSelector:))]
            fn responds_to_selector(&self, selector: Sel) -> bool {
                let own: bool = unsafe {
                    objc2::msg_send![super(self), respondsToSelector: selector]
                };
                own || self.ivars().original.as_ref()
                    .is_some_and(|original| original.respondsToSelector(selector))
            }

            #[unsafe(method_id(forwardingTargetForSelector:))]
            fn forwarding_target(&self, selector: Sel)
                -> Option<Retained<ProtocolObject<dyn UITextFieldDelegate>>>
            {
                // Preserve DayTarget's read-only edit policy and every other
                // optional delegate method, including future toolkit additions.
                self.ivars().original.as_ref()
                    .filter(|original| original.respondsToSelector(selector))
                    .cloned()
            }
        }
    );

    // The field owns its proxy via this association; its UIKit delegate is weak.
    // The proxy retains only the original delegate, never the field itself.
    static RETURN_PROXY_KEY: u8 = 0;

    fn install_return_delegate(field: &UITextField) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let key = std::ptr::from_ref(&RETURN_PROXY_KEY).cast();
        let object = std::ptr::from_ref(field).cast::<objc2::runtime::AnyObject>();
        unsafe {
            if !objc2::ffi::objc_getAssociatedObject(object, key).is_null() {
                return;
            }
            let allocated = ReturnKeyDelegate::alloc(mtm).set_ivars(ReturnKeyIvars {
                original: field.delegate(),
            });
            let delegate: Retained<ReturnKeyDelegate> =
                objc2::msg_send![super(allocated), init];
            objc2::ffi::objc_setAssociatedObject(
                object.cast_mut(), key,
                Retained::as_ptr(&delegate).cast_mut().cast(),
                objc2::ffi::OBJC_ASSOCIATION_RETAIN_NONATOMIC,
            );
            field.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        }
    }

    fn trace_traits(field: &UITextField, role: Role, event: &str) {
        // Deliberately never read field.text(), selection, or credential lengths.
        let language = field
            .textInputMode()
            .and_then(|mode| mode.primaryLanguage())
            .map(|value| value.to_string())
            .unwrap_or_default();
        let prediction = if field.respondsToSelector(objc2::sel!(inlinePredictionType)) {
            field.inlinePredictionType().0.to_string()
        } else {
            "unsupported".into()
        };
        let grammar = grammar_checking(field)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unsupported".into());
        crate::shared::nslog::nslog(&format!(
            "[AutoFill] event={event} role={role:?} language={language} content_type={} secure={} focused={} keyboard_type={} autocorrection={} spellcheck={} inline_prediction={} smart_quotes={} smart_dashes={} smart_insert_delete={} grammar_checking={} allows_hiding_shortcuts={}",
            field.textContentType(),
            field.isSecureTextEntry(),
            field.isFirstResponder(),
            field.keyboardType().0,
            field.autocorrectionType().0,
            field.spellCheckingType().0,
            prediction,
            field.smartQuotesType().0,
            field.smartDashesType().0,
            field.smartInsertDeleteType().0,
            grammar,
            field.inputAssistantItem().allowsHidingShortcuts(),
        ));
    }

    fn active_field() -> Option<(Retained<UITextField>, Role)> {
        FIELDS.with(|all| {
            let mut all = all.borrow_mut();
            all.retain(|(weak, _)| weak.load().is_some());
            all.iter().find_map(|(weak, role)| {
                let field = weak.load()?;
                field.isFirstResponder().then_some((field, *role))
            })
        })
    }

    fn reconcile_active_traits(event: &str) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let Some((field, role)) = active_field() else {
            crate::shared::nslog::nslog(&format!("[AutoFill] event={event} active_field=none"));
            return;
        };
        trace_traits(&field, role, &format!("{event}:received"));
        let repaired = credential_traits(&field, role);
        trace_traits(&field, role, &format!("{event}:reconciled repaired={repaired}"));

        // UIKit can finish the mode transition after notifying observers. Check
        // the SAME weak field once more after returning to the main queue. A
        // navigation/logout must not redirect this event to another input.
        let bound = dispatch2::MainThreadBound::new(Weak::new(&*field), mtm);
        let event = event.to_owned();
        dispatch2::DispatchQueue::main().exec_async(move || {
            let Some(mtm) = MainThreadMarker::new() else {
                return;
            };
            let Some(field) = bound.get(mtm).load() else {
                return;
            };
            let Some((current, current_role)) = active_field() else {
                return;
            };
            if !std::ptr::eq(&*field, &*current)
                || current_role != role
                || field.window().is_none()
                || !field.isUserInteractionEnabled()
            {
                return;
            }
            trace_traits(&field, role, &format!("{event}:settled"));
            let repaired = credential_traits(&field, role);
            trace_traits(
                &field,
                role,
                &format!("{event}:settled-reconciled repaired={repaired}"),
            );
        });
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
            fn input_mode_changed(&self, notification: &NSNotification) {
                // Restore only actual drift, immediately and after the mode
                // transition settles. The notification does not document trait resets.
                // reloadInputViews refreshes custom views, not an AutoFill session;
                // preserve the responder and keyboard type chosen by InputPurpose.
                let event = notification.name().to_string();
                reconcile_active_traits(&event);
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

#[cfg(test)]
mod credential_trait_tests {
    use super::{CredentialTrait, reconcile_credential_traits};
    use std::cell::RefCell;

    #[test]
    fn repairs_smart_formatting_alongside_intact_credential_traits() {
        let writes = RefCell::new(Vec::new());
        let repaired = reconcile_credential_traits(
            |kind| match kind {
                CredentialTrait::ContentType | CredentialTrait::Autocorrection
                | CredentialTrait::SpellChecking | CredentialTrait::InlinePrediction
                | CredentialTrait::GrammarChecking => Some(true),
                _ => Some(false),
            },
            |kind| writes.borrow_mut().push(format!("{kind:?}")),
        );
        assert_eq!(repaired, 3, "all smart formatting must be disabled on layout changes");
        assert_eq!(*writes.borrow(), ["SmartQuotes", "SmartDashes", "SmartInsertDelete"]);
    }

    #[test]
    fn repairs_supported_grammar_checking_without_touching_other_traits() {
        use CredentialTrait::*;
        let writes = RefCell::new(Vec::new());
        let repaired = reconcile_credential_traits(
            |kind| match kind {
                ContentType | Autocorrection | SpellChecking | InlinePrediction
                | SmartQuotes | SmartDashes | SmartInsertDelete => Some(true),
                GrammarChecking => Some(false),
            },
            |kind| writes.borrow_mut().push(format!("{kind:?}")),
        );
        assert_eq!(repaired, 1);
        assert_eq!(*writes.borrow(), ["GrammarChecking"]);
    }

    #[test]
    fn unchanged_credential_traits_never_invoke_native_setters() {
        // Reassigning even an unchanged content type can disturb AutoFill.
        for _ in 0..4 {
            assert_eq!(reconcile_credential_traits(
                |_| Some(true),
                |_| panic!("an intact credential session must receive no setters"),
            ), 0);
        }
    }

    #[test]
    fn repairs_only_drifted_traits_and_is_idle_on_the_next_input_mode_event() {
        use CredentialTrait::*;
        // Username content type is still correct; correction settings drifted.
        let state = RefCell::new([Some(true), Some(false), Some(false), Some(false), Some(true), Some(true), Some(true), Some(true)]);
        let writes = RefCell::new(Vec::new());
        let reconcile = || reconcile_credential_traits(
            |kind| state.borrow()[kind as usize],
            |kind| {
                writes.borrow_mut().push(kind);
                state.borrow_mut()[kind as usize] = Some(true);
            },
        );
        assert_eq!(reconcile(), 3);
        assert_eq!(*writes.borrow(), [Autocorrection, SpellChecking, InlinePrediction]);
        assert_eq!(reconcile(), 0);
        assert_eq!(writes.borrow().len(), 3);

        state.borrow_mut()[ContentType as usize] = Some(false);
        assert_eq!(reconcile(), 1);
        assert_eq!(writes.borrow().last(), Some(&ContentType));
    }

    #[test]
    fn unsupported_prediction_and_grammar_do_not_invoke_their_setters() {
        use CredentialTrait::*;
        let writes = RefCell::new(Vec::new());
        assert_eq!(reconcile_credential_traits(
            |kind| match kind {
                ContentType | SpellChecking | SmartQuotes | SmartDashes | SmartInsertDelete => Some(true),
                Autocorrection => Some(false),
                InlinePrediction | GrammarChecking => None, // iOS 16 lacks both APIs.
            },
            |kind| writes.borrow_mut().push(kind),
        ), 1);
        assert_eq!(*writes.borrow(), [Autocorrection]);
    }
}

#[cfg(test)]
mod navigation_focus_tests {
    use super::complete_navigation_focus;
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };
    type Pending = Rc<RefCell<Vec<Box<dyn FnOnce()>>>>;

    #[test]
    fn first_keyboard_focus_waits_until_after_completion_and_rechecks_route() {
        let queue: Pending = Rc::default();
        let deliveries = queue.clone();
        let allowed = Rc::new(Cell::new(true));
        let guard = allowed.clone();
        let focused = Rc::new(Cell::new(false));
        let focus = focused.clone();
        complete_navigation_focus(
            false,
            false,
            move |f| deliveries.borrow_mut().push(f),
            move || {
                if guard.get() {
                    focus.set(true);
                }
            },
        );
        assert!(
            !focused.get(),
            "initial focus must not run inside transition completion"
        );
        assert_eq!(queue.borrow().len(), 1);
        allowed.set(false); // Navigation/logout happened before the queued delivery.
        let callback = queue.borrow_mut().pop().unwrap();
        callback();
        assert!(!focused.get(), "stale route must not receive delayed focus");

        let queue: Pending = Rc::default();
        let deliveries = queue.clone();
        let focus = focused.clone();
        complete_navigation_focus(
            false,
            false,
            move |f| deliveries.borrow_mut().push(f),
            move || focus.set(true),
        );
        let callback = queue.borrow_mut().pop().unwrap();
        callback();
        assert!(
            focused.get(),
            "a valid request must receive focus after completion"
        );
    }

    #[test]
    fn visible_keyboard_handoff_is_immediate_and_cancelled_navigation_never_focuses() {
        let focused = Rc::new(Cell::new(0));
        let focus = focused.clone();
        complete_navigation_focus(
            false,
            true,
            |_| panic!("visible keyboard handoff must not be delayed"),
            move || focus.set(focus.get() + 1),
        );
        assert_eq!(focused.get(), 1);
        complete_navigation_focus(
            true,
            false,
            |_| panic!("cancelled navigation must not enqueue focus"),
            || panic!("cancelled navigation must not focus"),
        );
    }
}
