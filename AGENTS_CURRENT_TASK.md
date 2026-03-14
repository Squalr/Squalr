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

- Need human verification that removing the dead `squalr-engine-api/src/structures/data_types` source tree does not affect any external tooling or docs that referenced files outside the Cargo module graph.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- `squalr-engine-api/src/structures/data_types/mod.rs` is the live API surface and already re-exports `squalr_engine_domain::structures::data_types::*`; the deleted API-side `data_types` implementation files were orphaned and not part of the compiled module graph.
- Scan consumers still build against the domain-backed data type implementations. Verified with `cargo test -p squalr-engine-scanning --all-targets` and `cargo check -p squalr-engine --all-targets`.
- `cargo test -p squalr-engine-api -p squalr-engine-scanning --all-targets` still reports an unrelated existing failure on Windows in `squalr-engine-api/src/utils/file_system/file_system_utils.rs` for `/tmp/test` absolute-path detection.
