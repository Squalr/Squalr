# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- 

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Run `python build_and_deploy.py --debug --skip-worker --launch-log-file target/android_launch_logcat.txt` on a connected device to capture launch diagnostics without requiring root.
- Inspect `target/android_launch_logcat.txt` for `AndroidRuntime`, `DEBUG`, `libc`, and activity lifecycle errors tied to `com.squalr.android`.
- If launch diagnostics are clean but app still hangs on icon screen, run `python run_apk.py --launch-log-file target/android_launch_logcat_rerun.txt` for a second repro capture.

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
