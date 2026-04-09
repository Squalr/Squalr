# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- Assume any unstaged/uncomitted file changes are from a previous iteration (or if this file, probably the human author giving guidance), and can be kept if they look good. Do not ask me about them.
- Assume any connected android devices are rooted, and assume MacOS has SIP disabled.
- You don't get to declare things as fixed. Only "need human verification".

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- need human verification: Rename collision handling in project explorer now blocks rename/move when destination already exists (no overwrite).
- need human verification: Project item ordering metadata now persists across add/create/delete/move/rename/reorder operations.
- need human verification: Adding scan results from TUI now targets selected project folder; if no folder is selected, items append at root.
- need human verification: Project hierarchy now supports F2 inline rename on the selected tree row (Enter confirms, Escape cancels) without fullscreen takeover or struct viewer routing.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Added shared sort-order reconciler in `squalr-engine` project-item command executors to prevent manifest replacement and keep order entries complete.
- TUI hierarchy graph previously sorted by filesystem path; now consults project manifest sort order when available, then falls back to path order.
- Verified with `cargo check -p squalr-engine -p squalr-tui` and `cargo test -p squalr-tests --test project_items_command_tests`.
- Project hierarchy rename now renders as an inline focused row editor, clears stale rename text on cancel/submit, and keeps delete confirmation as the only fullscreen takeover.
