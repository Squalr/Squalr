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

- Need human verification: run a live pointer scan with enough roots to require paging and confirm the new pointer-scanner footer pages roots locally, keeps selection on the active root page, and still feels responsive while expanding and collapsing chains.
- Need human verification: compare the pointer-scanner results pane against regular scan results and confirm the continuous full-height column separators now look correct across the header, rows, and footer at common window sizes.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Pointer scanning uses dedicated API/session types: `PointerScanSession`, `PointerScanNode`, `PointerScanLevel`, and `PointerScanSummary`. The active session is stored in `EnginePrivilegedState`, and the top-level command surface is `pointer-scan reset|start|summary|expand|validate`.
- `PointerScanExecutor` now walks snapshot memory with native-width aligned loads, classifies matches into per-level `static` and `heap` candidate sets, and advances the frontier through heaps only. `PointerScanSession` stores compact level candidates plus summary counts, and exact display rows are materialized lazily during `pointer-scan expand` instead of eagerly building a global child-graph for every discovered pointer.
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
- The pointer-scanner results pane now virtualizes tree rendering with `ScrollArea::show_rows` plus range-based row materialization in `PointerScannerViewData`, so large root sets no longer build and paint every visible chain on the first repaint. Focused coverage now includes visible-row counts and ranged row slicing.
- Pointer-scanner root pagination is currently local to the GUI: `PointerScannerViewData` slices visible roots by `ScanSettings::results_page_size`, preserves selection on the active root page, and the results pane now renders a scan-results-style footer plus continuous column separators across the full pane.
- Pointer-scanner start / validate / summary / reset now surface immediate in-flight toolbar status text, including invalid-input and dispatch-failure messages for start / validate. Focused GUI coverage now asserts the pending start / validate status text and invalid max-depth feedback.
- Pointer-scan collection no longer stores one retained node per matched target address. It now keeps one compact candidate per pointer address within each discovery depth, which matches the older range-based `Level` design more closely and avoids inflating late-scan work when many targets fall within the same offset window.
- Pointer-scan validation now replays those compact levels directly: it re-resolves stored static candidates by `module_name + module_offset`, rebuilds heap reachability one frontier at a time, and carries only rebuilt heap addresses into the next step. The validation kernel still uses sorted target-frontier `partition_point` matching rather than per-target rescans.
- Pointer-scan execution logging now reports target metadata up front, level-by-level frontier scanning progress, unique reachable-node counts, and final retained-node counts. Validation logging now reports per-level static/heap rebuild progress and periodic heap-region progress so long validations no longer go silent after value collection.
- Pointer-scan start and validate executors now drop the symbol-registry read guard immediately after parsing the target address, so the expensive scan / validation work no longer holds that shared lock.
- Owner repro note: on 2026-03-15 22:54:31 the live logs showed `Performing pointer scan...` and value collection finishing for process `20408` (`34.8MB` read) before the GUI appeared to make no obvious progress; this still needs live verification after the background-thread dispatch plus repaint callback changes.
