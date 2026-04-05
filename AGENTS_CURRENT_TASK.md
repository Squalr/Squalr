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

- Keep the provider-layer raw-memory path as the canonical engine default; do not model ordinary OS memory access as a plugin.
- If plugin extensibility needs to grow, add clearer pre/post hook seams around memory query/read/write while preserving injected base providers as the default engine behavior.


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
- The GUI now has a hidden-by-default `Plugins` dock window, exposed through the `Windows` menu, that lists plugin metadata and lets built-in plugins be enabled or disabled.
- Plugin enablement now flows through new plugin list/set-enabled privileged commands into the shared session `PluginRegistry`, so toggles affect the same registry used by routed memory-view selection.
- The Plugins window listens for `ProcessChangedEvent` and refreshes activation state against the current target process; the Dolphin plugin currently activates by matching Dolphin/Slippi process names.
- Existing saved docking layouts are patched in-memory to add the hidden `Plugins` tab beside `Settings` when that window is missing, avoiding a forced layout reset.
- The CLI now formats plugin list and set-enabled responses directly, including current target-process context plus per-plugin enablement, eligibility, activity, and metadata summaries.
- The TUI now exposes a dedicated `Plugins` pane inside the Settings workspace with keyboard navigation, enable/disable toggles, metadata summaries, and automatic process-change refreshes backed by the same plugin commands as the GUI.
- The cleaner architecture is provider-layer interception, not a `raw memory` plugin: engine consumers should call query/read/write normally, routed providers should optionally delegate to a matching memory-view hook, and plain OS access should remain the built-in default path.
- Scan settings now persist a `PageRetrievalMode`, including a new `FromVirtualModules` option, so element scans can explicitly target plugin-owned guest pages instead of always inheriting host-memory settings.
- Routed module queries now merge host modules with virtual modules additively, which keeps ordinary module-based workflows available while still surfacing guest modules like Dolphin `GC`/`Wii`.
- The GUI scan-results address column now prefers absolute guest addresses for Dolphin guest-space hits, so addresses display as `8000xxxx` / `9000xxxx` instead of `GC+offset` in the primary scan-results view.
- The default scan path now already behaves as the desired auto mode: `PageRetrievalMode::FromSettings` falls through routed memory-view queries first, so an enabled matching Dolphin plugin yields guest virtual pages without requiring the user to manually select `FromVirtualModules`.
- Guest-address formatting is now shared in the API layer and reused by project-item naming, pointer previews, CLI pointer-scan logging, and GUI pointer-scan labels so Dolphin guest roots render as absolute guest addresses consistently.
- The GUI scan settings tab now exposes a `Page source` combo box with `Auto`, host, module, and virtual-module override options; `Auto` remains the default label for `FromSettings`.
- The GUI `Plugins` window now uses the shared themed refresh icon button and checkbox controls, removes the redundant current-process banner, and renders plugins as stacked selectable rows so status and descriptions match the surrounding project/process selection UI better.
- Dolphin memory-view discovery now treats "Dolphin is running but no game memory is exposed yet" as temporary unavailability, so routed module/page queries silently fall back to base host memory instead of spamming fallback debug logs on every refresh.
- Dolphin guest-space addresses now remain resolvable as `GC`/`Wii` ranges even when no game is loaded, and routed read/write calls no longer fall through to raw host memory for those guest addresses; unavailable guest reads simply fail so existing UI can render `??` instead of logging spam or reading nonsense host addresses.
- Routed memory-view reads and writes now short-circuit on plugin-owned guest addresses and do not log routine read/write misses; non-guest addresses bypass the Dolphin translator entirely and go straight to the base OS provider, and router activation logs use `Activated` wording instead of `Attached`.
- Routed guest reads and writes now only suppress raw-provider fallback when the base process does not actually map that numeric address range, which preserves quiet `??` behavior for real unavailable guest addresses while allowing host scans in overlapping `0x80000000` Dolphin ranges to keep refreshing correctly.
