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
- Need human verification: visually confirm on-device Android GUI process dropdown renders correct windowed process rows after dropdown list height behavior fix in `main_shortcut_bar_view.rs` (small lists render without `ScrollArea`; larger lists use capped scroll region).
    - Previous attempt failed: windowed list showed 2 random/non-windowed rows (for example `com.google.android.euic`).

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Android windowed filtering now requires primary package match (`cmdline == package`) to avoid false positives from colon-suffixed service processes.
- Android windowed classification now requires zygote ancestry anywhere in the parent chain, including resilient zygote name matching across `cmdline` and `comm`.
- Android package-path lookup fallback order is `/data/app` -> `packages.xml` -> `pm list packages -f`.
- Added Android unit tests for primary-process classification, zygote-ancestor lineage detection, cycle safety, package-manager parser coverage, and zygote-name variants (`android_process_query.rs`).
- Added process dropdown UX updates: vertical scroll area plus clipped/truncated long process names for combo-box rows.
- Process dropdown rendering now uses conditional scroll behavior: render direct rows when result count is small, otherwise enable capped-height scroll area (`squalr/src/views/main_window/main_shortcut_bar_view.rs`).
- Root cause for missing GUI rows was dependency replacement race; `ProcessSelectorViewData` is now single-registered in `main_window_view.rs` and consumed in `process_selector_view.rs` via shared dependency lookup.
- GUI/TUI parity audit was completed on 2026-02-22; current gaps are listed under WONTFIX.
- Validation baseline used repeatedly on 2026-02-22: `cargo fmt --all`, `cargo test -p squalr-tests --locked`, `cargo check -p squalr-engine-operating-system --target aarch64-linux-android --locked`, `cargo check -p squalr --locked`.
- Android compile/deploy checks passed on 2026-02-22: `python ./build_and_deploy.py --compile-check` and `python ./build_and_deploy.py --debug`.
- CLI-side rooted verification passed on 2026-02-22: `adb shell su -c "/data/local/tmp/squalr-cli process list -w -l 300"` showed `com.squalr.android` in windowed results.
- GUI-side runtime logs from deploy on 2026-02-22 show process selector requests/responses with non-empty results (`Received windowed process-list response with 67 entries.`).
- Direct `cargo check -p squalr --target aarch64-linux-android --locked` may fail in this environment due `aarch64-linux-android-clang` pathing for `ring`; use `cargo ndk` / deploy script paths for Android validation.
- Lockfile regeneration is currently blocked in this environment by yanked crate requirement `zip = "^7.4.0"` from `squalr-engine`.
