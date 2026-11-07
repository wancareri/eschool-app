use day::prelude::*;

#[allow(dead_code)]
static DUCK_INTRO_JSON: &str = include_str!("../../resource/animations/duck_intro.json");
#[allow(dead_code)]
static DUCK_LOGIN_JSON: &str = include_str!("../../resource/animations/duck_login.json");
#[allow(dead_code)]
static DUCK_PASSWORD_JSON: &str = include_str!("../../resource/animations/duck_password.json");
#[allow(dead_code)]
static LOTTIE_JS: &str = include_str!("../../resource/animations/lottie_light.min.js");

#[allow(dead_code)]
fn build_lottie_html(json_data: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  html, body {{
    width: 100%; height: 100%;
    overflow: hidden;
    background: transparent !important;
    background-color: transparent !important;
    display: flex;
    align-items: center;
    justify-content: center;
  }}
  #anim {{
    width: 100%;
    height: 100%;
  }}
  svg {{
    display: block;
    width: 100% !important;
    height: 100% !important;
  }}
</style>
</head>
<body>
<div id="anim"></div>
<script>
{LOTTIE_JS}
</script>
<script>
try {{
  var animData = {json_data};
  lottie.loadAnimation({{
    container: document.getElementById('anim'),
    renderer: 'svg',
    loop: true,
    autoplay: true,
    animationData: animData
  }});
}} catch (e) {{
  console.error("Lottie load error:", e);
}}
</script>
</body>
</html>"#
    )
}

pub fn duck_intro_animation(size: f64) -> impl Piece {
    lottie_view_raw(DUCK_INTRO_JSON, size)
}

pub fn duck_login_animation(size: f64) -> impl Piece {
    lottie_view_raw(DUCK_LOGIN_JSON, size)
}

pub fn duck_password_animation(size: f64) -> impl Piece {
    lottie_view_raw(DUCK_PASSWORD_JSON, size)
}

pub fn login_animation_view() -> impl Piece {
    duck_intro_animation(130.0)
}

pub fn login_animation_view_sized(size: f64) -> impl Piece {
    duck_intro_animation(size)
}

pub fn lottie_view_raw(json_data: &'static str, size: f64) -> impl Piece {
    #[cfg(target_os = "ios")]
    {
        use day_uikit::UiKitExt;
        let html = build_lottie_html(json_data);
        spacer()
            .frame(size, size)
            .uikit(move |view, _class, _mtm| {
                mount_lottie_webview(view, &html, size);
            })
    }
    #[cfg(not(target_os = "ios"))]
    {
        spacer().frame(size, size)
    }
}

#[cfg(target_os = "ios")]
fn mount_lottie_webview(parent_view: &objc2_ui_kit::UIView, html: &str, size: f64) {
    use objc2::runtime::{AnyClass, AnyObject};
    use objc2_core_foundation::{CGPoint, CGRect, CGSize};
    use objc2_foundation::NSString;

    unsafe {
        unsafe extern "C" {
            fn dlopen(path: *const std::ffi::c_char, mode: std::ffi::c_int) -> *mut std::ffi::c_void;
        }
        dlopen(c"/System/Library/Frameworks/WebKit.framework/WebKit".as_ptr(), 1);

        let Some(wk_class) = AnyClass::get(c"WKWebView") else {
            crate::shared::nslog::nslog("[Lottie] ERROR: WKWebView class not found");
            return;
        };

        let frame = CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: CGSize {
                width: size,
                height: size,
            },
        };

        let web_view: *mut AnyObject = objc2::msg_send![wk_class, alloc];
        let web_view: *mut AnyObject = objc2::msg_send![web_view, initWithFrame: frame];
        if web_view.is_null() {
            crate::shared::nslog::nslog("[Lottie] ERROR: Failed to instantiate WKWebView");
            return;
        }

        let _: () = objc2::msg_send![web_view, setOpaque: false];
        let clear_color = objc2_ui_kit::UIColor::clearColor();
        let _: () = objc2::msg_send![web_view, setBackgroundColor: &*clear_color];
        let _: () = objc2::msg_send![web_view, setUserInteractionEnabled: false];
        let _: () = objc2::msg_send![web_view, setAutoresizingMask: 18usize];

        let scroll_view: *mut AnyObject = objc2::msg_send![web_view, scrollView];
        if !scroll_view.is_null() {
            let _: () = objc2::msg_send![scroll_view, setScrollEnabled: false];
            let _: () = objc2::msg_send![scroll_view, setBounces: false];
            let _: () = objc2::msg_send![scroll_view, setBackgroundColor: &*clear_color];
        }

        let ns_html = NSString::from_str(html);
        let _: () = objc2::msg_send![
            web_view,
            loadHTMLString: &*ns_html,
            baseURL: std::ptr::null::<AnyObject>()
        ];

        let _: () = objc2::msg_send![parent_view, addSubview: web_view];
        crate::shared::nslog::nslog(&format!("[Lottie] Successfully mounted Lottie animation view ({size}x{size})"));
    }
}
