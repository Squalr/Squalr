# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- 

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Run one rooted-device smoke validation and capture the exact successful transcript (install + launch + privileged worker check).
- Verify final on-device launcher identity after reinstall (`Squalr` label + custom icon) and confirm resolved launcher component is `com.squalr.android/android.app.NativeActivity`.

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
- Migration completion status (2026-02-22): `squalr-android` crate removed from workspace and deleted after moving Android launcher/resources/scripts to `squalr` + workspace root.
- Remaining external dependency is rooted-device access to complete end-to-end privileged smoke validation.
