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

- Need human verification: reproduce the 2026-03-15 owner report where pointer-scan logs reach `Performing pointer scan...` and memory reads finish, and confirm the GUI now repaints on its own so the session summary plus root-node population appear without requiring manual UI interaction.
- Need human verification: exercise the live GUI pointer-scanner toolbar flow after the reset/start/validate background dispatch changes plus the repaint callback change, including `New` clearing the active session, the primary action using the normal start icon for a fresh scan and switching to validation for an existing session, and refresh/fresh start/validation root population while the window stays responsive.
- Need human verification: run a large live pointer validation against a real opened process and confirm the per-step binary-search target matching still reports sensible prune counts while the GUI remains responsive and no hard UI-thread deadlock reappears.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Pointer scanning uses dedicated API/session types: `PointerScanSession`, `PointerScanNode`, `PointerScanLevel`, and `PointerScanSummary`. The active session is stored in `EnginePrivilegedState`, and the top-level command surface is `pointer-scan reset|start|summary|expand|validate`.
- `PointerScanExecutor` now walks snapshot memory with native-width aligned loads, expands parent chains within the configured radius, classifies static nodes via module lookup, and materializes root-oriented session levels/nodes. Validation rebuilds heap levels from a new target, re-resolves static nodes by `module_name + module_offset`, replaces the active session, and reports actual prune counts.
- The GUI pointer scanner now has a real window composed of a toolbar plus lazy tree results. The toolbar owns target address, pointer size, max depth, offset, a real `New` session reset action, a primary `Start/Validate` action, refresh, copy/export, add-to-project, and status text. The results pane uses project-explorer-style expansion and shows module/base, offset chain, resolved address, depth, and static-vs-heap state.
- The pointer-scanner toolbar now uses themed `DataValueBox` inputs for target / validation / depth / offset, a restricted `DataTypeSelectorView` for `u32` / `u64`, and themed action buttons. The results grid now uses column separators and mono text for address-heavy fields.
- Pointer validation no longer rescans all heap memory once per required target address. It now scans each validation step once against a sorted target-address frontier via `partition_point`, which is the current binary-search-style kernel used to avoid the previous validation hangs.
- Project-explorer address items now expose a `Pointer Scan` context-menu action that opens the pointer-scanner window and autofills the target inputs. Module-relative address items currently warn that the stored offset is not auto-resolved to a live absolute address before scan start.
- Pointer project items now persist a real chain: root `address`, `module`, `pointer_offsets` as `Vec<i64>`, `pointer_size`, symbolic struct reference, and preview display value. Pointer project item creation is supported through the unprivileged create command and writes a real `.json` project item under the hidden project root.
- Project item preview refresh now resolves stored pointer chains at runtime before reading values. Freeze activation, freeze request execution, and the periodic freeze task also resolve full pointer chains before reading or writing memory.
- Focused coverage now exists for pointer session serialization, level building, static classification, validation pruning, pointer project item persistence, pointer freeze-target creation, pointer runtime chain resolution, GUI pointer action helpers for copy/export/add-to-project, stale async pointer-scanner responses, and address-item pointer-scan context-menu extraction.
- Pointer-scanner reset now invalidates in-flight summary/start/validate revisions before late callbacks can restore a cleared session, clears the local tree immediately on `New`, and keeps `New` / `Start|Validate` / refresh actions disabled while reset is still in flight. Focused coverage now includes the reset-vs-summary race.
- Pointer-scanner toolbar defaults now use hex-oriented target / validation / offset inputs, rename `Radius` to `Offset`, default the offset to `0x800`, force foreground text styling, and slightly increase toolbar control height to reduce the cramped row layout.
- Pointer-scan start now builds a dedicated user-mode snapshot instead of depending on the shared scan snapshot, so starting a pointer scan no longer requires a prior `scan new` or populated global snapshot.
- Shared snapshot-region merge logic now lives in `squalr-engine/src/command_executors/snapshot_region_builder.rs` and is reused by both `scan new` and pointer-scan start.
- Numeric base conversions were corrected so binary / hex / address inputs produce bytes in the requested endianness, fixing pointer target parsing for hex-style addresses such as `0x3010`.
- GUI pointer-scanner summary/start/validate callbacks no longer dispatch `pointer-scan expand` inline. Root expansion is now queued in `PointerScannerViewData` and drained during the next `PointerScannerView` pass, which is the current candidate fix for the reported start-time lock hang in standalone GUI mode.
- Standalone `dispatch_privileged_command` executes work on the caller thread unless the GUI moves it elsewhere, so GUI pointer-scanner summary / reset / start / validate / expand requests now dispatch from named background threads instead of the egui thread. Focused coverage now asserts pending summary/reset/start/validate state while those requests are queued, along with the deferred root-expand path.
- The GUI pointer scanner now registers an egui repaint callback. Background summary / start / validate / reset / expand responses request a repaint immediately, so deferred root expansion no longer depends on the user manually causing the next view pass.
- Pointer-scan start and validate executors now drop the symbol-registry read guard immediately after parsing the target address, so the expensive scan / validation work no longer holds that shared lock.
- The current session builder still path-expands chains to fit the one-parent `PointerScanNode` model, so shared subchains are duplicated in-session instead of represented as a DAG.
- Owner repro note: on 2026-03-15 22:54:31 the live logs showed `Performing pointer scan...` and value collection finishing for process `20408` (`34.8MB` read) before the GUI appeared to make no obvious progress; this still needs live verification after the background-thread dispatch plus repaint callback changes.
