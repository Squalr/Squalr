# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

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

- Need human verification: exercise registry snapshot bootstrap + refresh through the Android IPC worker path.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Unprivileged registry sync no longer uses a `mirror` plus a second local compatibility registry. `EngineUnprivilegedState` now keeps local project-authored symbols in project state and separately caches privileged-owned symbol metadata.
- Privileged snapshots now feed a `PrivilegedSymbolCatalog` for data-type metadata and privileged-authored symbolic structs, while project-authored symbols resolve from the opened project's local `ProjectSymbolCatalog`.
- Validation run: `cargo test -p squalr-engine-session --lib` passed after the unprivileged catalog refactor.
- Validation run: `cargo test -p squalr-tests --test registry_sync_tests` passed after the same refactor.
- Registry sync transport now uses `RegistryMetadata` and `StructLayoutDescriptor` consistently across command responses, bindings, session state, and tests; old `symbol_registry_snapshot` / `symbolic_struct_descriptor` module paths were removed.
- Dead `SymbolRegistry::get_instance()` singleton entrypoints were removed from the API and domain registries. Validation run: `cargo build -p squalr-engine --locked` passed after the rename cleanup.
