# Simple routes-and-pages architecture

## Goal

Make the project easy to navigate with familiar folders for routes, screens, shared UI, and iOS support. Keep each screen's code together and avoid heavyweight architecture layers.

## Layout

```text
src/
├── app/             # startup, window, state, global services/components
├── routes/          # navigation types and route definitions
├── pages/           # feature folders; each owns its screen and data/flow
├── components/      # UI reused across pages
└── ios/    # narrow UIKit compatibility
```

## Rules

1. `routes/` owns navigation types and route definitions; it does not render screens.
2. `pages/<screen>/` owns the screen and its related data, flow, and screen-only UI.
3. `components/` contains UI used by multiple screens. Window-level wiring belongs in `components/`.
4. Cross-cutting services belong in `app/services/`; UIKit compatibility belongs in `ios/`.
5. Prefer Day 0.7.1 public APIs when they preserve behavior. Keep unsupported behavior isolated and documented; do not silently delete it.
6. Preserve localization, navigation, AutoFill, keyboard/focus behavior, accessibility, session safety, offline cache semantics, and gestures.
7. Do not add parallel `features/`, `widgets/`, or `shared/` layers.

## Migration

Move files into this layout without changing user-visible behavior, then split large modules by responsibility. Compile and review each slice before continuing. No runtime upgrade, redesign, build-number change, or publishing is included.
