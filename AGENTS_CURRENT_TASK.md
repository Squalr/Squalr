# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/error_handling`

### Architecture Plan
Modify sparringly as new information is learned. Keep minimal and simple. The goal is to always have the architecture in mind while working on a task, as not to go adrift into minefields. The editable area is below:

----------------------

## Current Tasklist (Remove as things are completed, add remaining tangible tasks)
(If no tasks are listed here, audit the current task and any relevant test cases)


Note from owner: Panic is actually acceptable in some situations! Let us revert the use of result in these cases:
- [x] Restore fail-fast startup semantics in privileged engine initialization by returning a typed init `Result` when critical boot steps fail (`engine bindings init`, `process monitor start`) instead of logging and continuing.
- [x] Restore fail-fast startup semantics for IPC host initialization by propagating privileged CLI spawn/pipe bind failures out of `InterprocessEngineApiUnprivilegedBindings::new` and `SqualrEngine::new` instead of background-thread logging.
- [x] Enforce startup invariants in standalone bindings (`StandaloneEngineApiUnprivilegedBindings::new`) by replacing "log and continue with `None`" behavior with hard invariant failure (`panic!`/`expect`) for impossible states.
- [x] Decide Android fatal-startup policy and implement consistently: either panic on unrecoverable `android_main` init failures (preferred for critical bootstrap) or return explicit startup status to caller with centralized fatal handler.
- [x] Add regression tests for startup failure behavior (engine init + IPC init) to ensure critical-system bootstrap failures are fail-fast and no longer degrade silently.
- [x] Note from owner: Installer main.rs is still logging errors instead of panic.
- [x] Note from onwer: I see uses of eprintln instead of logging in 3 instances (2 in installer, 1 in project_manager.rs). Fix.

## Important Information
Important information discovered during work about the current state of the task should be appended here.

Initial analysis
- Audit baseline (non-test runtime crates):
  - `unwrap()`: 8 original occurrences.
  - `panic!()`: 6 original occurrences.
  - `Result<_, String>`: widespread, concentrated in process query, engine bindings (interprocess/standalone), scan memory reader, and selected API payloads.
- Existing typed error foundation already present:
  - `squalr-engine-api/src/conversions/conversion_error.rs`
  - `squalr-engine-api/src/structures/data_types/data_type_error.rs`
- Architectural constraint from `README.md` task definition:
  - Engine should normalize toward struct/typed errors.
  - CLI/GUI may use `anyhow`.
  - `Result<(_), String>` is explicitly called out as bad practice.

Discovered during iteration:
- CLI, TUI, and desktop GUI entrypoints now return `anyhow::Result<()>` and no longer panic during startup/event-loop failures.
- Android startup path no longer panics on engine/gui initialization; it logs and returns from `android_main`.
- Runtime unwrap removals completed in `trackable_task`, `snapshot_region_scan_results`, and Android package cache read path.
- Process query path now uses `ProcessQueryError` across `squalr-engine-processes` + `squalr-engine` OS provider boundary (including Windows/Linux/macOS/Android implementations and `squalr-tests` mock providers), replacing prior `Result<_, String>` signatures.
- Added focused unit tests for process-query typed error formatting/constructor behavior in `squalr-engine-processes/src/process_query/process_query_error.rs`.
- Engine binding traits now use typed `EngineBindingError` instead of `Result<_, String>`, with interprocess pipe-specific `InterprocessPipeError` preserving operation context + source error chaining.
- Added focused unit tests for `EngineBindingError` constructor/display behavior in `squalr-engine-api/src/engine/engine_binding_error.rs`.
- Snapshot region memory reads now return typed `SnapshotRegionMemoryReadError` values (including chunk-first-failure context) while preserving tombstone behavior for failed read addresses.
- Settings list response payloads (`general/memory/scan`) now use typed serializable `SettingsError` instead of `String`, and engine list executors now emit scope-specific typed read failures.
- Added focused unit tests for `SnapshotRegionMemoryReadError` and `SettingsError`, and updated settings command tests to exercise typed settings-list error payloads end-to-end.
- Eliminated remaining non-test `Result<_, String>` signatures by introducing `SymbolRegistryError` + `ValuedStructError`, and removed stale commented UI `unwrap()` usage from struct viewer row rendering.
- Post-migration audit confirmed no non-test `unwrap()`, no non-test `panic!`, and no `Result<_, String>` signatures remain in Rust source.
- Verification run completed successfully for `squalr-engine-processes`, `squalr-engine-api`, `squalr-engine-scanning`, and `squalr-tests`.
- Follow-up audit found over-correction: several startup-critical failures now only log and continue, creating partially initialized runtimes.
- Primary locations requiring fail-fast handling:
  - `squalr-engine/src/engine_privileged_state.rs` (`initialize` failures at lines `87`, `101`, `115` currently log-only).
  - `squalr-engine/src/engine_bindings/interprocess/interprocess_engine_api_unprivileged_bindings.rs` (IPC bootstrap failures at lines `116`, `120` currently log-only in worker thread).
  - `squalr-engine/src/engine_bindings/standalone/standalone_engine_api_unprivileged_bindings.rs` (missing privileged state at line `27` currently logs and degrades).
  - `squalr-android/src/lib.rs` (`android_main` fatal startup errors at lines `16`, `24`, `33`, `45` currently log and return).
