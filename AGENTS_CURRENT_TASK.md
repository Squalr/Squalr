# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- 

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Resolve Android privileged worker spawn ENOENT from app context (`Failed to spawn privileged CLI process... No such file or directory`) despite `/data/local/tmp/squalr-cli` existing and executable.
- Once worker spawn succeeds, rerun launch diagnostics and confirm breadcrumb progression past `After SqualrEngine::new.`, `After App::new.`, and `Before first frame submission.`.
- If first-frame breadcrumb appears but splash persists (`reportedDrawn=false`), inspect `eframe`/`winit` Android lifecycle callbacks and draw signal timing in app construction.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Android blockers were layered: missing Rust target, OpenSSL cross-link via `native-tls`, Android-incompatible desktop dependency (`rfd`), stale Android OS layer code, and brittle CLI bundling assumptions.
- `squalr-engine` now uses target-specific TLS for `ureq` (`rustls` on Android, `native-tls` elsewhere) and no longer hard-codes `NativeTls`.
- Android read/write now uses `/proc/<pid>/mem` with `FileExt::read_at`/`write_at`, avoiding unresolved `process_vm_*` linker symbols.
- Android memory/process querying currently reuses maintained Linux implementations in `squalr-engine-operating-system` for compile stability.
- Android build unification fix: `squalr/Cargo.toml` enables `eframe` feature `android-native-activity`, preventing `android-activity` backend mismatch.
- Android bootstrap is now owned directly by `squalr`: `android_main` is in `squalr/src/lib.rs` and package metadata/resources are in `squalr/Cargo.toml` + `squalr/android/`.
- Privileged worker path is standardized to `/data/local/tmp/squalr-cli`.
- Workspace-root scripts (`build_and_deploy.py`, `run_apk.py`, `debug_run_privileged_shell.py`) are now canonical and invoke Android builds from `squalr/`.
- NDK preflight now recognizes modern LLVM toolchain layouts (including Windows `.cmd` wrappers) and checks `cargo apk --help` for cargo-apk availability.
- Launch and identity fixes: scripts target `com.squalr.android/android.app.NativeActivity`; Android metadata sets `label = "Squalr"`, `icon = "@drawable/app_icon"`, and `resources = "android/res"`.
- Android launcher mismatch fix (2026-02-22): `squalr/android/AndroidManifest.xml` now declares `android.app.NativeActivity` with `android.app.lib_name = "squalr"` instead of `android_activity.MainActivity`; `squalr/src/lib.rs` exports `pub fn android_main(...)` for reliable symbol discovery.
- Deployment guard (2026-02-22): `build_and_deploy.py` now verifies resolved launcher identity before worker deployment and fails fast on component mismatch.
- Validation status (2026-02-22): compile-check passes; debug deploy reaches install/push but fails at `su` step on non-rooted shell (`/system/bin/sh: su: inaccessible or not found`).
- Validation status (2026-02-22): reran `python build_and_deploy.py --debug`; host preflight passed, APK reinstall succeeded, launcher resolution returned `com.squalr.android/android.app.NativeActivity`, and worker push succeeded before failing at `adb shell su -c 'chmod +x /data/local/tmp/squalr-cli'` on non-rooted shell.
- Launcher identity verification (2026-02-22): `aapt dump badging target/debug/apk/squalr.apk` reports `application-label:'Squalr'` and `application-icon-160:'res/drawable/app_icon.png'`; on-device resolver confirms `com.squalr.android/android.app.NativeActivity`.
- Migration completion status (2026-02-22): `squalr-android` crate removed from workspace and deleted after moving Android launcher/resources/scripts to `squalr` + workspace root.
- Remaining external dependency is rooted-device access to complete end-to-end privileged smoke validation.
- Launch diagnostics tooling (2026-02-22): `build_and_deploy.py` now supports `--skip-worker`, `--launch-log-seconds`, and `--launch-log-file`; it force-stops the app, clears logcat, launches, then captures PID/activity dump + filtered logcat.
- Launch diagnostics tooling (2026-02-22): `run_apk.py` now mirrors launch diagnostics capture with `--launch-log-seconds` and `--launch-log-file` for repeat repro runs without rebuilding.
- Launch diagnostics run (2026-02-22): `python build_and_deploy.py --debug --skip-worker --launch-log-file target/android_launch_logcat.txt` completed successfully on-device; filtered log showed no `AndroidRuntime`, `DEBUG`, or `libc` errors for `com.squalr.android`.
- Repro run (2026-02-22): `python run_apk.py --launch-log-file target/android_launch_logcat_rerun.txt` again showed clean filtered logcat, process alive (`pidof` resolved), and `NativeActivity` resumed.
- Hang characterization (2026-02-22): `dumpsys activity` and `dumpsys window` show app remains on `Splash Screen com.squalr.android` with `reportedDrawn=false`; unfiltered logcat confirms `libsqualr.so` loads successfully, suggesting a post-load native startup/draw-path stall rather than an immediate Java crash.
- Startup breadcrumb instrumentation (2026-02-22): `squalr` now initializes `android_logger` in `android_main` and emits `[android_bootstrap]` markers around native options, engine construction, app creation, engine initialization, and first-frame submission (`App::update` one-time marker).
- Launch diagnostic rerun (2026-02-22): `python build_and_deploy.py --debug --skip-worker --launch-log-file target/android_launch_logcat_startup_trace.txt` completed; script-filtered log still omits app-tag logs by design.
- Breadcrumb capture (2026-02-22): direct `adb logcat` with `Squalr:I` confirms startup stops at `[android_bootstrap] Before SqualrEngine::new.` followed by `Failed to spawn privileged CLI process for unprivileged host startup: No such file or directory (os error 2)`.
- Verification detail (2026-02-22): even after manual `adb push target/aarch64-linux-android/debug/squalr-cli /data/local/tmp/squalr-cli` and `adb shell chmod 755`, app launch still reports the same ENOENT spawn failure.
