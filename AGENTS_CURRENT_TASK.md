# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/docking-fixes`

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

- Need human verification: dragging a docked window onto an existing tab group inserts it as the first tab, dragging footer tabs reorders left/right based on hover half, footer tabs can still be dragged into other dock panels, self-center drop stays hidden for same-panel targets, and standalone windows still hide all self-drop targets while multi-tab panels keep cardinal self-drops.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Footer tab buttons now start dock drags, so inactive tabs can be moved without first selecting them.
- Drop overlay filtering now treats the current tab group as the same dock panel: center self-drops are hidden there, and cardinal self-drops remain only when the source window is part of a multi-tab group.
- Center drops into an existing tab group now insert the dragged window at tab index 0 and activate it.
- Footer tab drag hover now resolves against the hovered tab button itself: left half inserts before the target tab, right half inserts after it, with a footer-edge preview strip during drag.
