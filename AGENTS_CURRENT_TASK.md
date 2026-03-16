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

- Need human verification: exercise the live pointer-scan workflow against a real opened process in the GUI, including start, lazy expansion, validation, copy/export, add-to-project, project preview refresh, and freezing through the persisted pointer chain.
- Need human verification: confirm the pointer-scanner window defaults and styling in the live GUI, including hex-first target/validation/offset inputs, `Offset` labeling with `0x800` default, foreground text, and the taller toolbar control sizing.
- Need human verification: verify the project-explorer `Pointer Scan` context-menu entry on address items, including the themed toolbar autofill path and the current module-relative warning behavior.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Pointer scanning uses dedicated API/session types: `PointerScanSession`, `PointerScanNode`, `PointerScanLevel`, and `PointerScanSummary`. The active session is stored in `EnginePrivilegedState`, and the top-level command surface is `pointer-scan start|summary|expand|validate`.
- `PointerScanExecutor` now walks snapshot memory with native-width aligned loads, expands parent chains within the configured radius, classifies static nodes via module lookup, and materializes root-oriented session levels/nodes. Validation rebuilds heap levels from a new target, re-resolves static nodes by `module_name + module_offset`, replaces the active session, and reports actual prune counts.
- The GUI pointer scanner now has a real window composed of a toolbar plus lazy tree results. The toolbar owns target address, pointer size, max depth, offset, start/refresh/validate actions, copy/export, add-to-project, and status text. The results pane uses project-explorer-style expansion and shows module/base, offset chain, resolved address, depth, and static-vs-heap state.
- The pointer-scanner toolbar now uses themed `DataValueBox` inputs for target / validation / depth / offset, a restricted `DataTypeSelectorView` for `u32` / `u64`, and themed action buttons. The results grid now uses column separators and mono text for address-heavy fields.
- Project-explorer address items now expose a `Pointer Scan` context-menu action that opens the pointer-scanner window and autofills the target inputs. Module-relative address items currently warn that the stored offset is not auto-resolved to a live absolute address before scan start.
- Pointer project items now persist a real chain: root `address`, `module`, `pointer_offsets` as `Vec<i64>`, `pointer_size`, symbolic struct reference, and preview display value. Pointer project item creation is supported through the unprivileged create command and writes a real `.json` project item under the hidden project root.
- Project item preview refresh now resolves stored pointer chains at runtime before reading values. Freeze activation, freeze request execution, and the periodic freeze task also resolve full pointer chains before reading or writing memory.
- Focused coverage now exists for pointer session serialization, level building, static classification, validation pruning, pointer project item persistence, pointer freeze-target creation, pointer runtime chain resolution, GUI pointer action helpers for copy/export/add-to-project, stale async pointer-scanner responses, and address-item pointer-scan context-menu extraction.
- Pointer-scanner toolbar defaults now use hex-oriented target / validation / offset inputs, rename `Radius` to `Offset`, default the offset to `0x800`, force foreground text styling, and slightly increase toolbar control height to reduce the cramped row layout.
- Pointer-scan start now builds a dedicated user-mode snapshot instead of depending on the shared scan snapshot, so starting a pointer scan no longer requires a prior `scan new` or populated global snapshot.
- Shared snapshot-region merge logic now lives in `squalr-engine/src/command_executors/snapshot_region_builder.rs` and is reused by both `scan new` and pointer-scan start.
- Numeric base conversions were corrected so binary / hex / address inputs produce bytes in the requested endianness, fixing pointer target parsing for hex-style addresses such as `0x3010`.
- The current session builder still path-expands chains to fit the one-parent `PointerScanNode` model, so shared subchains are duplicated in-session instead of represented as a DAG.
