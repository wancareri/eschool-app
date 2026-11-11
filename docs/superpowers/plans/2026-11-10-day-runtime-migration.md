# Day 0.7.1 Runtime Migration Implementation Plan

> **For agentic workers:** Use superpowers:subagent-driven-development to execute and review the tasks below.

**Goal:** Move the application from its Day 0.4.3 fork to the current Day 0.7.1 framework and compatible native Lottie 0.5.2, then deliver a verified iOS IPA.

**Architecture:** Use one Day dependency source throughout the app, pieces, build tools and tests. Prefer released 0.7.1 crates; alias `day-app` as `day` and `day-toolkit-uikit` as `day-uikit` where that preserves app code. Replace retired optional pieces with their built-in equivalents. Keep the two-step login and the user's explicit password-capture exception; native credential roles must come from the new InputPurpose API.

**Tech Stack:** Rust 2024, Day 0.7.1, objc2 0.6, native Lottie 0.5.2, iOS UIKit, Day CLI 0.7.1.

**Spec:** User request: “так обнови фремйворк”; AGENTS.md navigation, keyboard, credential privacy and delivery constraints.

## Global Constraints

- Preserve existing application features, account isolation, logout resets, RU/BE/EN, notices and native Lottie.
- Keep one window toolbar and native interactive navigation. Never add `.on_back` guards or forced keyboard reloads on input-mode changes.
- Native hooks must target actual fields before layout-only wrappers.
- Do not claim the framework migration fixes the device Passwords symptom without device evidence.
- Keep the current authorized workspace and its uncommitted AGENTS.md research notes. No unrelated cleanup or deletion of build artifacts.

### Task 1: Dependency and application API migration

**Files:** Cargo.toml, Cargo.lock, build.rs as needed, src except pages/login.rs and shared/autofill.rs, existing tests.

- [ ] Verify crates.io versions (`day-app=0.7.1`, framework pieces/parts 0.7.1, native Lottie 0.5.2).
- [ ] Replace the old git graph and old Lottie patch table with one compatible dependency graph. Update backend feature names and replace built-in datetime/pullrefresh usages.
- [ ] Run `cargo check --offline` after dependency fetching; adapt APIs reported by the compiler without replacing functional screens with placeholders.
- [ ] Run `cargo check --target aarch64-apple-ios --no-default-features --features uikit`; adapt native integration to the renamed backend.
- [ ] Run existing workspace tests and adapt probes only where the new framework changes their public API or documented metrics.

### Task 2: Authentication API migration

**Files:** src/pages/login.rs, src/shared/autofill.rs.

- [ ] Use `.input_purpose(InputPurpose::Username/Password/NewPassword)` and `.submit_label(SubmitLabel::Next/Go/Done)` on the corresponding leaves before native hooks.
- [ ] Use the framework's same-field secure masking API while retaining captured values, focus guards and return behavior as needed.
- [ ] Remove only redundant native trait/masking work now owned by the framework. Preserve diagnostics without credential text or lengths.
- [ ] Update the credential-node regression for the new framework types and verify both mock and UIKit compilation.
- [ ] Preserve session checks, cancellation, completed-pop handling and no fixed focus timers.

### Task 3: Native compatibility audit and review

**Files:** old fork and new framework source (read-only audit); app native integrations if a confirmed compatibility gap exists.

- [ ] Compare fork-dependent cover OverFullScreen, keyboard root resizing, transition observers and toolbar lowering with Day 0.7.1.
- [ ] Report exact remaining compatibility gaps before deciding whether app configuration or a small maintained framework patch is required.
- [ ] Review integrated changes for lost features, duplicate framework instances and credential/privacy regressions.

### Task 4: Validation and delivery

**Files:** Day.toml, AGENTS.md, CI files if required.

- [ ] Run desktop check, UIKit check, `cargo test --workspace`, `day prepare -t ios-uikit --check`, and `git diff --check` on the final tree.
- [ ] Bump build from 73 to 74, document final dependency versions and changed traps.
- [ ] Commit reviewed changes, push to authorized `wancareri main`, monitor Actions and fix actual build failures.
- [ ] Download ios-ipa, check archive CRC, bundle id/build, executable and all three native Lottie assets. Copy IPA to Downloads and provide the GitHub artifact link.
