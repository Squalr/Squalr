# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- 

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Run one rooted-device smoke validation and capture the exact successful transcript (install + launch + privileged worker check).
- Verify final on-device launcher identity after reinstall (`Squalr` label + custom icon, `NativeActivity` launch path).
- Align Android script placement with owner direction (migrate/alias scripts into workspace root or document explicit reason to keep under `squalr-android/`).

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Android blockers were layered: missing Rust target, OpenSSL cross-link via `native-tls`, Android-incompatible desktop dependency (`rfd`), stale Android OS layer code, and brittle CLI bundling assumptions.
- `squalr-engine` now uses target-specific TLS for `ureq` (`rustls` on Android, `native-tls` elsewhere) and no longer hard-codes `NativeTls`.
- Android read/write now uses `/proc/<pid>/mem` with `FileExt::read_at`/`write_at`, avoiding unresolved `process_vm_*` linker symbols.
- Android memory/process querying currently reuses maintained Linux implementations in `squalr-engine-operating-system` for compile stability.
- Android build unification fix: `squalr/Cargo.toml` enables `eframe` feature `android-native-activity`, preventing `android-activity` backend mismatch.
- Shared bootstrap is in place: `squalr` exposes `run_gui` + `run_gui_android`; `squalr-android` is a thin launcher wrapper.
- Legacy Android CLI unpack/bootstrap path was removed (`squalr-android/build.rs` deleted); privileged worker path is standardized to `/data/local/tmp/squalr-cli`.
- `squalr-android/build_and_deploy.py` now does host preflight, CLI cross-build, APK build, optional install/launch, and privileged worker validation; includes `--compile-check`, `--debug`, `--release`.
- NDK preflight now recognizes modern LLVM toolchain layouts (including Windows `.cmd` wrappers) and checks `cargo apk --help` for cargo-apk availability.
- Launch and identity fixes: scripts target `com.squalr.android/android.app.NativeActivity`; Android metadata sets `label = "Squalr"`, `icon = "@drawable/app_icon"`, and `resources = "android/res"`.
- Validation status (2026-02-22): compile-check passes; debug deploy reaches install/push but fails at `su` step on non-rooted shell (`/system/bin/sh: su: inaccessible or not found`).
- Remaining external dependency is rooted-device access to complete end-to-end privileged smoke validation.
