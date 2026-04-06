# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- Assume any unstaged/uncomitted file changes are from a previous iteration (or if this file, probably the human author giving guidance), and can be kept if they look good. Do not ask me about them.
- Assume any connected android devices are rooted, and assume MacOS has SIP disabled.
- You don't get to declare things as fixed. Only "need human verification".

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Need human verification: exercise the plugin list surfaces in GUI, TUI, and CLI to confirm bundled capability labels read cleanly and match expected terminology.
- Need human verification: retry the reported `i24` scan repro against `winmine.exe+0579c` after the explicit 1-byte default and new GUI alignment control; engine-side exact and relative i24 scans did not reproduce the loss in deterministic tests.


## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Plugin architecture now treats an installable plugin as a package with one or more capabilities via `PluginCapability` and `PluginPackage`; singular `PluginKind` has been removed.
- Built-in plugin loading, session registry state, memory-view routing, and enable/disable side effects are now capability-driven.
- CLI/TUI/GUI plugin views now render capability lists instead of a singular kind label.
- Project plugin sync now respects the actual boolean result returned by enable/disable operations.
- Session routing tests no longer depend on live Dolphin discovery; they inject a deterministic test memory-view package instead.
- Validation run completed: `cargo check -p squalr-engine-api -p squalr-plugin-builtins -p squalr-engine-session -p squalr-engine -p squalr-cli -p squalr-tui -p squalr` and `cargo test -p squalr-plugin-builtins -p squalr-engine-session -p squalr-tui`.
- Scan settings now expose memory alignment in the GUI scan tab, and the default scan alignment is explicit `Alignment1` instead of implicit `None`.
- Added engine regression tests covering `i24` exact rescan (`3 -> 2`) and relative increased/decreased paths; both pass for 1-byte alignment, and exact rescan also passes for 4-byte alignment.
- The remaining live `i24` exact-scan mismatch was in the byte-array Boyer-Moore path, not process switching or missing vector fallback; the bad-character shift ignored the mismatch position and could skip valid `03 00 00` matches after a partial suffix match.
- Fixed the scalar and vector byte-array Boyer-Moore scanners to compute bad-character shifts from the actual mismatch index, and added focused regressions in `squalr-engine-scanning` plus the engine `i24` scan harness.
- Validation run completed: `cargo test -p squalr-engine-scanning -- --nocapture` and `cargo test -p squalr-engine i24_ -- --nocapture`.
- Element scans now log a warning whenever debug validation is enabled so doubled scan cost is explicit in logs.
- GUI and TUI plugin enablement toggles now immediately save the opened project so persisted plugin selections stay in the project config without requiring a separate manual save.
- Dock layout persistence now goes through shared `DockingManager` helpers for window visibility, selected tabs, and resize changes; closing the Plugins window or toggling it from the toolbar now survives restart.
- Scanner selection is still globally rule-driven in `RuleMapScanType`; plugin-provided data types influence compare-function availability and metadata through the symbol registry, but there is not yet a plugin hook that can override the planner before the exact non-float equality branch maps to byte-array Boyer-Moore.
- Added a data-type scan preference hook via `DataTypeScanPreference`; `RuleMapScanType` now asks the registered data type whether generic byte-array Boyer-Moore is appropriate before forcing that path.
- The 24-bit plugin data types (`u24`, `u24be`, `i24`, `i24be`) now return `PreferTypeScanner`, which keeps them on their type-owned scalar/vector compare path instead of forcing generic byte-array planning.
