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
- need human verification: Project hierarchy now supports F2 inline rename on the selected tree row with async refresh after submit, click-away cancel/select behavior, and project selector F2 inline rename support.
- need human verification: Project item rename now keeps the tree label synced with the item `name` property instead of diverging between visible label and backing filename.
- need human verification: Project hierarchy now supports double-click name-to-rename and double-click value-to-edit with a centered `DataValueBox` takeover, explicit commit/cancel actions, and close-project cleanup for address/pointer items.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Added shared sort-order reconciler in `squalr-engine` project-item command executors to prevent manifest replacement and keep order entries complete.
- TUI hierarchy graph previously sorted by filesystem path; now consults project manifest sort order when available, then falls back to path order.
- Verified with `cargo check -p squalr-engine -p squalr-tui` and `cargo test -p squalr-tests --test project_items_command_tests`.
- Project hierarchy rename now refreshes only after the async rename response, preserves selection/expanded state across the renamed path, and cancels cleanly when another row is selected.
- Project selector now enters inline rename from F2 on the selected project and clears rename state when selection changes or rename is canceled.
- Tree rows render the project item `name` property when present; the rename executor now updates that stored name alongside the path rename so labels no longer lag behind filenames.
- Project hierarchy row double-clicks are now split by hitbox: the name region enters rename, while the trailing value region opens a takeover editor backed by the existing runtime-value write path.
- Project hierarchy refresh now drops stale rename/value/delete takeovers when the targeted project items disappear, preventing orphaned modals after closing or switching projects.
