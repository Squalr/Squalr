# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- Assume any unstaged file changes are from a previous iteration, and can be kept if they look good
- The android device is rooted.
- You don't get to declare things as fixed. Only "need human verification".

## WONTFIX (For now)
- Add multi-data-type scan parity to GUI element scanner (`squalr/src/views/element_scanner/scanner/view_data/element_scanner_view_data.rs`) so one scan request can include multiple selected data types like TUI.
- Add GUI process list search/filter input parity with TUI process selector (`squalr/src/views/process_selector`) including in-memory filtering and refresh-aware state behavior.
- Add GUI project selector search/filter parity with TUI project list workflows (`squalr/src/views/project_explorer/project_selector`) so large project lists can be searched quickly.
- Add GUI output window controls parity with TUI (`squalr/src/views/output/output_view.rs`): clear log action and configurable max-line cap.
- Complete GUI settings parity with TUI for missing controls in memory/scan tabs (`squalr/src/views/settings/settings_tab_memory_view.rs`, `squalr/src/views/settings/settings_tab_scan_view.rs`), including start/end address editing, memory alignment, memory read mode, and floating-point tolerance.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Need human verification: Validate non-Android GUI boot on a clean desktop environment after Android support changes, confirming startup no longer crashes during update/version check.
- Need human verification: Validate desktop GUI shortcut process dropdown shows only true windowed processes (no background task leakage) after restoring non-Android windowed-only sourcing.
- Need human verification: Validate desktop GUI process selector in `Windowed` mode only shows true windowed processes (no broad non-windowed `.exe` leakage) after Android fallback gating changes.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Root cause for non-Android GUI boot crash: `ureq` v3 defaults TLS provider to `Rustls`, but desktop `squalr-engine` dependency enables only `native-tls`; version check thread panicked on first HTTPS request to GitHub.
- Mitigation implemented: added `AppProvisionerHttpClient` to create a configured `ureq` agent with target-specific TLS provider (`NativeTls` on non-Android, `Rustls` on Android), and routed version-check/download HTTP calls through it.
- Local verification: `cargo run -p squalr` no longer panics at boot; update check completes over HTTPS with `ureq::tls::native_tls` logs.
- Note: `cargo test -p squalr-engine` has one environment-specific failure in `squalr_engine::tests::privileged_shell_does_not_create_unprivileged_state` due named pipe access denied on this host.
- Root cause for windowed-process regression on desktop: Android fallback in `ProcessSelectorViewData::normalize_windowed_processes_with_fallback` and shortcut fallback was unconditional; on Windows, many non-windowed process names match the fallback heuristic (`contains('.') && !contains(':')`), polluting the windowed list.
- Mitigation implemented: gated both fallbacks to Android only (`cfg!(target_os = "android")` via `IS_ANDROID_TARGET`), so non-Android windowed mode now uses strict `get_is_windowed()` filtering.
- Local verification: `cargo test -p squalr process_selector_view_data --locked` passes with platform-conditional assertions for fallback behavior.
- Root cause for persistent desktop leakage after fallback gating: shortcut dropdown selection logic sourced from full process list when `show_windowed_processes_only` was false (default non-Android), reintroducing background tasks despite strict windowed filtering.
- Mitigation implemented: restored legacy desktop shortcut behavior by always sourcing shortcut dropdown from `windowed_process_list` on non-Android and only refreshing full list from the shortcut on Android.
- Local verification: `cargo test -p squalr process_selector_view_data --locked` passes including `refresh_shortcut_dropdown_process_list_keeps_desktop_windowed_only_behavior`.
- Android logging hooks consolidated into shared `squalr-engine-session` platform layer: added `PlatformLogHooks` trait + dispatcher in `src/logging/platform/platform_log_hooks.rs`, implemented Android hook init in existing `src/logging/platform/android_log_hooks.rs`, and updated `squalr` + `squalr-cli` entry points to call `initialize_platform_log_hooks_once(...)` instead of crate-local Android logger setup.
