# AGENTS.md — Developer & AI Agent Guide for `eschool-app`

> **Note for AI agents:** This document is the single source of truth for the project's architecture, conventions, workflows, and solved traps. Read this file completely before making any changes.

---

## 1. Project Overview & Tech Stack

- **Application:** `eschool-app` (ESchool / Электронный дневник) — cross-platform school diary app.
- **Framework:** [Day](https://daybrite.dev) (`wancareri/day` git fork) — Rust reactive UI framework with native platform widgets.
- **Primary Target:** `ios-uikit` (`aarch64-apple-ios`).
- **Stickers:** `day-piece-lottie` (Day 0.4-compatible revision), native LottieAnimationView on iOS and Android. JSON files live under `resource/assets/animations/` so Day bundles them.
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
- **AutoFill traits are configured AFTER Day realizes a field:** `shared/autofill.rs` configures only authentication fields. Register the explicit username/password/newPassword role; on input-mode changes restore traits for the current field without resigning focus. Keep `autocorrectionType = No`, `spellCheckingType = No`, and (where available, guarded by selector availability) `inlinePredictionType = No`. Password AutoFill is enabled by the credential content type, not by dictionary predictions. Do not force `autocorrectionType = Yes`: that let language changes replace the Passwords entry with word suggestions. Avoid reassigning unchanged traits during input-mode changes. The password visibility button changes masking on the same UITextField and keeps its `.password` role. Use UIKit’s `UITextContentTypeUsername`, `UITextContentTypePassword`, and `UITextContentTypeNewPassword` constants. The native key button opens the standard edit menu as a fallback when QuickType suggestions collapse.
- **System save prompts:** setting text traits improves AutoFill heuristics; it does not establish ownership of a third-party website. Guaranteed domain credential sharing/saving requires associated domains, matching signing entitlements and the website's AASA configuration. Do not promise that an unsigned/sideloaded build can force Apple Passwords to save credentials for e-schools.by.
- Do not call `reloadInputViews` on input-mode changes: it can interrupt the active Password AutoFill session. Preserve the existing content type when it already matches.
- **Locking Assistant Bar:**
  `inputAssistantItem.setAllowsHidingShortcuts: false` concerns the iPad shortcuts bar. Apple documents that iPhone ignores assistant-item contents; this is not a supported way to pin iPhone Password AutoFill suggestions. It also does not affect built-in auto-hiding logic. Do not treat it as a confirmed Passwords fix.
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

### Native Lottie migration (build 69)
- `src/widgets/lottie_view.rs` wraps `day_piece_lottie::lottie` for the three authentication stickers. Do not restore the app-owned WebView/HTML/JavaScript player or WebView prewarming.
- Native JSON assets are in `resource/assets/animations/`. Pass `animations/duck_intro`, `animations/duck_login`, or `animations/duck_password` to the player; it appends `.json` and resolves the bundled asset.
- The component is pinned to its Day 0.4-compatible revision. Its canonical Day dependencies are redirected to the existing `wancareri/day` source through `[patch]` so the app and the component use one framework instance.
- Native iOS SwiftPM Lottie dependencies/shims are contributed by the piece’s Cargo metadata; no duplicate app-specific Swift bridge is needed.

### Keyboard word-prediction correction (build 69)
- Removed the app’s forced `autocorrectionType = Yes` from authentication field setup and input-mode restoration. All credential roles now disable autocorrection/spell checking and, on supported iOS versions, inline predictions while keeping their explicit credential content type.
- This addresses the reported switch from Passwords to word corrections on iOS 27. Actual Passwords visibility still needs on-device verification.

### Shared page chrome and in-app notices (build 69)
- Use `widgets::header::page(title, path, content)` for page navigation and native titles. Main page headings belong in the navbar; do not repeat them in scroll content.
- `build_nav` owns the one window-level toolbar via `widgets::header::toolbar`. It switches between the login options menu and the authenticated connection-status bar item. Do not inject status buttons from individual pages or timers.
- `widgets::conn_status::toolbar` creates a native toolbar button; UIKit provides its navigation-bar glass treatment. It opens the existing `bottom_sheet::network_error_sheet`, built on `modal_window`.
- `widgets::notice` is the shared message UI using the same modal shell. `bind_error` consumes an error source once and presents it, so repeating the same failed action can show the error again. `presenter(state)` exposes Send-safe setters for async success callbacks.
- Global app errors, login validation, password creation/change and biometric setup notices use this shell. Network-status text and shared actions are localized in RU/BE/EN. PIN keypad validation remains part of the PIN form.
- Logout closes notice/network routes and clears their payload. The user subsequently requested publishing these accumulated changes to GitHub as build 69.

### Unresolved Passwords strip after keyboard-language changes
- User confirmed build 69 still loses Passwords on iOS 27: the strip remains visible but empty. Disabling autocorrection did not resolve credential-suggestion retention. Do not claim otherwise.
- Current diagnostic patch avoids reassigning `allowsHidingShortcuts` when already false, and records `[AutoFill]` events before/after input-mode reconciliation. This is an unverified hypothesis, not a confirmed fix.
- Settings → Dev → Input diagnostics exports only dedicated trait snapshots, including in release builds. Diagnostics must never read credential text, selection or lengths, and must exclude general authentication/API logs.
- Need actual device reproduction/logs to determine whether fields lose credential traits or UIKit loses its candidate session. USB device access is currently unavailable; the user cannot connect the iPhone now.

### AutoFill documentation investigation after build 70
- Current Day documentation exposes `.input_purpose(InputPurpose::Username/Password)`. The app’s pinned Day 0.4.3 revision does not expose `InputPurpose`; its UIKit renderer sets initial content type from `secure`, while app-owned `shared/autofill.rs` restores explicit UIKit credential constants after realization. Do not paste APIs from current docs into this older fork without checking source compatibility.
- Apple explicitly supports separate-page username/password AutoFill with corresponding content types. Multi-step login alone does not establish the cause of the intermittent empty strip.
- The login ellipsis menu now includes Copy input diagnostics so it can be exported before authentication, with no modal, form reset or forced responder handoff. Empty logs copy the localized explanation instead of leaving stale clipboard text. Included in the build 71 migration.
- References: https://daybrite.dev/docs/reference/textfield/ ; https://daybrite.dev/docs/reference/cli/ ; https://developer.apple.com/documentation/security/enabling-password-autofill-on-a-text-input-view ; https://developer.apple.com/documentation/uikit/uitextinputassistantitem

### Day CLI 0.7.1 migration (build 71)
- Latest stable `day-cli` verified against crates.io: 0.7.1. Both CI workflows pin it with `--locked`; iOS binary cache keys include that version. Commands now use `-t/--target`, since `-p/--platform` was removed.
- The application framework remains the pinned Day 0.4.3 fork. CLI installation does not change the runtime or introduce `InputPurpose`; do not describe this migration as a Passwords fix.
- CLI 0.7.1 requires Xcode 27 JSON5 `project.xcproj` files. Native iOS/macOS projects use the new official host templates; iOS retains `nslog_bridge.m` in compile sources. The old `project.pbxproj` is legacy, not the CLI's canonical configuration.
- iOS deployment target is 16.0. `[permissions] biometrics` in Day.toml owns the Face ID usage description; the CLI regenerates it in Info.plist. Do not hand-edit only the generated key.
- `android-material` is the canonical CLI target. The app's `material` feature aliases the existing `mdc` feature for compatibility with its older runtime and Lottie piece.
- Build 71 includes the previously prepared login-menu diagnostic-copy action.
