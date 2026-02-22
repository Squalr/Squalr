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

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Root cause for non-Android GUI boot crash: `ureq` v3 defaults TLS provider to `Rustls`, but desktop `squalr-engine` dependency enables only `native-tls`; version check thread panicked on first HTTPS request to GitHub.
- Mitigation implemented: added `AppProvisionerHttpClient` to create a configured `ureq` agent with target-specific TLS provider (`NativeTls` on non-Android, `Rustls` on Android), and routed version-check/download HTTP calls through it.
- Local verification: `cargo run -p squalr` no longer panics at boot; update check completes over HTTPS with `ureq::tls::native_tls` logs.
- Note: `cargo test -p squalr-engine` has one environment-specific failure in `squalr_engine::tests::privileged_shell_does_not_create_unprivileged_state` due named pipe access denied on this host.
