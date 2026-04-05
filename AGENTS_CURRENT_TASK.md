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

- Formalize the current provider-layer raw-memory fallback as a dedicated raw memory-view plugin only if it can be done without bypassing injected test providers.
- Implement the first real translated memory view behavior in the Dolphin plugin: virtual pages, modules, and translated reads/writes.


## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- The plugin contract now lives in `squalr-engine-api::plugins`; the temporary `squalr-plugin-core` crate was removed to keep plugin authoring centered on the API crate.
- Memory-view plugins are currently assumed to be built in and statically linked; third-party discovery/install is intentionally deferred.
- `EngineOsProviders` now lazily attaches a matching memory-view plugin per opened process and routes memory query/read/write calls through it, falling back to the injected base providers when no plugin matches or a plugin operation is still unimplemented.
- `cargo test -p squalr-engine-session --lib` and plugin crate tests pass after moving the plugin contract into `squalr-engine-api`.
- `cargo test -p squalr-engine --lib` is currently blocked by unrelated missing imports in the existing pointer-scan start test module, so full engine-lib validation still needs human follow-up.
- Plugin core and Dolphin plugin implementations were split out of `lib.rs` and `mod.rs` into named files so the plugin system is navigable by filename.
- `squalr-plugin-memory-view-dolphin` now has a dedicated address-space helper module that codifies the current GC/Wii guest ranges and host/guest translation math.
- The Dolphin plugin now discovers candidate MEM1 and MEM2 host regions from raw process pages, exposes them as canonical `GC`/`Wii` modules and virtual pages, and translates guest-address reads/writes back to host memory.
- Current Dolphin discovery is still heuristic: GC requires a plausible six-byte game ID header at the candidate base, while Wii extended memory is currently accepted by exact region size alone.
- `cargo test -p squalr-engine-api --lib` currently has an unrelated failing `FileSystemUtils::is_cross_platform_absolute_path` test for Unix-style paths, so the API crate target is not globally green yet.
