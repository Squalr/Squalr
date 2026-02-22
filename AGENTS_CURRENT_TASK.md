# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- Assume any unstaged file changes are from a previous iteration, and can be kept if they look good
- The android device is rooted.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Continue pruning `pr/android-fixes` diff vs `main` by removing remaining non-Android churn where not required (workspace config churn / lockfile noise).
- Run full rooted-device deploy via `python ./build_and_deploy.py` (non-compile-check path) and validate windowed filtering behavior on device.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Fixed Android windowed filtering false positives by requiring a process to be the primary package process (`cmdline == package`) before it can be considered windowed.
- This excludes colon-suffixed auxiliary/service processes (for example `com.app:worker`) from windowed-only results.
- Added Android unit tests for primary-package process classification in `android_process_query.rs`.
- Reduced Android logging noise by removing startup breadcrumb logs in `squalr/src/lib.rs` and non-essential info logs in `android_process_query.rs`.
- Validation run: `cargo fmt --all`, `cargo test -p squalr-tests --locked`, `cargo check -p squalr-engine-operating-system --target aarch64-linux-android --locked`.
- Android compile-check deploy path succeeded: `python ./build_and_deploy.py --compile-check`.
- Note: direct `cargo check -p squalr --target aarch64-linux-android --locked` can fail in this environment due missing `aarch64-linux-android-clang` pathing for `ring`; use `cargo ndk`/deploy script path for Android validation.
