# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- 

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- [x] Restore Android cross-build for `squalr-cli` and `squalr-android`.
- [x] Update docs with current Android build/run workflow.
- [x] Add VS Code launch entry that performs Android build-only flow.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Android build break root causes were layered: missing Rust target, OpenSSL cross-link via `native-tls`, desktop-only `rfd` dependency on Android, stale Android OS layer implementations, and brittle CLI bundling path in `squalr-android`.
- `squalr-engine` now uses target-specific TLS for `ureq` (`rustls` on Android, `native-tls` elsewhere) and removes hard-coded `NativeTls` selection in code.
- Android memory reader/writer now use `/proc/<pid>/mem` with `FileExt::read_at`/`write_at`, avoiding unresolved `process_vm_readv/process_vm_writev` symbols on Android linker.
- Android memory/process querying currently reuses maintained Linux implementations in `squalr-engine-operating-system` for compile stability.
- `squalr-android/build.rs` now bundles `target/<triple>/<profile>/squalr-cli` into `OUT_DIR/squalr-cli-bundle` and fails with a clear message if the CLI has not been built first.
- Android workspace/IDE checks can fail with android-activity 0.6.0 missing backend features when `squalr` pulls `eframe` defaults on Android; enabling `eframe` feature `android-native-activity` in `squalr/Cargo.toml` unifies `android-activity` on `native-activity` and removes that mismatch.
- Android docs now prioritize a rooted-device quickstart that leads directly to a deployable GUI APK (`cargo apk build --release`), install via `adb`, and privileged worker verification via `su`.
