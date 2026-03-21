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

- Need human verification of `cargo build --locked --release -p squalr-cli` after the CLI pointer scan summary logger update; local verification is currently blocked by unrelated `squalr-engine-domain` portable-SIMD `SupportedLaneCount` compile failures on `nightly-aarch64-apple-darwin`.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- `PointerScanSummary` no longer exposes `get_target_address()`. Callers must read `get_target_descriptor()` and handle address/value targets from `PointerScanTargetDescriptor`.
- `squalr-cli/src/response_handlers/pointer_scan/mod.rs` now logs the summary target via `PointerScanTargetDescriptor` display formatting, which preserves both direct-address and value-target scans.
- Local `cargo build --locked --release -p squalr-cli` and `cargo test --locked -p squalr-cli` both proceed past the original CLI error, then fail in `squalr-engine-domain` with widespread portable-SIMD `LaneCount<N>: SupportedLaneCount` errors on `nightly-aarch64-apple-darwin`.
