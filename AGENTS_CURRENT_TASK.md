# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- Assume any unstaged file changes are from a previous iteration, and can be kept if they look good
- The android device is rooted.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Investigate Android privileged worker lifecycle: app-side spawn succeeds, but `squalr-cli --ipc-mode` exits immediately and IPC channel breaks (`Broken pipe`).
- Validate end-to-end process list population once worker lifecycle is stable.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Android build/runtime ownership is in `squalr` crate (`android_main` in `squalr/src/lib.rs`, manifest/resources under `squalr/android/`); `squalr-android` crate removed.
- Canonical Android entry scripts: `build_and_deploy.py`, `run_apk.py`, `debug_run_privileged_shell.py`.
- Launcher identity is pinned as `com.squalr.android/android.app.NativeActivity` with `android.app.lib_name = "squalr"`; deploy script enforces this.
- Android worker target path is standardized as `/data/local/tmp/squalr-cli`.
- Interprocess spawn diagnostics now log worker command, su candidate resolution, and invocation-specific failures (`su -c`, `su 0 sh -c`, `su root sh -c`).
- Device status (2026-02-22): rooted `Pixel_9_Pro_Fold` (`adb serial: 4C101FDKD000Z8`, fingerprint `google/comet/comet:16/BP3A.251005.004.B3/14332485:user/release-keys`).
- Root-only shell validation now succeeds: `adb shell su -c id` returns `uid=0(root)`.
- Manual privileged worker launch validation succeeds: `debug_run_privileged_shell.py` launches `squalr-cli --ipc-mode` and `pidof squalr-cli` reports a running pid.
- App launch now reaches all Android bootstrap breadcrumbs through first-frame submission (`reportedDrawn=true`, no splash window entry).
- Active blocker: during app bootstrap, privileged worker spawn is reported as launched, but IPC listener immediately errors (`failed to fill whole buffer`) and subsequent command dispatches fail with `Broken pipe (os error 32)`.
- `build_and_deploy.py --debug --launch-log-seconds 30` currently fails final worker handshake polling because no long-lived `squalr-cli` pid remains after app launch.
- Process selector dispatch-failure guard was already landed; additional stale-request timeout guard now landed in GUI state to prevent infinite spinner when callbacks never arrive.
- Rooted UI validation (2026-02-22): after timeout guard, process selector spinner clears within a few seconds instead of remaining indefinitely.
- Host validation (2026-02-22): `cargo test -p squalr --lib -- --nocapture` passed (28 passed, 0 failed), including new process-selector stale-request tests.
