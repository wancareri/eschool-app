# AGENTS.md — Developer & AI Agent Guide for `eschool-app`

> **Note for AI agents:** This document is the single source of truth for the project's architecture, conventions, workflows, and solved traps. Read this file completely before making any changes.

---

## 1. Project Overview & Tech Stack

- **Application:** `eschool-app` (ESchool / Электронный дневник) — cross-platform school diary app.
- **Framework:** [Day](https://daybrite.dev) (`wancareri/day` git fork) — Rust reactive UI framework with native platform widgets.
- **Primary Target:** `ios-uikit` (`aarch64-apple-ios`).
- **Language / Tooling:**
  - Rust 2024 edition
  - Objective-C 2.0 bindings: `objc2`, `objc2_ui_kit`, `objc2_foundation`, `block2`
  - SVG Vector generation: `day-build` (`resource/vectors/*.svg` -> `crate::res::vectors::*`)
  - Manifest: `Day.toml` (contains app metadata, window size, and incrementing build number `build = N`)

---

## 2. Git Remotes & CI / CD Build Pipeline

The project uses a dual-remote Git setup:
- `origin`: `https://github.com/kayzer4/eschool-app.git` (often 403 on push).
- `wancareri`: `git@github.com-wancareri:wancareri/eschool-app.git` (configured SSH key, triggers GitHub Actions CI on push to `main`!).

### Building & Releasing Workflow:
1. **Always verify compilation locally before pushing:**
   ```bash
   # Desktop / bare check
   cargo check --manifest-path /home/kayzer/PetProjects/eschool-app/Cargo.toml

   # iOS UIKit target check (fast, ~0.5s, target aarch64-apple-ios is installed locally):
   cargo check --manifest-path /home/kayzer/PetProjects/eschool-app/Cargo.toml --target aarch64-apple-ios --no-default-features --features uikit
   ```
2. **Bump build number:**
   Edit `Day.toml`: increment `build = <N>`.
3. **Commit & Push to `wancareri main`:**
   ```bash
   git -C /home/kayzer/PetProjects/eschool-app add -A
   git -C /home/kayzer/PetProjects/eschool-app commit -m "..."
   git -C /home/kayzer/PetProjects/eschool-app push wancareri main
   ```
4. **Monitor GitHub Actions CI:**
   Due to proxy/token quirks in the environment, ALWAYS query GitHub CLI with clean environment and token extraction:
   ```bash
   cd /home/kayzer/PetProjects/eschool-app && env -u HTTPS_PROXY -u HTTP_PROXY -u ALL_PROXY -u https_proxy -u http_proxy -u all_proxy GH_TOKEN="$(cat ~/.config/gh/hosts.yml | grep -A 5 'github.com:' | grep 'oauth_token:' | awk '{print $2}')" gh run list -R wancareri/eschool-app -L 1
   ```
5. **Download IPA Artifact:**
   ```bash
   rm -rf /tmp/build && mkdir -p /tmp/build && cd /tmp/build
   env -u HTTPS_PROXY -u HTTP_PROXY -u ALL_PROXY -u https_proxy -u http_proxy -u all_proxy GH_TOKEN="$(cat ~/.config/gh/hosts.yml | grep -A 5 'github.com:' | grep 'oauth_token:' | awk '{print $2}')" gh run download <RUN_ID> -R wancareri/eschool-app -D /tmp/build
   cp /tmp/build/ios-ipa/*.ipa /home/kayzer/Downloads/eschool-build<N>.ipa
   ```

---

## 3. Architecture & Key Files

```
eschool-app/
├── Day.toml                 # App metadata, window options, build number (e.g. build = 62)
├── Cargo.toml               # Dependencies: day, eschool-api, objc2, etc.
├── build.rs                 # Resource codegen via day_build::generate_resources()
├── resource/
│   ├── vectors/             # SVG icons (lock, eye, sun, moon, globe, tab_login...)
│   └── locales/             # Fluent (.ftl) translation catalogs (ru, be, en)
└── src/
    ├── main.rs / lib.rs     # Entry point: day_start!(options: window(), root)
    ├── app/
    │   ├── state.rs         # Global AppState (auth tokens, profile, signals)
    │   └── window.rs        # Window shell, nav_body (switches between Login and Tab Nav)
    ├── pages/
    │   ├── login.rs         # Intro splash, username step, password step, theme/lang sheets
    │   ├── diary.rs         # Diary lessons, marks, homework
    │   ├── schedule.rs      # Timetable
    │   ├── teachers.rs      # Teachers directory
    │   └── settings.rs      # User settings, preferences
    ├── shared/
    │   ├── ios_nav.rs       # Native UINavigationBar items, trailingItemGroups attachment
    │   ├── keyboard.rs      # Keyboard height tracking, 120fps hardware follower, UITextField swizzles
    │   ├── colors.rs        # Dynamic hex/accent color management
    │   ├── haptics.rs       # iOS UIImpactFeedbackGenerator / UINotificationFeedbackGenerator
    │   └── secure.rs        # iOS Keychain token persistence
    └── widgets/
        └── lottie_view.rs   # Duck stickers (duck_intro_animation, duck_login_animation, duck_password_animation)
```

---

## 4. Critical Rules & Solved Traps

### A. Navigation, Pop Gestures & `.on_back`
- **NEVER use `.on_back(...)` on `nav_stack` unless explicitly vetoing:**
  In `day-uikit`, defining `.on_back` arms Day's guard (`guarded = true`) on `DayNavController`. This causes `interactivePopGestureRecognizer` to intercept the swipe gesture immediately on the first millimeter of touch, firing the callback. If the callback dismisses the keyboard or assumes a completed pop, the user pulling and releasing without popping will experience an ugly keyboard hide/show jump!
- **Handling Pop Natively:**
  Omit `.on_back`. UIKit handles the interactive pop gesture 100% natively. If the user completes the pop, Day updates `path: Signal<Vec<AuthRoute>>`. Observe completed pops using `day::reactive::watch(move || path.get(), ...)`.

### B. Navigation Bar Buttons in iOS 16+ (`trailingItemGroups`)
- **Authentication declares ONE window-level `.toolbar(...)` in `build_nav`:** use a single `toolbar_menu` with `Symbol::More` and `ToolbarPlacement::Primary`. It contains language (globe icon) and appearance submenus. Declaring the same commands on root and destination pages caused duplicated buttons and overflow. Native `UIMenu` anchors to the actual bar item; do not reintroduce delayed manual toolbar injection or action sheets with hard-coded popover coordinates.
- In iOS 16+, `day-uikit` manages `trailingItemGroups` on `UINavigationItem`. When `trailingItemGroups` has any value, UIKit **completely ignores `rightBarButtonItems`**!
- To attach bar buttons natively, wrap the `UIBarButtonItem` slice using `UIBarButtonItemGroup`:
  ```rust
  let fixed_group = UIBarButtonItemGroup::fixedGroupWithRepresentativeItem_items(None, &arr, mtm);
  let groups = NSArray::from_retained_slice(&[fixed_group]);
  nav_item.setTrailingItemGroups(&groups);
  nav_item.setRightBarButtonItems(Some(&arr));
  ```
  > [!CAUTION]
  > The representative item in `fixedGroupWithRepresentativeItem_items` MUST BE `None`. Passing `Some(item)` causes an immediate `NSInternalInconsistencyException` crash!

- **Native UIBarButtonItem vs custom views:**
  Always build native bar items using `UIBarButtonItem::initWithPrimaryAction(...)` with style `Plain`. Custom `UIButton`s wrapped in `initWithCustomView` collapse to 0×0 width in `trailingItemGroups` unless manual AutoLayout constraints are established. Native items behave identically to the Back button.

### C. Keyboard & Duck Sticker Synchronization
- Keep keyboard followers per mounted page using weak view references: the outgoing and incoming pages coexist during an interactive pop. A single global content/sticker view loses the revealed page and retains popped views.
- While the native transition coordinator reports `initiallyInteractive`, defer keyboard end-frame updates until transition completion. On cancellation keep the previous lift; after a committed pop use UIKit's settled keyboard layout guide. Do not animate the form towards a predicted hidden keyboard while the swipe is still cancellable.
- Suppress DayAppDelegate's root-resize callback without processing the frame there again. The dedicated keyboard observer owns the one animation.
- Authentication pages use the global control-aware tap-to-dismiss recognizer. Do not add ancestor `.on_tap(dismiss)` recognizers over input fields and buttons.
- In `keyboard.rs`, only observe `UIKeyboardWillChangeFrameNotification`. Do NOT register redundant observers for `didShow`, `didHide`, or `didChangeFrame` — `didShow` has `duration = 0.0`, which causes an instant jump after the smooth animation ends.
- Apply hardware translations via `CGAffineTransform` within `UIView::animateWithDuration` using the keyboard notification's duration and curve.
- Authentication focus must wait for the destination controller’s actual UIKit transition completion (`shared/autofill.rs::focus_after_navigation`), including the welcome push. Do not use 350 ms timers or focus during an animated push: its duration can exceed that timer, making the keyboard enter diagonally. Preserve an already visible keyboard with `performWithoutAnimation` on the final responder handoff. Keep route/session guards and do not focus after cancellation.

### D. iOS AutoFill & Passwords on Russian / Multilingual Keyboards
- **Do NOT use overlapping invisible companion fields (`companion_pw` / `opacity(0.02)`):**
  Overlapping two text fields at the same location confuses iOS AutoFill and accessibility heuristics. Keep a single interactive `text_field` per step. Explicit user exception in build 66: a separate one-point password capture field below the username card is mounted only while the username step is active. It uses clear text rather than opacity, cannot receive touches or accessibility focus, and is removed before the visible password field is used. Read native values before advancing; do not assume every iOS credential provider will fill the capture field.
- **AutoFill traits are configured AFTER Day realizes a field:** `shared/autofill.rs` configures only authentication fields. Day sets `autocorrectionType: No` after `setSecureTextEntry`, so the former global swizzles were overwritten. Register the explicit username/password/newPassword role; on input-mode changes restore traits for the current field without resigning focus. The password visibility button changes masking on the same UITextField and keeps its `.password` role. Use UIKit’s `UITextContentTypeUsername`, `UITextContentTypePassword`, and `UITextContentTypeNewPassword` constants. The native key button opens the standard edit menu as a fallback when QuickType suggestions collapse.
- **System save prompts:** setting text traits improves AutoFill heuristics; it does not establish ownership of a third-party website. Guaranteed domain credential sharing/saving requires associated domains, matching signing entitlements and the website's AASA configuration. Do not promise that an unsigned/sideloaded build can force Apple Passwords to save credentials for e-schools.by.
- Do not call `reloadInputViews` on input-mode changes: it can interrupt the active Password AutoFill session. Preserve the existing content type when it already matches.
- **Locking Assistant Bar:**
  Set `inputAssistantItem.setAllowsHidingShortcuts: false` so iOS does not collapse the shortcut bar.
- **Language changes:** update localized text and native page titles in place. Keep the authentication stack, focus and field values alive. Do not apply a whole-form `.animation(...)` to the auth form; menu changes must not replay its entrance animation.

---

## 5. Summary of Recent Builds

- **Build 60:** Removed `setTrailingItemGroups(&empty)` and removed `translatesAutoresizingMaskIntoConstraints(false)`.
- **Build 61:** Removed misaligned clear button (`clear_btn`), added vector SVGs (`sun.svg`, `moon.svg`, `globe.svg`), adjusted keyboard notifications.
- **Build 62:**
  - Converted Theme and Language buttons to native `UIBarButtonItem`s with SF Symbols (`sun.max`, `globe`, `RU/BE/EN`) attached directly to `UINavigationBar` (including on `intro_view`).
  - Removed `.on_back(...)` to eliminate the interactive back-swipe keyboard hide/show glitch.
  - Removed overlapping companion password fields and enforced `autocorrectionType = Yes` with automatic refocusing on language change to restore AutoFill Passwords.

- **Build 64:** Native declarative theme/language toolbar on welcome and all login steps; interactive-pop keyboard updates wait for commit/cancel; weak per-page followers; guarded delayed focus and refocus after completed password-to-login pops.

- **Build 65:** One window-level ellipsis menu, native anchored language/appearance submenus, live auth localization without entrance replay, stable password field masking, and scoped AutoFill/input-mode handling.

- **Build 66:** Native submenu icon compatibility for Day’s UIKit menu lowering; RU/BE/EN labels; localized reactive error cards and corrected transport-error classification; opaque navigation background matching the page; no keyboard reload on language changes; user-requested separate password capture input and native value synchronization before advancing. AutoFill eligibility still requires device verification.

- **Build 67:** Central `reset_auth_flow` called by every logout path clears drafts, focus, error and auth navigation; logout also clears settings selection/loading. Successful login removes filled fields before the deferred draft reset to preserve UIKit save heuristics. Password is seeded into its native field before focus, filled-field styling stays stable during AutoFill, and normal username-to-password focus has no 350 ms delay (the welcome push still does). A scoped return-key delegate avoids the default resign/reopen cycle. A native key button opens the system text-edit menu independently of QuickType. The background-tap dismisser ignores keyboard windows, keyboard bounds and presented modals. Language labels use `EN · English` order. System AutoFill/save behavior still requires device verification and does not bypass domain association.

- **Build 68:** Authentication autofocus follows the actual native transition completion instead of timers, with guarded responder handoff after layout. A visible keyboard stays in place during secure-field activation. The password sticker WebView is preloaded while the welcome screen is idle, consumed once at mount, and never reparented from an active page. Sticker WebViews/configurations now release their creation retain correctly. Validate keyboard direction and frame pacing on a physical iPhone.
