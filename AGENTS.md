# AGENTS.MD

## Workflow
- Always scan `README.md` first for the project overview + architecture constraints.
- Then read **Agentic Current Task** (below) and continue from there.
- End each session by:
  - removing unused imports / dead helpers,
  - running relevant tests,
  - checkpointing with a commit,
  - updating **Agentic Current Task** + **Concise Session Log**, compacting existing notes if needed to eliminate redundency and spam.

If you must patch source files to fix bugs while testing, that’s acceptable as long as the README architecture remains intact.

## Coding Conventions
- Variable names must be coherent and specific. `i`, `idx`, and generic `index` are forbidden. Use names like `snapshot_index`, `range_index`, `scan_result_index`, etc.
- No unhandled `unwrap()`, panics, etc. On failure: return a `Result` or log via `log!` macros.
- Comments end with a period.
- Format with default Rust formatter (repo includes `.rustfmt.toml`).
- Prefer rustdoc/intellisense-friendly function comments where practical.
- Remove unused imports.
- Prefer single-responsibility. Do not inline structs that do not belong in the file.

## Agentic Current Task
### Goal
We are on `pr/unit-tests` building `squalr-tests` as a workspace crate to test **command/response contracts** for GUI/CLI commands. The test suite is about validating **request payloads, dispatch, and typed response decoding**.

### Phase 1 (done)
Contract tests for parsing + request dispatch + typed response extraction are implemented and split into per-command suites.

### Phase 2 (in progress): OS Mock / DI seam
We need deterministic tests for privileged executors that currently call static OS singletons directly (examples: `MemoryQueryer`, `MemoryReader`, `MemoryWriter`, `ProcessQuery`). Until we have DI seams, “real OS behavior” tests can’t be correct or stable.

**Deliverable:** a minimal dependency-injection seam so tests can supply mock OS behavior.

### Current State (facts)
- `squalr-tests` exists as a workspace crate.
- Tests are split by command under `squalr-tests/tests/*_command_tests.rs`.
- Phase 1 covers all currently exposed `PrivilegedCommand` + `UnprivilegedCommand` variants with contract tests.
- `EnginePrivilegedState` now supports injected OS providers (process query + memory query/read/write), with production defaults bound to existing singletons.
- `scan_results` privileged executors (`query`, `list`, `refresh`, `freeze`, `set_property`) now route memory operations through injected OS providers.
- `MockEngineBindings` is centralized in `squalr-tests/src/mocks/mock_engine_bindings.rs` and reused by all command contract suites.
- Deterministic OS-behavior tests exist in `squalr-tests/tests/os_behavior_command_tests.rs` for memory read/write (success + failure), process list/open/close (including open-failure), scan-new page bounds merge flow, and scan-results query/list/refresh/freeze/set-property flows (success + failure paths).
- `scan_results add_to_project` remains a stubbed executor in this branch because project-item mutation hooks are not wired yet.
- Parser rejection coverage now exists per command family for malformed/incomplete arguments (missing required args and invalid value formats).
- `cargo test -p squalr-tests` is currently passing (124 integration tests; revalidated on 2026-02-08 after adding CI + failure-path/parser-rejection coverage, with singleton usage under `scan_results` limited to the intentional `add_to_project` stub path).

### If something is too hard to test
- Stub the test, and write down **why** (architecture limitation) + what would need to change.
- Keep notes short and aligned with the README architecture plan.

## Agent Scratchpad and Notes

### Current Tasklist (Remove as things are completed, add remaining tangible tasks)
- (none)

### Architecture Plan (Modify sparringly as new information is learned. Keep minimal and simple)
- Phase 1: command parsing + request dispatch + typed response decode via engine API mocks. [done]
- Phase 2: OS-behavior tests with injectable privileged OS access. [in progress]
- Implemented seam: process/memory providers attached to `EnginePrivilegedState` as trait objects, defaulting to current singleton-backed behavior in production.

