# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/multi-results`

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

- Need human verification of the GUI scan-results surviving-type filter dropdown defaulting to the scan-selected types, showing a single-type icon or multi-type `+n` text while keeping all selected filters enabled after a new scan, the 72px type column / wider address layout, address-ascending mixed-type ordering, and filtered paging behavior on a real scan.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Scan-results query/list requests now accept optional `data_type_filters`; GUI/TUI query paths send explicit filter sets so result counts and pages are computed against the active types instead of filtering a loaded page.
- Scan-results query/list responses now include surviving per-type result counts so UIs can hide eliminated data types without inferring from the current page.
- Snapshot paging now zips scan results by address ascending at page-load time, preserving stable unfiltered `ScanResultRef` global indices so refresh/freeze/edit flows still resolve the correct entries.
- GUI scan results now default the surviving-type filter to the scan-selected types, preserve that selection through the empty pre-results query that happens on a new scan, render the header filter with a single-type icon or multi-type `+n` label in a 72px type column, clip row text to its column bounds, show only surviving data types in a stacked filter popup, and start with equal value/previous-value widths to leave more room for the address column.
- Test coverage now covers address-ascending mixed-type ordering, filter-aware paging, the scan-results multi-selection `+n` filter label, preserving selected filters while surviving-type counts are temporarily empty, and equal default value/previous-value widths. `cargo test -p squalr --lib` passes. `cargo test -p squalr-engine-api --lib` still has a pre-existing unrelated failure in `utils::file_system::file_system_utils::tests::is_cross_platform_absolute_path_detects_unix_absolute_paths`.
