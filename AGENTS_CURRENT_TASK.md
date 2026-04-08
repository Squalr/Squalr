# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- Assume any unstaged/uncomitted file changes are from a previous iteration (or if this file, probably the human author giving guidance), and can be kept if they look good. Do not ask me about them.
- Assume any connected android devices are rooted, and assume MacOS has SIP disabled.
- You don't get to declare things as fixed. Only "need human verification".

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Need human verification: exercise element scanner array equality UX with comma-separated numeric input (for example `i32` + `1, 2, 3`) and confirm the results page now shows one array-width match instead of multiple scalar matches.
- Need human verification: add an array scan result to the project (for example `u8[3]` or `i32[2]`) and confirm the project item preserves the fixed container width for live display/edit/freeze rather than collapsing back to the scalar base type.
- Need human verification: select an address or pointer project item in the struct viewer, use the new `type` row to switch between scalar, array, fixed-array, and pointer containers, and confirm the live value editor/preview updates to the new container semantics without losing the symbolic field definition.
- Need human verification: in the element scanner, enter an array or masked pattern value under a non-`==` compare (for example `Changed` + `1, 2` or `Not Equal` + `xx D4`) and confirm the value box is flagged invalid before dispatch.
- Need human verification: exercise masked hex-pattern scans in the element scanner (for example `u8` + `hex_pattern` + `xx D4`) to confirm wildcarded prefix/suffix matches are no longer skipped in the live process path.
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
- The 24-bit plugin crate now explicitly enables `portable_simd`; the shared 24-bit vector compare helper names `std::simd::Simd` directly, so this crate currently builds on nightly-style feature gating rather than hiding that type behind another abstraction.
- The byte-array Boyer-Moore overlap path was still overshifting on some partial-suffix mismatches because it preferred any non-zero good-suffix shift over the bad-character shift; overlap-preserving scans need the smaller safe shift. `RuleMapScanType` also now keeps `NotEqual` off the equality-only byte-array scanner.
- Numeric scan inputs now infer array equality from comma-separated values, so `i32` plus `1, 2, 3` builds one 12-byte constraint instead of failing numeric parsing.
- Scan constraint building now rejects non-equality comparisons when the parsed value spans multiple elements, keeping typed-array and hex-pattern scans on the equality-only path explicitly.
- Snapshot filter collections now track logical result width separately from the base data type unit size, so array-pattern scans page/count/materialize as one container-width result and the results pane can display `i32` array payloads like `1, 2`.
- `RuleMapScanType` now chooses byte-array scanners before the small-filter scalar early return, so array rescans on narrowed filters keep array semantics instead of dropping to scalar single-element comparisons.
- The masked byte-array scanner now uses a mask-aware bad-character shift fallback instead of the exact-byte Boyer-Moore table for wildcard patterns; this fixes skipped matches like `xx D4` over `32 D4`.
- Validation run completed: `cargo test -p squalr-engine-domain primitive_data_type_numeric -- --nocapture`, `cargo test -p squalr-engine-domain scan_constraint_builder -- --nocapture`, `cargo test -p squalr-engine-api get_scan_results_page_reads_multi_element_result_payloads -- --nocapture`, `cargo test -p squalr-engine-scanning scanner_scalar_byte_array_booyer_moore_masked -- --nocapture`, and `cargo check -p squalr-engine-domain -p squalr-engine-api -p squalr-engine-scanning -p squalr-engine -p squalr`.
- Element scanner value validation is now scan-aware through the shared `ScanConstraintBuilder` path: the GUI box still does parse-only validation elsewhere, but scan entry now marks container-width values invalid unless the compare type is immediate equality.
- Validation run completed: `cargo test -p squalr-engine-session validate_scan_constraint_rejects_non_equality_array_values -- --nocapture`, `cargo test -p squalr-engine-domain scan_constraint_builder -- --nocapture`, and `cargo check -p squalr-engine-domain -p squalr-engine-session -p squalr`.
- Project-item address creation now serializes anonymous symbolic field definitions like `u8[3]` from refreshed scan-result payload widths instead of storing only the base type id.
- Anonymous symbolic field definitions now round-trip through the registry/UI path: the symbol registry can parse them on demand, fixed-array sizes contribute to struct size, struct viewer edits keep the container metadata, and data-type icon/string conversion normalizes back to the base type for display.
- Validation run completed: `cargo test -p squalr-engine build_symbolic_field_definition_string_uses_fixed_array_for_multi_element_scan_result -- --nocapture`, `cargo test -p squalr-engine-domain symbolic_field_definition -- --nocapture`, `cargo test -p squalr struct_viewer_view_data -- --nocapture`, and `cargo check -p squalr-engine-domain -p squalr-engine -p squalr`.
- Struct viewer symbolic project-item fields now use a dedicated `SymbolicFieldDefinitionSelection` model and composite editor, so the `type` row edits both the base data type and container kind explicitly instead of preserving containers implicitly on base-type changes.
- Changing the struct viewer `type` row now refreshes the cached validation, edit, and display state for dependent live-value fields immediately, so array/pointer container edits apply to the visible value box without requiring a refocus.
- Validation run completed: `cargo test -p squalr ui::widgets::controls::symbolic_field_definition_selector -- --nocapture`, `cargo test -p squalr struct_viewer_view_data -- --nocapture`, `cargo test -p squalr-engine-domain symbolic_field_definition -- --nocapture`, and `cargo check -p squalr --quiet`.