- Non-startup command/runtime paths are mostly correct to keep as recoverable `Result` + logging; the regression scope is concentrated in bootstrapping critical systems.
- Restored fail-fast startup behavior with typed `EngineInitializationError` in `squalr-engine`, including privileged binding init, process monitor startup, and unprivileged-host IPC bootstrap errors.
- `InterprocessEngineApiUnprivilegedBindings::new` now initializes synchronously and returns startup errors directly; `SqualrEngine::new` now propagates those failures.
- Standalone unprivileged bindings now require an `Arc<EnginePrivilegedState>` directly, enforcing startup invariant at construction.
- Android startup policy now uses fatal panic-on-bootstrap-failure semantics in `android_main`.
- Added fail-fast regression tests in `squalr-engine` for privileged startup monitor failure and IPC spawn/bind startup failures.
- Follow-up verification found one integration-test compile regression after startup API hardening: `squalr-tests/tests/os_behavior_command_tests.rs` assumed direct `Arc<EnginePrivilegedState>` return from `new_with_os_providers`; fixed helper to handle the typed startup `Result` explicitly.
- Installer startup now fails fast in `squalr-installer/src/main.rs`: logger init failure and `eframe::run_native` startup failure both panic with explicit fatal messages.
- Removed all current Rust `eprintln!` usage by converting project watcher stderr output to `log::error!` and using panic for installer fatal bootstrap errors.

## Agent Scratchpad and Notes 
Append below and compact regularly to relevant recent, keep under ~20 lines and discard useless information as it grows.
- Prioritize replacing error signatures at trait boundaries first (`ProcessQueryer`, `ProcessQueryProvider`, engine bindings), then cascade call sites.
- Keep serialized command responses backward-compatible where needed; if shape changes are required, update tests in `squalr-tests`.
- Non-test panic/unwrap cleanup should be done before deep refactors so runtime behavior is safer during migration.
- Added `thiserror` to `squalr-engine-processes` and centralized process query failures under `process_query_error.rs`.