### Concise Session Log (append-and-compact-only, keep very short)
- `pr/unit-tests`: Added `squalr-tests` workspace crate.
- `pr/unit-tests`: Split integration tests into per-command suites for maintainability.
- `pr/unit-tests`: Added broad parser + command/response contract coverage.
- `pr/unit-tests`: Added `EnginePrivilegedState` OS provider DI seam (process query + memory query/read/write) with production defaults.
- `pr/unit-tests`: Added canonical OS mock surface in `squalr-tests/src/mocks/mock_os.rs`.
- `pr/unit-tests`: Centralized `MockEngineBindings` in `squalr-tests/src/mocks/mock_engine_bindings.rs` and updated all command contract suites to reuse it.
- `pr/unit-tests`: Wired `scan_results` executors (`query/list/refresh/freeze/set_property`) to injected OS providers instead of static OS singletons.
- `pr/unit-tests`: Expanded deterministic OS-behavior tests (`os_behavior_command_tests`) to cover scan-results query/list/refresh/freeze provider usage.
- `pr/unit-tests`: Added deterministic `scan_results set_property` OS-behavior tests for value writes and freeze/unfreeze toggling through injected providers.
- `pr/unit-tests`: Fixed bool deanonymization for supported formats so `set_property is_frozen` decodes boolean payloads correctly.
- `pr/unit-tests`: Audited test framework on 2026-02-08 and recorded prioritized gaps in `audit.txt` (CI enforcement missing, failure-path depth gaps, `scan_results add_to_project` still stub-bound for behavior testing).
- `pr/unit-tests`: Converted 2026-02-08 `audit.txt` findings into actionable `AGENTS.MD` tasklist items focused on CI enforcement, OS failure-path depth, and parser rejection coverage.
- `pr/unit-tests`: Added CI workflow `.github/workflows/squalr-tests-pr.yml` to enforce `cargo test -p squalr-tests` for PRs targeting `pr/unit-tests` when relevant workspace paths change.
- `pr/unit-tests`: Expanded `os_behavior_command_tests` with deterministic failure-path assertions for `memory_read`, `memory_write`, `process_open`, and `scan_results` (`query/list/refresh/freeze/set_property`) using mock OS toggles.
- `pr/unit-tests`: Added parser rejection tests across all command-family suites for malformed/incomplete args and invalid value formats; revalidated `cargo test -p squalr-tests` passes with 124 integration tests on 2026-02-08.
- `pr/unit-tests`: Reconciled 2026-02-08 `audit.txt` items against current branch state; retained only remaining actionable unit-test framework follow-ups (broader CI lane + warning-signal controls) in the task list.
- `pr/unit-tests`: Added scheduled CI workflow `.github/workflows/workspace-nightly.yml` to run `cargo test --workspace` (plus manual dispatch) for broader regression coverage outside PR path filters.
- `pr/unit-tests`: Added warning-baseline controls to `.github/workflows/squalr-tests-pr.yml`; touched unit-test crates (`squalr-tests`, `squalr-engine`, `squalr-engine-api`) now compare warning counts against PR base and fail only on warning regressions.
- `pr/unit-tests`: Revalidated on 2026-02-08 that `cargo test -p squalr-tests` passes locally (124 integration tests); active DI routing remains in place and singleton usage under `scan_results add_to_project` remains intentionally stub-bound pending project-item mutation hooks.
- `pr/unit-tests`: Revalidated again on 2026-02-08 that `cargo test -p squalr-tests` passes locally (124 integration tests) with no new actionable Phase 2 tasklist items.

## Agentic Off Limits / Not ready yet
- `pr/cli-bugs`: CLI does not spawn a window / execute commands reliably; align with GUI behavior.
- `pr/error_handling`: Normalize error style (engine uses struct-based errors; cli/gui can use `anyhow!`).
- `pr/tui`: Not ready yet.
