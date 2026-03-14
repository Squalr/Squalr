# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/docking`

# Notes from Owner (Readonly Section)
- Assume any unstaged/uncomitted file changes are from a previous iteration (or if this file, probably the human author giving guidance), and can be kept if they look good. Do not ask me about them.
- Assume any connected android devices are rooted, and assume MacOS has SIP disabled.
- You don't get to declare things as fixed. Only "need human verification".

## WONTFIX (For now)
- Add GUI process list search/filter input parity with TUI process selector (`squalr/src/views/process_selector`) including in-memory filtering and refresh-aware state behavior.
- Add GUI project selector search/filter parity with TUI project list workflows (`squalr/src/views/project_explorer/project_selector`) so large project lists can be searched quickly.
- Add GUI output window controls parity with TUI (`squalr/src/views/output/output_view.rs`): clear log action and configurable max-line cap.
- Complete GUI settings parity with TUI for missing controls in memory/scan tabs (`squalr/src/views/settings/settings_tab_memory_view.rs`, `squalr/src/views/settings/settings_tab_scan_view.rs`), including start/end address editing, memory alignment, memory read mode, and floating-point tolerance.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Need human verification: `pr/multi-scan` pass 2 keeps the GUI element scanner dropdown as a balanced two-column checkbox popup, restores click-drag painting across contiguous entries, leaves string/custom as clickable facade rows without checkboxes, and still dispatches scans with all selected data types.
- Need human verification: multi-data-type scans no longer inflate scan result metadata to massive sizes when one selected data type yields zero matches; `SnapshotRegionScanResults::get_filter_bounds()` now ignores zero-result collections so region resizing stays bounded to actual matches.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- User explicitly requested the previously deferred GUI multi-data-type scan work. Implemented in `squalr` only: selector state now tracks an active type plus selected set, the dropdown stays open for checkbox multi-select, dragging with the primary mouse button applies the initial select/deselect state across hovered entries, and scan dispatch now sends every selected `DataTypeRef`.
- Pass 2 final follow-up keeps the two-column popup stable by rendering the data type rows through fixed-width grids and clipping each item to its own bounds in `squalr/src/ui/widgets/controls/data_type_selector/data_type_selector_view.rs` and `squalr/src/ui/widgets/controls/data_type_selector/data_type_item_view.rs`, which prevents the right column from painting into the left. String/custom facade rows are clickable again but remain unchecked/non-multi-select. `cargo test -p squalr --locked` passed.
- Investigated inflated scan result sizes on multi-type scans. Root cause was `squalr-engine-api/src/structures/results/snapshot_region_scan_results.rs`: when one data type collection had zero results, `get_filter_bounds()` still treated its minimum address as `0`, so `SnapshotRegion::resize_to_filters()` could expand a region to `0..high_user_space_address` and report totals in the TB range. Fixed by skipping zero-result collections when computing bounds. Added regressions in `squalr-engine-api` and `squalr-tests` covering the high-address partial multi-type case. `cargo test -p squalr-engine-api --locked get_filter_bounds_ignores_zero_result_collections` and `cargo test -p squalr-tests --locked --test os_behavior_command_tests` passed. Full `cargo test -p squalr-engine-api --locked` still has a pre-existing unrelated failure in `utils::file_system::file_system_utils::tests::is_cross_platform_absolute_path_detects_unix_absolute_paths` on Windows.
