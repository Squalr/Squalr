# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- 

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Add Android smoke validation steps that are run and documented together:
  - host preflight (`ANDROID_HOME`, `ANDROID_NDK_ROOT`, `aarch64-linux-android-clang` visibility),
  - `cargo ndk ... build -p squalr-cli`,
  - `cargo apk build --target aarch64-linux-android --lib`,
  - `adb` install + launch + privileged worker IPC handshake.
- After migration, add at least one automated compile check path for Android (scripted local check or CI job) to prevent Slint-era regressions from reappearing.
- Decide whether to fully adopt owner-preferred architecture Option B (`squalr` as Android `cdylib`) and move Android deploy scripts to workspace root, without impacting current desktop VSCode launch/debug flows.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Android build break root causes were layered: missing Rust target, OpenSSL cross-link via `native-tls`, desktop-only `rfd` dependency on Android, stale Android OS layer implementations, and brittle CLI bundling path in `squalr-android`.
- `squalr-engine` now uses target-specific TLS for `ureq` (`rustls` on Android, `native-tls` elsewhere) and removes hard-coded `NativeTls` selection in code.
- Android memory reader/writer now use `/proc/<pid>/mem` with `FileExt::read_at`/`write_at`, avoiding unresolved `process_vm_readv/process_vm_writev` symbols on Android linker.
- Android memory/process querying currently reuses maintained Linux implementations in `squalr-engine-operating-system` for compile stability.
- Android workspace/IDE checks can fail with android-activity 0.6.0 missing backend features when `squalr` pulls `eframe` defaults on Android; enabling `eframe` feature `android-native-activity` in `squalr/Cargo.toml` unifies `android-activity` on `native-activity` and removes that mismatch.
- Android docs now prioritize a rooted-device quickstart that leads directly to a deployable GUI APK (`cargo apk build --release`), install via `adb`, and privileged worker verification via `su`.
- Corrected Android APK install path in README quickstart to include target triple output (`target/aarch64-linux-android/release/apk/squalr-android.apk`) and added a fallback locate command.
- `squalr-android/build_and_deploy.py` now handles end-to-end rooted deploy (APK install + worker push/chmod/verify) and falls back to debug APK when release signing is not configured.
- Restored interactive release prompt in `squalr-android/build_and_deploy.py`; running without flags now asks whether to use release mode, with `--release`/`--debug` available as explicit overrides.
- Fixed `squalr-android/build_and_deploy.py` APK install path resolution to support both `target/<triple>/<profile>/apk/...` and `target/<profile>/apk/...` output layouts from `cargo apk`.
- Added `squalr-android/run_apk.py` to launch the installed Android app over adb without rebuilding; it resolves launch activity across known package ids and supports a `--package` override.
- Set explicit Android app id in `squalr-android/Cargo.toml` (`[package.metadata.android] package = "com.squalr.android"`) and updated `run_apk.py` to default-launch only that package, with optional legacy fallback flag.
- `squalr` now exposes shared GUI bootstrap entrypoints (`run_gui`, `run_gui_android`), and `squalr-android` is a thin Android launcher wrapper calling that shared startup path.
- `squalr-android/src/lib.rs` no longer references Slint APIs and no longer uses panic-based startup; startup failures are logged.
- Removed obsolete Android CLI bundling bootstrap from `squalr-android` (`build.rs` deleted, legacy unpack flow removed).
- Android privileged worker runtime path is now standardized to `/data/local/tmp/squalr-cli` in both engine spawn (`interprocess_engine_api_unprivileged_bindings.rs`) and `debug_run_privilged_shell.py`.
- Local Android target checks currently fail before crate-level validation due to missing NDK toolchain wiring (`aarch64-linux-android-clang`/sysroot headers like `assert.h`), so preflight toolchain verification should be explicit in docs/scripts.