### Concise Session Log
- Audited repository for runtime error-handling hotspots (`unwrap`, `panic`, `Result<_, String>`, existing typed errors).
- Set current task to `pr/error_handling` and created a concrete implementation tasklist for next session.
- Replaced runtime panic/unwrap hotspots in the initial target files and added `anyhow` to CLI/TUI/GUI crates.
- Ran `cargo fmt`, `cargo check -p squalr-cli`, `cargo check -p squalr-tui`, `cargo check -p squalr-engine-api`, and `cargo check -p squalr`.
- Replaced process query + OS provider `Result<_, String>` boundaries with typed `ProcessQueryError` and updated test mocks to match.
- Ran `cargo fmt`, `cargo test -p squalr-engine-processes`, and `cargo check -p squalr-engine -p squalr-tests`.
- Replaced `Result<_, String>` in interprocess/standalone engine bindings with typed errors (`EngineBindingError`, `InterprocessPipeError`) and propagated signatures through engine-api + tests.
- Ran `cargo fmt`, `cargo check -p squalr-engine-api`, `cargo check -p squalr-engine`, `cargo check -p squalr-tests`, `cargo test -p squalr-engine-api engine_binding_error`, and `cargo test -p squalr-tests`.
- Replaced snapshot region memory reader `Result<_, String>` signatures with typed `SnapshotRegionMemoryReadError` and added structured failure propagation.
- Replaced settings list response payloads from `Result<T, String>` to `Result<T, SettingsError>` and updated command executors/tests.
- Ran `cargo fmt`, `cargo test -p squalr-engine-scanning snapshot_region_memory_read_error`, `cargo test -p squalr-engine-api settings_error`, `cargo check -p squalr-engine -p squalr-cli -p squalr-engine-scanning`, and `cargo test -p squalr-tests --test settings_command_tests`.
- Added `SymbolRegistryError` and `ValuedStructError`, migrated remaining typed-error boundaries, and ran `cargo fmt`, `cargo test -p squalr-engine-api symbol_registry_error`, `cargo test -p squalr-engine-api valued_struct_error`, `cargo check -p squalr-engine-api`, `cargo check -p squalr-engine`, and `cargo check -p squalr`.
- Audited for regressions with `rg` checks over Rust sources: no non-test runtime `unwrap!`/`panic!` and no `Result<_, String>` signatures found.
- Ran `cargo test -p squalr-engine-processes`, `cargo test -p squalr-engine-api`, `cargo test -p squalr-engine-scanning`, and `cargo test -p squalr-tests` (all passing).
- Re-ran branch validation audit: `rg` checks confirmed no non-test `unwrap()` usage and only test-only `panic!`; no `Result<_, String>` signatures detected in current Rust sources.
- Re-ran relevant verification suites: `cargo test -p squalr-engine-processes`, `cargo test -p squalr-engine-api`, `cargo test -p squalr-engine-scanning`, and `cargo test -p squalr-tests` (all passing; existing unrelated warning-only diagnostics remain).
- Audited startup/initialization paths for over-zealous panic removal and identified critical bootstrap sites that currently log-and-continue instead of failing fast.
- Updated tasklist with concrete follow-up fixes focused on engine/bootstrap invariant enforcement and startup failure regression tests.
- Restored fail-fast bootstrap flow across privileged engine startup, IPC unprivileged host startup, standalone invariant construction, and Android fatal startup semantics.
- Added `EngineInitializationError`, refactored startup constructors to return typed errors, and added regression tests under `squalr-engine`.
- Ran `cargo fmt`, `cargo test -p squalr-engine`, `cargo check -p squalr-cli`, `cargo check -p squalr`, and `cargo check -p squalr-tests`.
- Fixed `squalr-tests` integration helper (`create_test_state`) to unwrap `EnginePrivilegedState::new_with_os_providers` via explicit `match` + failure panic message, aligning test expectations with typed startup failures.
- Ran `cargo test -p squalr-tests` (passing).
- Re-validated `pr/error_handling` checkpoint with `cargo test -p squalr-engine` and `cargo test -p squalr-tests`; both passed and startup fail-fast regression tests remain green.
- Re-validated `pr/error_handling` on 2026-02-08 with `cargo test -p squalr-engine` and `cargo test -p squalr-tests`; both passed, with only pre-existing warning-only diagnostics.
- Performed targeted unused-code cleanup in pointer scan scaffolding and element scan dispatcher (removed dead imports + unused locals) without behavioral changes.
- Ran `cargo fmt`, `cargo test -p squalr-engine`, and `cargo test -p squalr-tests` on 2026-02-08; all tests passed and warning count decreased slightly.
- Re-ran `pr/error_handling` audit on 2026-02-08: `rg` checks found no non-test `unwrap()` and no `Result<_, String>` signatures; startup-critical Android `panic!` usage remains intentional by policy.
- Re-ran `cargo test -p squalr-engine` and `cargo test -p squalr-tests` on 2026-02-08; all tests passed, with only pre-existing warning-only diagnostics.
- Re-validated on 2026-02-08: targeted `rg` audit still shows no Rust `Result<_, String>` and no non-test `unwrap()` usage; `panic!` usage remains limited to tests plus intentional Android fatal-startup policy.
- Re-ran `cargo test -p squalr-engine` and `cargo test -p squalr-tests` on 2026-02-08; both passed again with only existing warning-only diagnostics.
- Completed owner follow-up fixes on 2026-02-08: installer fatal startup paths now panic (no log-and-continue), `project_manager` watch failures use `log::error!`, and `rg -n 'eprintln!' -g '*.rs'` returns no matches.
- Ran `cargo fmt`, `cargo check -p squalr-installer`, and `cargo test -p squalr-engine-api` on 2026-02-08 (passing; existing warning-only diagnostics unchanged).
- Re-validated on 2026-02-08 with startup/error-handling guardrail audit: `rg -n "Result<[^\\n>]*,\\s*String>" --glob "*.rs"` and `rg -n "eprintln!" --glob "*.rs"` returned no matches; `panic!` remains confined to tests plus intentional Android/installer fatal-startup paths.
- Re-ran `cargo test -p squalr-engine`, `cargo test -p squalr-tests`, and `cargo check -p squalr-installer` on 2026-02-08; all passed with only existing warning-only diagnostics.
