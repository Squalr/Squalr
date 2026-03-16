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

- Define a dedicated pointer scan session model and result types. Do not reuse flat `ScanResult` pagination for pointer chains; store pointer levels, parent-child relationships, static-vs-heap classification, and summary counts explicitly.
- Expand the pointer scan command surface to match the README algorithm. Keep `target_address`, pointer size, max depth, and offset radius, then add the follow-up request/response types needed for validation scans and lazy result expansion.
- Implement the privileged pointer scan pipeline in `squalr-engine-scanning`. Seed level 0 from the target address, walk outward level-by-level, separate static/module-backed matches from heap matches, and persist the full chain graph instead of only refreshing snapshot bytes.
- Add engine query commands for pointer scan summaries and child expansion. The GUI needs cheap summary reads plus on-demand tree expansion rather than eager materialization of every chain.
- Implement CLI pointer scan output handlers around the new query API so pointer scan commands are inspectable outside the GUI.
- Build pointer scanner view data and actions in the GUI, using the element scanner window structure as the visual baseline. The top area should own target address entry, pointer size selection, max depth, offset radius, action buttons, and scan status.
- Build a pointer results tree view in the GUI, styled consistently with existing scan panes but using project-explorer-style lazy expansion semantics. Include columns for module/base, offset chain, resolved address, depth, and static-vs-heap state.
- Add pointer result actions that matter for workflows: validate against a new target, copy/export a chain, and add the selected chain to the project as a pointer item.
- Extend `ProjectItemTypePointer` and related project/struct-view plumbing to persist an actual pointer chain instead of only a preview string, then resolve/freeze through that chain at runtime.
- Add focused tests for pointer command parsing, pointer session/result serialization, level-building logic, static/module classification, validation pruning, and project item persistence.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- `squalr/src/views/pointer_scanner/pointer_scanner_view.rs` is an empty placeholder window. The docking entry and toolbar button already exist, but there is no pointer-specific view data, controls, or results presentation yet.
- `PointerScanRequest` currently exposes only `target_address`, `pointer_data_type_ref`, `max_depth`, and `offset_size`. There is no validation-scan request, no result-query request, and no lazy-expansion command surface.
- `PointerScanResponse` currently reuses `ScanResultsMetadata`, which fits flat element scans but not pointer chains. Pointer scanning needs its own summary/query model.
- `PointerScanRequestExecutor` deanonymizes the target address, then passes the same snapshot for both statics and heaps. That is only a stub path today and does not match the README pointer-level algorithm.
- `squalr-engine-scanning/src/pointer_scans/pointer_scan_executor_task.rs` currently only recollects snapshot values. The actual pointer discovery logic is still commented out.
- `PointerScanLevel` only stores raw `SnapshotRegionFilter` vectors and currently exposes misleading `get_*` methods that consume the stored data. This type will need redesign once real pointer levels are persisted.
- CLI pointer scan handling is effectively absent: `squalr-cli/src/response_handlers/pointer_scan/mod.rs` is a no-op.
- Pointer project items are not modeled yet. `ProjectItemTypePointer` only stores a display string for preview/freeze text, while `Pointer` stores offsets as `Vec<u8>`, which is too narrow for real pointer-chain offsets.
- Existing reusable UI patterns:
  - `ElementScannerView` already provides the overall window structure for toolbar + results + footer.
  - `ProjectHierarchyViewData` already demonstrates the lazy tree expansion pattern the pointer results pane should mimic.
