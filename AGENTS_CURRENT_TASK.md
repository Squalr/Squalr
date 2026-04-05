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

- Define the future plugin-facing registration actions for privileged-authored data types/structs now that raw symbol-registry access is no longer exposed through `RegistryContext` or `EnginePrivilegedState` public getters.
- Add unprivileged edit/save flows for project-owned user symbols so project config changes immediately resync the privileged runtime, not just open/close.
- Decide whether project item type metadata should join the snapshot in this branch or remain a later slice.
- Need human verification: exercise snapshot bootstrap + refresh through the Android IPC worker path. Owner reported the standalone GUI build works, but Android verification is deferred.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Implemented a first registry sync slice: `RegistryGetSnapshot` command, `RegistryChangedEvent`, `SymbolRegistryMirror`, and thread-safe `SymbolRegistry` snapshot import/export.
- `EngineUnprivilegedState::initialize()` now subscribes to engine event envelopes and immediately bootstraps the symbol snapshot; event dispatch waits for the symbol registry mirror to catch up to the envelope generation before listeners run.
- `EngineUnprivilegedState` now owns a local compatibility `SymbolRegistry` seeded from snapshots, so runtime metadata/formatting helpers no longer depend on the global singleton bridge.
- Privileged command transport now carries a `SymbolRegistrySnapshot` only for registry command results, and unprivileged bindings apply that snapshot before invoking existing response callbacks.
- Engine event subscriptions now flow through an `EngineEventEnvelope` carrying registry generation metadata, so standalone and IPC events share a pre-dispatch sync hook instead of exposing raw `Receiver<EngineEvent>`.
- `SymbolRegistry` now exposes metadata mutation APIs for data type descriptors and symbolic structs, and `EnginePrivilegedState` wraps them so successful mutations emit `RegistryChangedEvent` automatically once.
- `EngineUnprivilegedState` now exposes mirror-backed data-type metadata queries, and the struct viewer uses them for type-list population plus symbolic type validation instead of reading those bits directly from the singleton registry.
- `EngineUnprivilegedState` now also fronts value-formatting and parsing helpers used by the GUI/TUI struct viewers and value boxes, so `squalr/src` and `squalr-tui/src` no longer call `SymbolRegistry::get_instance()` directly.
- User-authored symbols now have a serialized `ProjectSymbolCatalog` in project info, and project open/close synchronously push that catalog into the privileged runtime through the new `RegistrySetProjectSymbols` command.
- `SymbolRegistry` now maintains a separate project-authored symbol layer so project-owned symbols can be replaced wholesale without clobbering privileged/plugin-owned registry entries.
- Added `squalr-tests/tests/registry_sync_tests.rs` covering bootstrap + event-driven refresh of the mirrored symbol registry.
- Validation run: `cargo test -p squalr-engine-domain --lib` passed (19 tests).
- Validation run: `cargo test -p squalr-tests --test registry_sync_tests` passed.
- Validation run: `cargo test -p squalr-engine --lib` passed (48 tests).
- Validation run: `cargo test -p squalr struct_viewer_view_data --lib` passed (6 targeted tests).
- Validation run: `cargo check -p squalr --lib` passed; warnings were pre-existing unused-variable/dead-field warnings in settings/app code.
- Validation run: `cargo test -p squalr-engine-domain --lib symbol_registry` passed.
- Validation run: `cargo test -p squalr-engine-projects project_info_round_trip_preserves_project_symbol_catalog` passed.
- Validation run: `cargo check -p squalr-engine` passed after the project-symbol sync slice; one local unused import warning was removed.
- Validation run: `cargo check -p squalr` passed after routing GUI struct-view/value formatting through `EngineUnprivilegedState`; only pre-existing settings/app warnings remain.
- Validation run: `cargo check -p squalr-tui` passed after routing TUI struct-view formatting/editing through `EngineUnprivilegedState`; one pre-existing dead-code warning on `TuiPane::StructViewer` remains.
- Validation run: `cargo check -p squalr-engine` passed after threading explicit `SymbolRegistry` references through snapshot-backed scan result materialization and element/pointer-scan constraint finalization.
- Latest singleton reduction slice removed direct `SymbolRegistry::get_instance()` usage from `squalr-engine-domain/src/**` and from the `scan_results` executor path by passing explicit registry references from `EnginePrivilegedState` into snapshot/result helpers.
- Latest singleton reduction slice also moved element-scan and pointer-scan constraint deanonymization/finalization off the global registry path by threading `&SymbolRegistry` through `AnonymousScanConstraint` and `ScanConstraintFinalized` in both API/domain constraint modules.
- Validation run: `cargo check -p squalr-engine` passed after threading explicit `SymbolRegistry` references through scan rule/filter helpers, snapshot aggregate counting, scan initialization, and `project_items` executors.
- Validation run: `cargo test -p squalr-engine --lib` passed (48 tests) after the same explicit-registry pass.
- Validation run: `cargo test -p squalr-tests --test os_behavior_command_tests` passed (24 tests) after updating explicit-registry constructor call sites.
- Validation run: `cargo test -p squalr-engine-api --lib` still has one unrelated pre-existing failure in `utils::file_system::file_system_utils::tests::is_cross_platform_absolute_path_detects_unix_absolute_paths` on Windows; all snapshot/scan-result tests touched by this slice passed, including the updated `ScanResult` tests.
- Latest singleton reduction slice removed direct `SymbolRegistry::get_instance()` usage from the remaining API scan rule/filter helpers, from snapshot aggregate result counting, from the runtime `project_items` executor path, and from the `ScanResult` display-value fallback by routing through explicit `&SymbolRegistry` parameters, `EngineExecutionContext` helpers, or stored display values.
- Latest compatibility-bridge slice removed binding-level snapshot application into the singleton registry from the standalone and interprocess unprivileged bindings; registry bootstrap/refresh now flows only through typed responses and `EngineUnprivilegedState` mirror updates.
- Validation run: `cargo test -p squalr-engine --lib engine_bindings::standalone::standalone_engine_api_unprivileged_bindings` passed after switching those tests to assert response-level snapshot behavior instead of singleton side effects.
- Current singleton reader count snapshot: `squalr-engine-domain/src/**` = 0 direct matches, `squalr-engine-api/src/**` = 0 direct matches, `squalr-engine/src/**` = 0 direct matches.
- Current runtime registry mutation producer audit: the only concrete producer path in `squalr-engine/src/**` is `registry_set_project_symbols_request_executor`, and it already routes through `EnginePrivilegedState::set_project_symbol_catalog()`. The remaining producer task is for future plugin-authored data type / struct producers, not for an existing bypass in the current engine crate.
- Added `EnginePrivilegedState` unit tests covering producer-side bookkeeping for `set_project_symbol_catalog()` and `register_symbol_data_type_descriptor()`, asserting generation bumps, snapshot updates, and `RegistryChangedEvent` emission.
- `cargo test -p squalr-tests registry_sync` still hits unrelated pre-existing pointer scan test compile failures in `squalr-tests/tests/scan_command_tests.rs`.
- Latest singleton cleanup slice removed the remaining runtime `SymbolRegistry::get_instance()` readers from `squalr-engine-session/src/**` and `squalr-engine-scanning/src/**`; targeted search across current runtime crates now returns 0 direct matches.
- `squalr-tests/tests/registry_sync_tests.rs` now validates mirror-backed state instead of mutating or asserting the global singleton registry.
- Validation run: `cargo test -p squalr-tests --test registry_sync_tests` passed after the session/scanner cleanup.
- Validation run: `cargo check -p squalr-engine` passed after the same cleanup.
- Validation run: `cargo test -p squalr-engine-session --lib engine_privileged_state` passed after adding producer wiring tests.
- Latest API-hardening slice removed the public symbol-registry getter from `RegistryContext`, replaced `EnginePrivilegedState::get_symbol_registry()` with `read_symbol_registry(...)`, and converted the remaining engine read paths (`memory`, `pointer_scan`, `scan`, `scan_results`) to that controlled read access.
- Validation run: `cargo check -p squalr-engine-api` passed after removing the public `RegistryContext` symbol-registry getter.
- Validation run: `cargo check -p squalr-engine-session` passed after adding `EnginePrivilegedState::read_symbol_registry(...)`.
- Validation run: `cargo check -p squalr-engine` passed after migrating the remaining executor read paths.
- Validation run: `cargo test -p squalr-engine-session --lib engine_privileged_state` passed after the API-hardening slice.
