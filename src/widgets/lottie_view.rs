use day::prelude::*;
use std::sync::LazyLock;

#[allow(dead_code)]
static DUCK_INTRO_JSON: &str = include_str!("../../resource/animations/duck_intro.json");
#[allow(dead_code)]
static DUCK_LOGIN_JSON: &str = include_str!("../../resource/animations/duck_login.json");
#[allow(dead_code)]
static DUCK_PASSWORD_JSON: &str = include_str!("../../resource/animations/duck_password.json");
#[allow(dead_code)]
static LOTTIE_JS: &str = include_str!("../../resource/animations/lottie_light.min.js");

static DUCK_INTRO_HTML: LazyLock<String> = LazyLock::new(|| build_lottie_html(DUCK_INTRO_JSON, Some("-45 115 475 295")));
static DUCK_LOGIN_HTML: LazyLock<String> = LazyLock::new(|| build_lottie_html(DUCK_LOGIN_JSON, Some("55 95 400 320")));
static DUCK_PASSWORD_HTML: LazyLock<String> = LazyLock::new(|| build_lottie_html(DUCK_PASSWORD_JSON, Some("70 45 360 425")));

#[allow(dead_code)]
fn build_lottie_html(json_data: &str, initial_viewbox: Option<&str>) -> String {
    let mut html = String::with_capacity(LOTTIE_JS.len() + json_data.len() + 1024);
    html.push_str(r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  html, body {
    width: 100%; height: 100%;
    margin: 0; padding: 0;
    overflow: hidden;
    background: transparent !important;
    background-color: transparent !important;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  #anim {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  svg {
    display: block !important;
    width: 100% !important;
    height: 100% !important;
    max-width: 100% !important;
    max-height: 100% !important;
    margin: auto;
  }
</style>
</head>
<body>
<div id="anim"></div>
<script>
"#);
    html.push_str(LOTTIE_JS);
    html.push_str(r#"
</script>
<script>
try {
  var animData = "#);
    html.push_str(json_data);
    html.push_str(r#";
  var vb = "#);
    if let Some(vb) = initial_viewbox {
        html.push('"');
        html.push_str(vb);
        html.push('"');
    } else {
        html.push_str("null");
    }
    html.push_str(r#";
  var anim = lottie.loadAnimation({
    container: document.getElementById('anim'),
    renderer: 'svg',
    loop: true,
    autoplay: true,
    animationData: animData,
    rendererSettings: {
      preserveAspectRatio: 'xMidYMid meet',
      viewBoxSize: vb,
      viewBoxOnly: true,
      progressiveLoad: false,
      hideOnTransparent: false
    }
  });
  window.lottieAnim = anim;
  var fixSvg = function() {
    var svg = document.querySelector('svg');
    if (!svg) return;
    svg.removeAttribute('width');
    svg.removeAttribute('height');
    svg.style.width = '100%';
    svg.style.height = '100%';
    svg.style.maxWidth = '100%';
    svg.style.maxHeight = '100%';
    svg.setAttribute('preserveAspectRatio', 'xMidYMid meet');

    try {
      var g = svg.querySelector('g');
      if (g && typeof g.getBBox === 'function') {
        var bbox = g.getBBox();
        if (bbox && bbox.width > 30 && bbox.height > 30 && bbox.width < 1200) {
          var pad = Math.max(bbox.width, bbox.height) * 0.04;
          var vx = (bbox.x - pad).toFixed(1);
          var vy = (bbox.y - pad).toFixed(1);
          var vw = (bbox.width + pad * 2).toFixed(1);
          var vh = (bbox.height + pad * 2).toFixed(1);
          svg.setAttribute('viewBox', vx + ' ' + vy + ' ' + vw + ' ' + vh);
        }
      }
    } catch(err) {}
  };
  anim.addEventListener('DOMLoaded', fixSvg);
  setTimeout(fixSvg, 20);
  setTimeout(fixSvg, 100);
  setTimeout(fixSvg, 500);
  window.addEventListener('resize', function() {
    fixSvg();
    if (window.lottieAnim) window.lottieAnim.resize();
  });
  var keepPlaying = function() {
    if (window.lottieAnim && window.lottieAnim.isPaused) {
      window.lottieAnim.play();
    }
  };
  document.addEventListener('visibilitychange', keepPlaying);
  window.addEventListener('focus', keepPlaying);
  window.addEventListener('pageshow', keepPlaying);
  setInterval(keepPlaying, 250);
} catch(e) {
  console.error("Lottie error:", e);
}
</script>
</body>
</html>"#);
    html
}

pub fn duck_intro_animation(size: f64) -> impl Piece {
    lottie_view_cached(&DUCK_INTRO_HTML, size)
}

pub fn duck_login_animation(size: f64) -> impl Piece {
    lottie_view_cached(&DUCK_LOGIN_HTML, size)
}

pub fn duck_password_animation(size: f64) -> impl Piece {
    lottie_view_cached(&DUCK_PASSWORD_HTML, size)
}

pub fn login_animation_view() -> impl Piece {
    duck_intro_animation(130.0)
}

pub fn login_animation_view_sized(size: f64) -> impl Piece {
    duck_intro_animation(size)
}

pub fn lottie_view_cached(html: &'static str, size: f64) -> impl Piece {
    #[cfg(target_os = "ios")]
    {
        use day_uikit::UiKitExt;
        column(())
            .uikit(move |view, _class, _mtm| {
                mount_lottie_webview(view, html, size);
            })
            .frame(size, size)
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = html;
        column(()).frame(size, size)
    }
}

pub fn lottie_view_raw(json_data: &'static str, size: f64) -> impl Piece {
    #[cfg(target_os = "ios")]
    {
        use day_uikit::UiKitExt;
        let html = build_lottie_html(json_data, None);
        column(())
            .uikit(move |view, _class, _mtm| {
                mount_lottie_webview(view, &html, size);
            })
            .frame(size, size)
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = json_data;
        column(()).frame(size, size)
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

        // Shared WKProcessPool so WebKit process is kept warm across screens
        static POOL_PTR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let pool = {
            let mut ptr = POOL_PTR.load(std::sync::atomic::Ordering::Relaxed);
            if ptr == 0 {
                if let Some(pool_cls) = AnyClass::get(c"WKProcessPool") {
                    let p: *mut AnyObject = objc2::msg_send![pool_cls, alloc];
                    let p: *mut AnyObject = objc2::msg_send![p, init];
                    POOL_PTR.store(p as usize, std::sync::atomic::Ordering::Relaxed);
                    ptr = p as usize;
                }
            }
            ptr as *mut AnyObject
        };

        let web_view: *mut AnyObject = if let Some(cfg_cls) = AnyClass::get(c"WKWebViewConfiguration") {
            let cfg: *mut AnyObject = objc2::msg_send![cfg_cls, alloc];
            let cfg: *mut AnyObject = objc2::msg_send![cfg, init];
            if !pool.is_null() {
                let _: () = objc2::msg_send![cfg, setProcessPool: pool];
            }
            let wv: *mut AnyObject = objc2::msg_send![wk_class, alloc];
            objc2::msg_send![wv, initWithFrame: frame, configuration: cfg]
        } else {
            let wv: *mut AnyObject = objc2::msg_send![wk_class, alloc];
            objc2::msg_send![wv, initWithFrame: frame]
        };

        if web_view.is_null() {
            crate::shared::nslog::nslog("[Lottie] ERROR: Failed to instantiate WKWebView");
            return;
        }

        let _: () = objc2::msg_send![web_view, setOpaque: false];
        let clear_color = objc2_ui_kit::UIColor::clearColor();
        let _: () = objc2::msg_send![web_view, setBackgroundColor: &*clear_color];
        let _: () = objc2::msg_send![web_view, setUserInteractionEnabled: false];

        let scroll_view: *mut AnyObject = objc2::msg_send![web_view, scrollView];
        if !scroll_view.is_null() {
            let _: () = objc2::msg_send![scroll_view, setScrollEnabled: false];
            let _: () = objc2::msg_send![scroll_view, setBounces: false];
            let _: () = objc2::msg_send![scroll_view, setBackgroundColor: &*clear_color];
            // Disable automatic safe-area insetting inside the WKWebView:
            // Ensures the SVG stays 100% centered and scales to bounds without being pushed down!
            let _: () = objc2::msg_send![scroll_view, setContentInsetAdjustmentBehavior: 2usize]; // UIScrollViewContentInsetAdjustmentNever
        }

        let ns_html = NSString::from_str(html);
        let bundle_class = AnyClass::get(c"NSBundle");
        let base_url: *mut AnyObject = if let Some(b_cls) = bundle_class {
            let main_bundle: *mut AnyObject = objc2::msg_send![b_cls, mainBundle];
            if !main_bundle.is_null() {
                objc2::msg_send![main_bundle, bundleURL]
            } else {
                std::ptr::null_mut()
            }
        } else {
            std::ptr::null_mut()
        };

        let _: () = objc2::msg_send![
            web_view,
            loadHTMLString: &*ns_html,
            baseURL: base_url
        ];

        let _: () = objc2::msg_send![parent_view, addSubview: web_view];
        let _: () = objc2::msg_send![web_view, setTranslatesAutoresizingMaskIntoConstraints: false];

        // Pin all 4 edges with Auto Layout so WKWebView matches parent container frame perfectly
        let top: *mut AnyObject = objc2::msg_send![web_view, topAnchor];
        let p_top: *mut AnyObject = objc2::msg_send![parent_view, topAnchor];
        let c_top: *mut AnyObject = objc2::msg_send![top, constraintEqualToAnchor: p_top];
        let _: () = objc2::msg_send![c_top, setActive: true];

        let bottom: *mut AnyObject = objc2::msg_send![web_view, bottomAnchor];
        let p_bottom: *mut AnyObject = objc2::msg_send![parent_view, bottomAnchor];
        let c_bottom: *mut AnyObject = objc2::msg_send![bottom, constraintEqualToAnchor: p_bottom];
        let _: () = objc2::msg_send![c_bottom, setActive: true];

        let leading: *mut AnyObject = objc2::msg_send![web_view, leadingAnchor];
        let p_leading: *mut AnyObject = objc2::msg_send![parent_view, leadingAnchor];
        let c_leading: *mut AnyObject = objc2::msg_send![leading, constraintEqualToAnchor: p_leading];
        let _: () = objc2::msg_send![c_leading, setActive: true];

        let trailing: *mut AnyObject = objc2::msg_send![web_view, trailingAnchor];
        let p_trailing: *mut AnyObject = objc2::msg_send![parent_view, trailingAnchor];
        let c_trailing: *mut AnyObject = objc2::msg_send![trailing, constraintEqualToAnchor: p_trailing];
        let _: () = objc2::msg_send![c_trailing, setActive: true];

        let _: () = objc2::msg_send![parent_view, setBackgroundColor: &*clear_color];
        let _: () = objc2::msg_send![parent_view, setUserInteractionEnabled: false];
        let _: () = objc2::msg_send![parent_view, setClipsToBounds: true];
        crate::shared::nslog::nslog(&format!("[Lottie] Successfully mounted Lottie animation view ({size}x{size})"));
    }
}
