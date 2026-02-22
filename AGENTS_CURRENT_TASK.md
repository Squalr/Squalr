# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- Assume any unstaged file changes are from a previous iteration

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Verify Android privileged worker launch on a rooted device after Android `su` compatibility expansion in `InterprocessEngineApiUnprivilegedBindings` (attempts now include `su -c`, `su 0 sh -c`, and `su root sh -c` per candidate path).
- Once worker spawn succeeds, rerun launch diagnostics and confirm breadcrumb progression past `After SqualrEngine::new.`, `After App::new.`, and `Before first frame submission.` (scripts now summarize missing checkpoints directly from logcat).
- If first-frame breadcrumb appears but splash persists (`reportedDrawn=false`), inspect `eframe`/`winit` Android lifecycle callbacks and draw signal timing in app construction (using scripted `reportedDrawn` + splash-window summaries).

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Android blockers addressed: target/toolchain gaps, Android TLS/cross-linking, desktop-only dependency removal, stale OS-layer code paths, and CLI bundling assumptions.
- Android engine/platform updates: `ureq` uses `rustls` on Android, memory IO uses `/proc/<pid>/mem`, and Android currently reuses maintained Linux query paths for compile stability.
- Android build/runtime ownership moved into `squalr`: `android_main` in `squalr/src/lib.rs`, manifest/resources in `squalr/Cargo.toml` + `squalr/android/`, and `squalr-android` crate removed.
- Build/deploy scripts are workspace-root canonical entrypoints: `build_and_deploy.py`, `run_apk.py`, `debug_run_privileged_shell.py`.
- Launcher identity is pinned and validated as `com.squalr.android/android.app.NativeActivity` with `android.app.lib_name = "squalr"`, app label `Squalr`, and expected icon resources.
- Deploy guardrail exists: `build_and_deploy.py` fails fast when resolved launcher does not match expected component.
- Current external blocker remains rooted-device access; non-rooted validation consistently fails at `su` operations during worker deployment.
- Privileged worker target path is standardized: `/data/local/tmp/squalr-cli`.
- Launch diagnostics support `--skip-worker`, `--launch-log-seconds`, and `--launch-log-file` for non-rooted repro loops without rebuild.
- Log filtering now includes `Squalr:I` so Android bootstrap breadcrumbs are captured directly in script log outputs.
- Startup breadcrumbs instrumented around Android bootstrap: before/after engine creation, app construction, engine initialization, and first-frame submission.
- Current failure signature: startup stops at `Before SqualrEngine::new.` with privileged worker spawn failure (`ENOENT`) in unprivileged-host startup path.
- Worker command quoting fix landed: Android worker command is now `/data/local/tmp/squalr-cli --ipc-mode` without embedded path quotes.
- Additional spawn diagnostics landed (2026-02-22): interprocess bindings now log worker command, each su candidate attempt, unavailable su paths, and aggregated failure context.
- Launch diagnostics enhancement landed (2026-02-22): scripts now summarize `reportedDrawn` from `dumpsys activity` and splash-window presence from `dumpsys window`.
- Launch diagnostics enhancement landed (2026-02-22): `build_and_deploy.py` and `run_apk.py` now summarize Android bootstrap breadcrumb progression and explicitly list missing checkpoints through first-frame submission.
- Recent host-side validation status (2026-02-22): compile-check and debug deploy flows complete through install/push on test device, with expected `su` failure on non-rooted shell.
- Android privileged spawn compatibility expanded (2026-02-22): each `su` candidate now tries `su -c`, `su 0 sh -c`, and `su root sh -c` with per-invocation diagnostics; host unit tests pass for interprocess initialization paths.
