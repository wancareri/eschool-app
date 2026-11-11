//! Long-press access to the same detail action exposed by a tap.
use day::prelude::*;

pub fn attach(piece: impl Piece, action: impl Fn() + 'static) -> AnyPiece {
    #[cfg(target_os = "ios")]
    {
        use day_uikit::UiKitExt;
        // Create an explicit native hit-test container before attaching the recognizer.
        piece
            .background(Color::rgba(0.0, 0.0, 0.0, 0.0))
            .uikit(move |view, _, mtm| native::install(view, mtm, action))
            .any()
    }
    #[cfg(not(target_os = "ios"))]
    {
        piece
            .context_menu(vec![menu_item("…").action(action)])
            .any()
    }
}

#[cfg(target_os = "ios")]
mod native {
    use objc2::rc::Retained;
    use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
    use objc2_foundation::NSObject;
    use objc2_ui_kit::{
        UIGestureRecognizer, UIGestureRecognizerState, UILongPressGestureRecognizer, UIView,
    };

    define_class!(
        #[unsafe(super(UILongPressGestureRecognizer, UIGestureRecognizer, NSObject))]
        #[thread_kind = MainThreadOnly]
        #[ivars = Box<dyn Fn()>]
        struct DetailLongPress;
        impl DetailLongPress {
            #[unsafe(method(invokeDetail:))]
            fn invoke_detail(&self, _sender: &UIGestureRecognizer) {
                if self.state() == UIGestureRecognizerState::Began { (self.ivars())(); }
            }
        }
    );
    pub fn install(view: &UIView, mtm: MainThreadMarker, action: impl Fn() + 'static) {
        let allocated = DetailLongPress::alloc(mtm).set_ivars(Box::new(action));
        let recognizer: Retained<DetailLongPress> = unsafe {
            msg_send![super(allocated), initWithTarget: Option::<&NSObject>::None, action: Option::<objc2::runtime::Sel>::None]
        };
        unsafe {
            recognizer.addTarget_action(&recognizer, sel!(invokeDetail:));
        }
        recognizer.setCancelsTouchesInView(true);
        view.setUserInteractionEnabled(true);
        view.addGestureRecognizer(&recognizer);
    }
}
