# Simple feature-folder migration

**Goal:** Group the entire project by app function with one clear responsibility per file. Remove the parallel `pages/features/widgets/shared` layer pattern and keep unsupported UIKit behavior isolated.

**Layout:** `app/`, `routes/`, `pages/<screen>/`, shared `components/`, cross-cutting `app/services/`, and `ios/`.

## Completed slices

- [x] Split login screen by responsibility, move its large regression suite to `login/page/auth_error_tests.rs`, and keep its route API stable.
- [x] Split diary page into period, pager, gesture, and summary modules.
- [x] Move diary data and auth orchestration beside their owning feature.
- [x] Move shared UI into `components/`, feature-only UI beside its feature, and window wiring into `components/`.
- [x] Move cross-cutting services into `app/services/` and UIKit services into `ios/`.
- [x] Group route screens under `pages/<screen>/` and move route definitions to `routes/`.
- [x] Move window error binding into `components/error_handler.rs`.
- [x] Move note persistence to `diary/data/notes.rs`.

## Remaining work

- [ ] Split remaining large files by responsibility, especially attachments, PIN, lesson/grade details, and diary loading/marks.
- [ ] Replace direct feature-to-component cleanup calls with app-level state or callbacks.
- [ ] Remove remaining native iOS adapters only after confirming an equivalent Day 0.7.1 API.
- [ ] Add a short source import check for accidental legacy layer paths.
- [ ] Final review and bare/iOS compilation checks.

Do not run tests unless requested. Do not bump the build number, commit, or publish as part of this refactor.
