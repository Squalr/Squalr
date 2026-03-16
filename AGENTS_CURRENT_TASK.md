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

- Replace the placeholder pointer scan session returned by `PointerScanExecutor` with the real README algorithm. Level 0/session plumbing now exists, but node discovery, parent-child graph building, and static-vs-heap separation are still missing.
- Implement real pointer validation scans in the privileged engine. The `validate` command surface exists and returns session summaries, but it currently reports "not implemented" and does not prune chains.
- Build pointer scanner view data and actions in the GUI, using the element scanner window structure as the visual baseline. The top area should own target address entry, pointer size selection, max depth, offset radius, action buttons, and scan status.
- Build a pointer results tree view in the GUI, styled consistently with existing scan panes but using project-explorer-style lazy expansion semantics. Include columns for module/base, offset chain, resolved address, depth, and static-vs-heap state.
- Add pointer result actions that matter for workflows: validate against a new target, copy/export a chain, and add the selected chain to the project as a pointer item.
- Extend `ProjectItemTypePointer` and related project/struct-view plumbing to persist an actual pointer chain instead of only a preview string, then resolve/freeze through that chain at runtime.
- Add focused tests for pointer command parsing, pointer session/result serialization, level-building logic, static/module classification, validation pruning, and project item persistence.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- `squalr/src/views/pointer_scanner/pointer_scanner_view.rs` is an empty placeholder window. The docking entry and toolbar button already exist, but there is no pointer-specific view data, controls, or results presentation yet.
- Pointer scanning now has dedicated API/session types: `PointerScanSession`, `PointerScanNode`, `PointerScanLevel`, and `PointerScanSummary`. The engine stores the active session in `EnginePrivilegedState` instead of reusing flat `ScanResultsMetadata`.
- The top-level CLI/API command is now subcommand-based: `pointer-scan start|summary|expand|validate`. `start` accepts `target_address`, `pointer_size`, `max_depth`, and `offset_radius`.
- Engine query commands for pointer scan summaries and lazy child expansion now exist and read from the stored session. CLI pointer scan responses also log summaries and expanded nodes.
- `squalr-engine-scanning/src/pointer_scans/pointer_scan_executor_task.rs` still only recollects snapshot values and returns an empty placeholder session. The real pointer discovery walk is still commented out.
- The `validate` command currently only resolves/parses the new target address and reports a "not implemented" status. It does not prune or rebuild pointer levels yet.
- Pointer project items are not modeled yet. `ProjectItemTypePointer` only stores a display string for preview/freeze text, while `Pointer` stores offsets as `Vec<u8>`, which is too narrow for real pointer-chain offsets.
- Existing reusable UI patterns:
  - `ElementScannerView` already provides the overall window structure for toolbar + results + footer.
  - `ProjectHierarchyViewData` already demonstrates the lazy tree expansion pattern the pointer results pane should mimic.
- `PointerScanExecutor` now walks snapshot memory with native-width aligned loads, expands parent chains by matching pointer values within the configured radius, classifies static nodes via module lookup, and materializes root-oriented `PointerScanSession` levels/nodes instead of returning an empty placeholder session.
- The current session builder path-expands chains to fit the one-parent `PointerScanNode` model, so shared subchains are duplicated in-session rather than represented as a DAG.
