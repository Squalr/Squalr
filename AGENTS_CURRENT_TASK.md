# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- Assume any unstaged file changes are from a previous iteration, and can be kept if they look good
- The android device is rooted.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Audit GUI project against TUI for functionality gaps now that Android full process-list refresh no longer reproduces as empty.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Android build/runtime ownership is in `squalr` crate (`android_main` in `squalr/src/lib.rs`, manifest/resources under `squalr/android/`); `squalr-android` crate removed.
- Canonical Android entry scripts: `build_and_deploy.py`, `run_apk.py`, `debug_run_privileged_shell.py`.
- Launcher identity is pinned as `com.squalr.android/android.app.NativeActivity` with `android.app.lib_name = "squalr"`; deploy script enforces this.
- Android worker target path is standardized as `/data/local/tmp/squalr-cli`.
- Device status (2026-02-22): rooted `Pixel_9_Pro_Fold` (`adb serial: 4C101FDKD000Z8`, fingerprint `google/comet/comet:16/BP3A.251005.004.B3/14332485:user/release-keys`).
- Root-only shell validation succeeds: `adb shell su -c id` returns `uid=0(root)`.
- Manual privileged worker launch validation succeeds: `debug_run_privileged_shell.py` launches `squalr-cli --ipc-mode` and `pidof squalr-cli` reports a running pid.
- Root cause for worker exits was `Cli::stay_alive()` reading stdin in `su -c` context (EOF); fix keeps IPC worker alive via sleep loop.
- Interprocess spawn diagnostics now log worker command, su candidate resolution, and invocation-specific failures (`su -c`, `su 0 sh -c`, `su root sh -c`).
- Privileged worker IPC diagnostics now log explicit receive/connection/read-lock failures before listener shutdown.
- Process selector includes dispatch-failure + stale-timeout guards, and now logs process-list request dispatch + response counts.
- GUI now triggers an initial full process-list refresh on process-selector view construction for deterministic Android validation.
- Host validation (2026-02-22): `cargo test -p squalr --lib -- --nocapture` passed (28 passed, 0 failed).
- Host validation (2026-02-22): `cargo test -p squalr-cli -- --nocapture` passed (2 passed, 0 failed).
- Android validation (2026-02-22, 08:45 local): `build_and_deploy.py --debug --launch-log-seconds 30` passed; worker detected (`pidof squalr-cli` => `16655`); artifact `logs/android_bootstrap_20260222_084500.log`.
- Android validation (2026-02-22, 08:54 local): `build_and_deploy.py --debug --launch-log-seconds 45` logged GUI dispatch + successful full process-list response (`8111` entries), with `SqualrCli` execution log for `require_windowed=false`; worker detected (`pidof squalr-cli` => `17904`); artifact `logs/android_process_selector_validation_20260222_085458.log`.
- Android validation (2026-02-22, 09:00 local): `build_and_deploy.py --debug --launch-log-seconds 45` passed; GUI logged full process-list dispatch + response (`8114` entries), repeated windowed responses (`0` entries), and worker detected (`pidof squalr-cli` => `18841`).
- No occurrences in current Android validation logs of prior failure signatures: `failed to fill whole buffer`, `Broken pipe (os error 32)`, or IPC listener read-lock/receive failure logs.
- GUI process-selector refresh root cause (2026-02-22): `ProcessSelectorView` held a read lock on `ProcessSelectorViewData` while drawing the toolbar; toolbar refresh click requires a write lock, so refresh attempts could no-op.
- Fix (2026-02-22): render `ProcessSelectorToolbarView` before acquiring the read lock in `squalr/src/views/process_selector/process_selector_view.rs`, allowing refresh dispatch to acquire write access.
- Host validation (2026-02-22): `cargo test -p squalr --lib -- --nocapture` passed (28 passed, 0 failed) after lock-order change.
