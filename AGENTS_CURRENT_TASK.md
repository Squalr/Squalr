# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- 

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Run rooted-device validation once on a host with Android SDK/NDK env vars + connected rooted device, then capture successful smoke output (install + launch + privileged worker check). Progress (2026-02-22): `python ./squalr-android/build_and_deploy.py --compile-check` succeeds fully; `python ./squalr-android/build_and_deploy.py --debug` builds + installs APK and pushes worker on a connected device, but fails at privileged step with `/system/bin/sh: su: inaccessible or not found` when running `adb shell su -c 'chmod +x /data/local/tmp/squalr-cli'`. Remaining blocker is rooted-device availability (or root shell access) rather than host toolchain configuration. Follow-up (2026-02-22): owner observed launcher showing default Android app/icon. Root cause: `cargo apk` emits `android.app.NativeActivity` and was not using the checked-in `android/AndroidManifest.xml`; launcher scripts were targeting `android_activity.MainActivity` and APK metadata had default label/icon. Fixes applied: launch scripts now target `com.squalr.android/android.app.NativeActivity`; `squalr-android/Cargo.toml` now sets Android resources + label/icon metadata; `android/res/drawable/app_icon.png` added. Validation: `python ./squalr-android/build_and_deploy.py --compile-check` succeeds and `aapt dump badging target/debug/apk/squalr-android.apk` now reports `application-label:'Squalr'` and `application-icon-160:'res/drawable/app_icon.png'`. Pending: confirm visual launch behavior on a physical device after reinstall.
- Clean up squalr-android once all scripts and logic are 100% migrated out and accounted for (the point was to get rid of squalr-android and move the scripts to the project root).

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
- Android privileged worker runtime path is now standardized to `/data/local/tmp/squalr-cli` in both engine spawn (`interprocess_engine_api_unprivileged_bindings.rs`) and Android helper scripts.
- Script cleanup (2026-02-22): removed misspelled legacy helper `debug_run_privilged_shell.py` and replaced it with `debug_run_privileged_shell.py` that runs `adb shell su -c <cli> --ipc-mode` without `shell=True`.
- Local Android target checks currently fail before crate-level validation due to missing NDK toolchain wiring (`aarch64-linux-android-clang`/sysroot headers like `assert.h`), so preflight toolchain verification should be explicit in docs/scripts.
- `squalr-android/build_and_deploy.py` now performs host preflight checks (`ANDROID_HOME`, `ANDROID_NDK_ROOT`, Rust Android target, `aarch64-linux-android-clang`, cargo-ndk/cargo-apk presence), runs `cargo ndk ... -p squalr-cli`, runs `cargo apk build --lib`, and in smoke mode performs install + app launch + privileged worker process validation.
- Added non-device automation path: `python ./squalr-android/build_and_deploy.py --compile-check` for preflight + Android compile validation without adb deployment.
- Architecture decision: keep Option B (`squalr` Android `cdylib` + thin `squalr-android` launcher) and keep deploy scripts in `squalr-android/` for now to avoid disrupting current desktop VSCode launch/debug paths.
- Script audit follow-up (2026-02-22): modernized `squalr-android/debug_run_privileged_shell.py` with adb/device/root/CLI preflight checks and direct process I/O streaming; no additional legacy Android helper scripts remain.
- Android preflight fixes (2026-02-22): `squalr-android/build_and_deploy.py` now recognizes modern NDK clang layouts (`aarch64-linux-android*-clang*` under `toolchains/llvm/prebuilt/*/bin`, including Windows `.cmd` wrappers) plus legacy `build/core/toolchains` path; cargo-apk presence check now uses `cargo apk --help` (compatible with current cargo-apk CLI).
- Android compile fixes (2026-02-22): `squalr` Android bootstrap now imports `AndroidApp` from `android-activity` directly (no `eframe::winit` re-export assumption); Android default dock layout imports are available on Android builds; Android build no longer links desktop audio stack (`rodio`) by gating `audio_player` module + dependency to non-Android targets.
- Validation run (2026-02-22): compile-check completed successfully (CLI cross-build + APK build/sign); full debug deploy reached device install/push stage and fails only at `su` invocation because the connected device shell does not provide root.
- Launcher + APK identity fix (2026-02-22): `run_apk.py` and `build_and_deploy.py` now launch explicit `com.squalr.android/android.app.NativeActivity`; Android metadata now includes `label = "Squalr"`, `icon = "@drawable/app_icon"`, and `resources = "android/res"` so built APK badging no longer shows empty/default app identity.
- Compatibility alias (2026-02-22): added `squalr-android/run-apk.py` wrapper so the owner command with a hyphen works identically to `run_apk.py`.
