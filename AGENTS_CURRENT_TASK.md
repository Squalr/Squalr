# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- Assume any unstaged file changes are from a previous iteration, and can be kept if they look good
- The android device is rooted.
- You don't get to declare things as fixed. Only "need human verification".
- Keep .idea/ in gitignore you keep fucking this up. The goal is not to undo ALL changes from main. We want good changes. The goal is to eliminate stupid and speculative changes. Formatting is fine. Gitignore was fine.

## WONTFIX (For now)
- Add multi-data-type scan parity to GUI element scanner (`squalr/src/views/element_scanner/scanner/view_data/element_scanner_view_data.rs`) so one scan request can include multiple selected data types like TUI.
- Add GUI process list search/filter input parity with TUI process selector (`squalr/src/views/process_selector`) including in-memory filtering and refresh-aware state behavior.
- Add GUI project selector search/filter parity with TUI project list workflows (`squalr/src/views/project_explorer/project_selector`) so large project lists can be searched quickly.
- Add GUI output window controls parity with TUI (`squalr/src/views/output/output_view.rs`): clear log action and configurable max-line cap.
- Complete GUI settings parity with TUI for missing controls in memory/scan tabs (`squalr/src/views/settings/settings_tab_memory_view.rs`, `squalr/src/views/settings/settings_tab_scan_view.rs`), including start/end address editing, memory alignment, memory read mode, and floating-point tolerance.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).
- Need human verification: visually confirm on-device Android GUI process dropdown now renders the expected full windowed list after windowed-list normalization + scroll-state reset in:
  - `squalr/src/views/process_selector/view_data/process_selector_view_data.rs`.
  - `squalr/src/views/main_window/main_shortcut_bar_view.rs`.
  human: latest report says fixes failed again.
  - Previous attempt failed before this change: dropdown showed 2 random/non-windowed rows (example: `com.google.android.euic`).

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Android windowed filtering now requires primary package match (`cmdline == package`) to avoid false positives from colon-suffixed service processes.
- Android windowed classification now requires zygote ancestry anywhere in the parent chain, including resilient zygote name matching across `cmdline` and `comm`.
- Android package-path lookup fallback order is `/data/app` -> `packages.xml` -> `pm list packages -f`.
- Added Android unit tests for primary-process classification, zygote-ancestor lineage detection, cycle safety, package-manager parser coverage, and zygote-name variants (`android_process_query.rs`).
- Added process dropdown UX updates: vertical scroll area plus clipped/truncated long process names for combo-box rows.
- Process dropdown rendering now uses conditional scroll behavior: render direct rows when result count is small, otherwise enable capped-height scroll area (`squalr/src/views/main_window/main_shortcut_bar_view.rs`).
- Windowed process lists are now defensively normalized in GUI view-data before rendering: enforce `is_windowed == true` and sort deterministically by case-insensitive name then PID (`squalr/src/views/process_selector/view_data/process_selector_view_data.rs`).
- Main shortcut bar dropdown scroll area now salts its state with a per-refresh nonce to avoid stale scroll offsets reopening at trailing rows (`squalr/src/views/main_window/main_shortcut_bar_view.rs`).
- Added GUI unit tests for windowed-process normalization behavior (filter non-windowed entries + deterministic ordering) in `squalr/src/views/process_selector/view_data/process_selector_view_data.rs`.
- Root cause for missing GUI rows was dependency replacement race; `ProcessSelectorViewData` is now single-registered in `main_window_view.rs` and consumed in `process_selector_view.rs` via shared dependency lookup.
- GUI/TUI parity audit was completed on 2026-02-22; current gaps are listed under WONTFIX.
- Validation baseline (latest run: 2026-02-22) passed: `cargo fmt --all`, `cargo test -p squalr-tests --locked`, `cargo check -p squalr-engine-operating-system --target aarch64-linux-android --locked`, `cargo check -p squalr --locked`.
- Session validation refresh (2026-02-22): passed `cargo fmt --all`, `cargo test -p squalr-tests --locked`, `cargo test -p squalr process_selector_view_data --locked`, and `cargo check -p squalr --locked` (warnings unchanged/pre-existing).
- Android compile/deploy checks passed on 2026-02-22: `python ./build_and_deploy.py --compile-check` and `python ./build_and_deploy.py --debug`.
- CLI-side rooted verification passed on 2026-02-22: `adb shell su -c "/data/local/tmp/squalr-cli process list -w -l 300"` showed `com.squalr.android` in windowed results.
- GUI-side runtime logs from deploy on 2026-02-22 show process selector requests/responses with non-empty results (`Received windowed process-list response with 67 entries.`).
- Targeted revalidation (2026-02-22) passed: `cargo test -p squalr process_selector_view_data --locked` and `cargo check -p squalr --locked` (same pre-existing warnings only).
- Session revalidation (2026-02-22) passed again: `cargo test -p squalr process_selector_view_data --locked` and `cargo check -p squalr --locked`; warnings remain pre-existing and unchanged.
- Session revalidation (2026-02-22, follow-up) passed again: `cargo test -p squalr process_selector_view_data --locked` and `cargo check -p squalr --locked`; warnings remain pre-existing and unchanged.
- Session revalidation (2026-02-22, latest) passed: `cargo fmt --all`, `cargo test -p squalr process_selector_view_data --locked`, and `cargo check -p squalr --locked`; warnings remain pre-existing and unchanged.
- Current session (2026-02-22) remains blocked on on-device-only visual verification of Android GUI process dropdown row correctness; local CLI/tests/checks cannot replace that validation.
- `cargo check -p squalr --locked` still reports existing GUI unused-variable/unreachable-pattern warnings, with no new failures.
- Direct `cargo check -p squalr --target aarch64-linux-android --locked` may fail in this environment due `aarch64-linux-android-clang` pathing for `ring`; use `cargo ndk` / deploy script paths for Android validation.
- Lockfile regeneration is currently blocked in this environment by yanked crate requirement `zip = "^7.4.0"` from `squalr-engine`.
