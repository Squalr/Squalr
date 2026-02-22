# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- 

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Replace legacy Slint Android bootstrap in `squalr-android/src/lib.rs` with the current egui app bootstrap path. Current file still uses `slint::android::AndroidApp`, `slint::run_event_loop()`, and panic-based startup.
- Decide Android app entry architecture and implement it consistently:
  - Option A: make `squalr-android` a thin Android launcher wrapper that calls shared GUI startup from the `squalr` crate.
  - Option B: make `squalr` itself produce the Android `cdylib` and retire duplicate Android GUI bootstrap code.
- Eliminate hard-coded legacy package id paths (`/data/data/rust.squalr_android/...`) and switch to a single source of truth for worker binary path based on `com.squalr.android`.
- Align privileged worker deployment and runtime execution path:
  - `build_and_deploy.py` currently pushes worker to `/data/local/tmp/squalr-cli`.
  - Android runtime spawn path currently expects `/data/data/<package>/files/squalr-cli`.
  - Pick one path strategy and enforce it in scripts + engine spawn logic.
- Update Android helper scripts (`debug_run_privilged_shell.py`, launch/deploy scripts) to use the same package id/path strategy and remove stale legacy defaults.
- Add Android smoke validation steps that are run and documented together:
  - host preflight (`ANDROID_HOME`, `ANDROID_NDK_ROOT`, `aarch64-linux-android-clang` visibility),
  - `cargo ndk ... build -p squalr-cli`,
  - `cargo apk build --target aarch64-linux-android --lib`,
  - `adb` install + launch + privileged worker IPC handshake.
- After migration, add at least one automated compile check path for Android (scripted local check or CI job) to prevent Slint-era regressions from reappearing.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Android build break root causes were layered: missing Rust target, OpenSSL cross-link via `native-tls`, desktop-only `rfd` dependency on Android, stale Android OS layer implementations, and brittle CLI bundling path in `squalr-android`.
- `squalr-engine` now uses target-specific TLS for `ureq` (`rustls` on Android, `native-tls` elsewhere) and removes hard-coded `NativeTls` selection in code.
- Android memory reader/writer now use `/proc/<pid>/mem` with `FileExt::read_at`/`write_at`, avoiding unresolved `process_vm_readv/process_vm_writev` symbols on Android linker.
- Android memory/process querying currently reuses maintained Linux implementations in `squalr-engine-operating-system` for compile stability.
- `squalr-android/build.rs` now bundles `target/<triple>/<profile>/squalr-cli` into `OUT_DIR/squalr-cli-bundle` and fails with a clear message if the CLI has not been built first.
- Android workspace/IDE checks can fail with android-activity 0.6.0 missing backend features when `squalr` pulls `eframe` defaults on Android; enabling `eframe` feature `android-native-activity` in `squalr/Cargo.toml` unifies `android-activity` on `native-activity` and removes that mismatch.
- Android docs now prioritize a rooted-device quickstart that leads directly to a deployable GUI APK (`cargo apk build --release`), install via `adb`, and privileged worker verification via `su`.
- Corrected Android APK install path in README quickstart to include target triple output (`target/aarch64-linux-android/release/apk/squalr-android.apk`) and added a fallback locate command.
- `squalr-android/build_and_deploy.py` now handles end-to-end rooted deploy (APK install + worker push/chmod/verify) and falls back to debug APK when release signing is not configured.
- Restored interactive release prompt in `squalr-android/build_and_deploy.py`; running without flags now asks whether to use release mode, with `--release`/`--debug` available as explicit overrides.
- Fixed `squalr-android/build_and_deploy.py` APK install path resolution to support both `target/<triple>/<profile>/apk/...` and `target/<profile>/apk/...` output layouts from `cargo apk`.
- Added `squalr-android/run_apk.py` to launch the installed Android app over adb without rebuilding; it resolves launch activity across known package ids and supports a `--package` override.
- Set explicit Android app id in `squalr-android/Cargo.toml` (`[package.metadata.android] package = "com.squalr.android"`) and updated `run_apk.py` to default-launch only that package, with optional legacy fallback flag.
- `squalr-android/src/lib.rs` is still Slint-era bootstrap code and currently references `slint::*` APIs despite no `slint` dependency in `squalr-android/Cargo.toml`; this is now the primary code-level Android blocker.
- `squalr-engine/src/engine_bindings/interprocess/interprocess_engine_api_unprivileged_bindings.rs` still spawns Android worker from `/data/data/rust.squalr_android/files/squalr-cli`, which conflicts with current package id `com.squalr.android`.
- Android worker path strategy is internally inconsistent: deploy script verifies `/data/local/tmp/squalr-cli` while engine runtime expects `/data/data/<package>/files/squalr-cli`.
- Local Android target checks currently fail before crate-level validation due to missing NDK toolchain wiring (`aarch64-linux-android-clang`/sysroot headers like `assert.h`), so preflight toolchain verification should be explicit in docs/scripts.
