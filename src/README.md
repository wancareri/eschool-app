# Source layout

```text
src/
├── app/                    # startup, window, and global state
│   ├── data/                # cache, session checks, and credentials
│   ├── preferences/         # colors, haptics, and language
│   ├── format.rs            # school dates and grades
│   ├── i18n/                # translations
│   ├── log_bridge.rs        # app logging setup
│   ├── state.rs             # shared app state
│   └── window.rs            # app entry and screen selection
├── pages/                  # screens grouped by app function
│   ├── diary/              # screen, diary UI, data, and behavior
│   ├── login/              # screen, auth flow, and illustration
│   ├── pin/
│   ├── schedule/
│   ├── settings/
│   └── teachers/
├── components/             # UI pieces imported by screens
└── ios/                    # UIKit-specific compatibility code
```

`app/window.rs` connects navigation to pages. Each page folder holds its screen and behavior. Screen-specific files live beside the page; UI shared by multiple pages lives in the top-level `components/`. iOS-only UIKit code goes in `ios/`; app code should use Day APIs unless an audited Day 0.7.1 gap requires an adapter.

Keep one clear responsibility per file. Preserve keyboard/focus, AutoFill, modal, navigation, accessibility, localization, session, offline-cache, and gesture behavior.

## Remaining cleanup

Large files still need responsibility-based splits: diary data/marks, attachments, PIN, settings, and lesson/grade details. Some UIKit hooks remain in page code and need to move into `ios/` during the corresponding slice.
# Source map

`app/` owns startup, shared state, preferences, credentials, and cached data. `pages/` owns a screen and its screen-specific UI/logic. `components/` contains UI reused by multiple pages. `ios/` contains UIKit adapters only.

## Platform rules

- Use Day parts for permissions, notifications, biometrics, haptics, and clipboard; check capability before exposing optional actions.
- Keep secrets out of `day::prefs`. `app/data/credentials.rs` uses the same `day-part-keychain` API on every platform; Day chooses Apple Keychain, Android Keystore, HarmonyOS HUKS, Windows DPAPI, or Linux Secret Service internally. Web reports `Unsupported`, and the app does not fall back to plaintext preferences or process memory.
- UIKit-specific keyboard, navigation, AutoFill, and lifecycle code belongs in `ios/`; other pages should call a shared capability API where one exists.
- Lottie runs natively on iOS/Android and through Day's webview-backed player on desktop/ArkUI/DOM toolkits.
- Browser login is not supported yet: the current API path is blocking and its OAuth redirect flow needs browser-compatible async transport and service CORS support.
