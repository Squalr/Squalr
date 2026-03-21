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
- Need human verification: confirm the pointer-scanner toolbar no longer renders the old status text row at all, so the results panel gets that vertical space back.
- Need human verification: confirm child pointer-tree contexts now show a synthetic first `Back` row inside the results list, the footer pager no longer contains its own back/up button, and the vertical splitters stop at the footer boundary instead of extending through it.
- Need human verification: confirm the pointer-scanner toolbar now uses three bars in live use: `New | Depth | Offset | Pointer size | Data type`, then `Target/validation address | Scan | Add`, then a separate value-pointer bar with `Target/validation value | Start value scan`, with the offset editor staying decimal and the refresh button still absent.
- Need human verification: confirm value-seeded pointer scans and validation scans actually work live from the new value bar, using the selected value data type to seed all matching addresses without breaking normal address-seeded scans.
- Need human verification: confirm pointer-scanner value inputs now default to decimal formatting like regular scans, default the selected target data type to `i32`, and render their address/value field text with preview colors rather than hard-forcing `theme.foreground`.
- Need human verification: confirm the property viewer now gives runtime address/pointer `value` fields a sane numeric default format, preserves the active display format on edit commits, and exposes useful alternate preview formats such as decimal / hex / binary instead of gravitating to string.
- Need human verification: confirm right-clicking project items now uses the shared styled context-menu widget rather than egui's native popup styling, puts pointer-scan actions at the top of the menu, keeps the existing create/delete actions below them, and exposes both `Pointer Scan for Base Address` and `Pointer Scan for Resolved Address` on pointer items.
- Need human verification: confirm project-item add flows now expose `New Folder`, `New Address`, and `New Pointer` from both the toolbar add button and the project-item right-click menu, and that the new address create path actually persists a usable blank address item.
- Need human verification: confirm the project-hierarchy toolbar now keeps the delete-selected action right-aligned while the add menu remains on the left with the close-project action.
- Need human verification: confirm drag-and-drop in the project explorer now clearly distinguishes `drop into folder` versus `insert between rows`, with full-row selection-color highlighting for folder drops, stronger before/after insertion lines, and sane reorder behavior both within the same parent and when dragging nested items out into an ancestor directory.
- Need human verification: confirm the delete-confirmation take-over now centers the action row, adds spacing between `Cancel` and `Delete`, and gives `Cancel` a visible non-delete button treatment.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Owner: Removed to compact
- Pointer-scanner result rows now need width-safe clipped painting, inline disclosure inside the primary column, and branch-depth text derived from `display depth + discovery depth - 1` rather than raw discovery depth alone.
- Pointer-scanner results now follow the regular scan-results sizing model more closely: the parent content rect owns the width budget, the header/rows/footer share the same splitter positions, and each row allocates exactly the width the scroll area gives it instead of deriving column widths from clip-space.
- Pointer-scanner leaf rows must never use `allocate_rect` for the missing disclosure icon path, because that advances egui layout and makes non-expandable rows render at a different effective height than rows with children.
- Pointer-scanner toolbar status still exists in view-model state for commands/tests, but the toolbar should not render it as a third row anymore.
- Pointer-scanner child contexts now reserve row index `0` for a synthetic navigate-up row in the results list, so visible row count is `current_context_node_ids + 1` whenever `current_context_parent_node_id` is present.
- Pointer-scanner synthetic navigate-up rows should read `Return to depth n-1` using the current parent node depth, and positive child offsets should render as `0x...` without a leading `+`.
- `DataValueBoxView` should render the active display-format icon on the right side instead of a generic down-arrow, using the existing decimal/binary/hex/string icon set as the visible affordance for the popup.
- Pointer-scanner node materialization now carries an explicit `branch_total_depth` from the root static through every expanded child node, so the UI can render depth as the stable rule `root = 1 of y`, then `2 of y`, `3 of y`, etc. without trying to recompute `y` from the current context.
- Pointer-scanner toolbar state now carries a real target-value `DataTypeSelection` instead of a hidden string id, so the UI can render an actual value-type selector alongside pointer size while project-item creation still reads the selected target data type from the same shared state. Offset defaults and summary hydration are now decimal to match the requested editor format.
- Pointer-scan start/validate requests now share a `PointerScanTargetRequest`, and sessions/summaries now carry a `PointerScanTargetDescriptor` plus the resolved target-address set so address-seeded and value-seeded scans reuse the same collection, validation, and lazy-materialization pipeline.
- Address-target session materialization must keep the old tree semantics exactly; only terminal target fan-out is special for value-target sessions. The value-target resolver therefore scans on a temporary cloned snapshot so exact-value seed discovery does not mutate or shrink the live pointer-scan snapshot before the wavefront scan runs.
- Pointer-scanner value-target editors should default to unsigned decimal inputs, and the toolbar should let the shared `DataValueBoxView` default width match the regular element-scanner value box instead of forcing a wider custom width.
- Struct-viewer runtime `value` fields for address/pointer items should derive edit/display formatting from the resolved validation data type, not from the backing UTF-8 storage field, so numeric previews do not default to string semantics.
- Committing a struct-viewer `value` edit should preserve the active display format when possible and regenerate the supported display-format list from the committed `DataValue`.
- Project-hierarchy right-click menus now track explicit popup position in view-data and render through the shared `ContextMenu` + `ToolbarMenuItemView` path so they visually match the rest of the application's menus.
- Project-hierarchy pointer-scan context actions should be built as a small typed list rather than a single address tuple: address items expose one `Pointer Scan` entry, while pointer items expose `Base Address` and `Resolved Address` entries that reuse the existing pointer-resolution walk before opening the scanner.
- Project-item creation is now routed through a shared `ProjectHierarchyCreateItemKind` flow so the toolbar add button and the row context menu both dispatch the same `folder / address / pointer` create requests, and the engine create executor now supports blank address items in addition to directories and pointers.
- Project-hierarchy drag/drop now carries an explicit `into / before / after` target model so the UI can render full-row folder targets separately from horizontal insertion targets, and cross-parent before/after drops now execute as a move into the target parent followed by a sibling reorder instead of being silently suppressed.
- Project-hierarchy drag/drop hover detection should use the live pointer position against the row rectangle instead of `row_response.hover_pos()` so active drag feedback continues to render while the cursor is captured for dragging.
- Folder drop targets should render with `theme.selected_background` + `theme.selected_border`, and before/after drop targets should use a heavier horizontal line with end caps so insertions read as deliberate placement rather than a faint separator.
- The delete-confirmation take-over now uses a centered fixed-height action row with explicit inter-button spacing and secondary styling for `Cancel` instead of two flush left-aligned buttons.
