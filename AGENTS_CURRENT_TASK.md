# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/pointer-scanning`

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

- Owner: Removed to compact
- Need human verification: confirm the pointer-scanner results grid now stays inside the visible panel, uses `Module` / `Value` / `Resolved` / `Depth`, removes the old `Static` / `Action` columns, shows the inline right-arrow only on expandable rows with no leftover gap on leaf rows, and reports sane branch depth as `x of y` rather than inverted values like `5 of 2`.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Owner: Removed to compact
- Pointer-scanner result rows now need width-safe clipped painting, inline disclosure inside the primary column, and branch-depth text derived from `display depth + discovery depth - 1` rather than raw discovery depth alone.
- Pointer-scanner node materialization now carries an explicit `branch_total_depth` from the root static through every expanded child node, so the UI can render depth as the stable rule `root = 1 of y`, then `2 of y`, `3 of y`, etc. without trying to recompute `y` from the current context.
