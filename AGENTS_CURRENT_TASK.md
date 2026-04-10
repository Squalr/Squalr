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
- need human verification: Project hierarchy preview refresh now only requests visible/selected item previews, preserves cached preview text for off-screen rows, deduplicates duplicate address/pointer reads per refresh, and caps large fixed-array preview reads while still loading the full live value when entering value edit.
- need human verification: Project hierarchy array previews now collapse to element-style summaries like `[a, b, c, d, ...]` instead of dumping long full array strings into the tree row.
- need human verification: `VirtualSnapshots` now exist as a session-owned materialized memory-view layer in `squalr-engine-session`, and the GUI project hierarchy preview path now uses one named virtual snapshot instead of list-command-coupled preview refresh.
- need human verification: Added a dockable GUI `Memory Viewer` with results-style virtual-page navigation, sparse `??`-until-read hex/ASCII rendering, and visible-chunk refresh through a dedicated virtual snapshot.
- need human verification: Memory viewer refresh now preserves the selected page by base address instead of raw page index, prefers the first module-backed page on initial load, and labels pages as unreadable when visible chunk reads fail.
- need human verification: Project hierarchy context menus now support `Open in Memory Viewer` for address/pointer items, opening the viewer on the target address, resolving module-relative addresses, and scrolling the containing row into view.
- need human verification: Docked windows now support title-bar maximize, title-bar double-click maximize, and tab double-click maximize, transfer maximize state to the newly active sibling tab inside tab groups instead of blanking the dock, clip memory viewer row rendering to panel bounds, and visually mark maximized windows with a highlighted header plus `Fullscreen` badge.
- need human verification: Scan-result and memory-viewer add-to-project now use `module+0xOFFSET` names whenever module metadata exists instead of plain offsets.
- need human verification: Memory viewer now renders hex bytes in the theme hexadecimal color, clears drag-selection mode on mouse release, supports restored cross-row drag selection without stale follow-on drags, arrow-key cursor movement, merged per-row selection bands, a blinking hex nibble caret, forward byte-by-byte live hex writes until the end of the page, and add-to-project from the context menu.
- need human verification: Memory viewer now has a right-side interpretation panel with canonical common-add icon buttons per interpreted type, truncated array/hex previews, aligned numeric array interpretations, exact-size scalar interpretation gating, and `Ctrl+A` full-page selection; add-to-project still defaults to `u8` for one byte or `u8[n]` for multi-byte selections unless an interpretation row overrides the type.

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
- `ProjectItemsListRequest` now accepts an internal-only optional `preview_project_item_paths` filter; `None` keeps legacy full preview refresh behavior for existing callers like the TUI.
- GUI project hierarchy preview refresh now caches duplicate `(address, module, layout)` reads and duplicate pointer-chain evaluations within a refresh pass, and large fixed arrays are preview-read with a capped anonymous layout instead of the full array.
- Project-item preview formatting now prefers element-aware array truncation over raw character slicing, so both truncated reads and long full-array display strings collapse to short summaries like `[0, 1, 2, 3, ...]`.
- Added a first-pass `VirtualSnapshots` subsystem to `squalr-engine-session`; queries are declarative replacement sets, refreshes are async per snapshot, and results carry materialized address/pointer reads plus pointer-path metadata.
- GUI project hierarchy now keeps project item metadata refresh on `ProjectItemsList`, but it skips list-driven previews and instead builds consumer-specific virtual snapshot queries for visible/selected project items, then overlays preview fields from the latest snapshot generation.
- Entering project-item value edit now performs a dedicated full live memory read for the item's configured type, so preview truncation does not leak into editing; the current takeover view remains the seam for a future dedicated byte/hex editor.
- Added `MemoryQueryRequest/Response` so GUI consumers can query routed virtual memory pages and modules for the currently opened process through the existing memory-view plugin path.
- The new GUI memory viewer keeps page metadata separate from byte materialization: page lists come from `MemoryQueryRequest`, while visible hex rows issue aligned `u8[N]` virtual-snapshot chunk queries and cache materialized bytes sparsely per page.
- The memory viewer currently enumerates raw usermode committed pages, so genuinely unreadable/guard pages can still exist; the viewer now preserves selection by page base address across refresh and explicitly marks pages as unreadable when visible chunk reads fail.
- Memory viewer page stats now show the module name directly with a `(No Module)` fallback instead of prefixing labels with `Module`.
- Memory viewer initial load currently resolves to the first page containing the first reported module base address, which can yield a nonzero page index if the raw page list begins with unowned gaps.
- Memory viewer focus requests are now asynchronous and module-aware: the viewer stores a pending target, resolves module-relative offsets against its current module list after refresh, then applies a one-shot row scroll on the containing page.
- Dock maximize is transient UI state in `DockRootViewData` rather than persisted layout state; the dock root swaps normal split rendering for the maximized panel, a footer-tab double click now toggles maximize for that sibling, and when a maximized window belongs to a tab group the maximize state now follows the currently active sibling tab.
- Maximized dock windows now expose a lightweight `is_window_maximized` helper in `DockRootViewData`, and the title bar uses that to switch to a highlighted header state, render a `Fullscreen` badge, and support drag-area double-click toggling in addition to the existing maximize button/tab gesture.
- `DockedWindowView` now clips child content to the dock content rect, and the memory viewer additionally clips each painted hex row to the visible row rect to stop overflow.
- `ProjectItemsCreateRequest` address items can now optionally carry explicit `address` and `module_name` fields, and the create executor seeds those into new address project items instead of always creating zero-address placeholders.
- Project-item names created from scan results or memory-viewer addresses now force module-relative `module+0xOFFSET` text when module metadata exists, independent of the current scan-result display mode.
- Memory viewer selection is contiguous-address based within the current page; drag selection now follows the live pointer only while an explicit drag gesture is active, mouse release clears the drag-selection state, arrow keys move or extend the active cursor by byte/row, and live hex editing uses a cursor/nibble state that writes one byte at a time and keeps advancing until the current page ends.
- Memory viewer now splits its content area into the main hex grid plus a right-side interpretation panel; the panel shows truncated raw hex, UTF-8, scalar interpretations, and aligned array interpretations (`u8/i8/u16/i16/u32/i32/f32/u64/i64/f64`) when the current selection size lines up.
- Scalar interpretation rows in the memory viewer now only appear when the selection byte count exactly matches that scalar type size; larger selections only expose array interpretations that consume the entire selection.
- Memory viewer interpretation rows can now create project items as the exact displayed type by routing through an explicit `data_type_id` override; if no interpretation override is used, selection-aware add still creates `u8` for one byte or `u8[n]` for multi-byte selections.
- Memory viewer interpretation actions now use the shared `icon_handle_common_add` icon-button pattern instead of text labels, so the row label/tooltip carries the type context while matching the rest of the GUI.
- Memory viewer keyboard navigation now supports `Ctrl+A` / command-select-all for the entire current page, anchoring the selection to the full page while leaving hex editing focused at the first byte.
