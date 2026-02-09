# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/scan-commands`

### Architecture Plan
Modify sparingly as new information is learned. Keep minimal and simple.
The goal is to keep the architecture in mind and not drift into minefields.

----------------------

- Split scan command families at the privileged command root:
  - `Scan` => element scan lifecycle only (`new` / `reset` / `collect-values` / `element-scan`).
  - `PointerScan` => pointer scan command family.
  - `StructScan` => struct scan command family.
- CLI naming rule:
  - `sscan` is alias-only shorthand for CLI input.
  - Internal naming stays explicit everywhere (`PointerScan`, `StructScan`, `pointer_scan`, `struct_scan`).
- Keep command/response architecture symmetric (request -> top-level privileged command variant -> top-level privileged response variant).
- Preserve existing executor behavior first; this branch is command organization, not scan algorithm changes.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks.)

- [x] Introduce top-level command groups in `squalr-engine-api`:
  - Add explicit modules for pointer/struct scan command groups (no internal `pscan` / `sscan` module names, these keywords are reserved as cli shorthand):
    - `commands/pointer_scan/*`.
    - `commands/struct_scan/*`.
  - Remove pointer/struct variants from `commands/scan/scan_command.rs` and `commands/scan/scan_response.rs`.
  - Update `commands/privileged_command.rs` aliases:
    - `Scan` remains `scan`/`s` for element scan.
    - Add top-level `PointerScan` command with canonical command spelling `pointer-scan` and alias `pscan`.
    - Add top-level `StructScan` command with canonical command spelling `struct-scan` and alias `sscan`.
  - Update `commands/privileged_command_response.rs` with matching response variants.
- [x] Retarget request/response mappings:
  - `PointerScanRequest` and `StructScanRequest` map to new top-level privileged command/response variants.
  - `PointerScanResponse` and `StructScanResponse` implement `TypedPrivilegedCommandResponse` against their new response envelopes.
  - Keep `ScanNewRequest`, `ScanResetRequest`, `ScanCollectValuesRequest`, and `ElementScanRequest` under `scan`.
- [x] Update engine command execution routing in `squalr-engine`:
  - Add executor modules for explicit command groups (`command_executors/pointer_scan`, `command_executors/struct_scan`) or equivalent routing split.
  - Update `command_executors/privileged_command_executor.rs` match arms to dispatch new top-level command variants.
  - Keep existing request executors for pointer/struct behavior unchanged.
- [x] Update CLI response handling:
  - Route new privileged response variants in `squalr-cli/src/response_handlers/mod.rs`.
  - Add dedicated pointer/struct response handlers or wire through existing execute handler explicitly.
- [x] Update tests in `squalr-tests`:
  - Rewrite parser assertions in `tests/scan_command_tests.rs`:
    - `scan` no longer accepts `pointer-scan` or `struct-scan`.
    - Add parser coverage for top-level `pointer-scan` / `struct-scan`.
    - Add alias coverage for `pscan` / `sscan` CLI shorthand.
  - Update dispatch assertions from `PrivilegedCommand::Scan(ScanCommand::PointerScan|StructScan)` to new top-level command variants.
  - Keep OS behavior tests unchanged unless compile fallout requires import path updates.
- [x] Validation pass:
  - Run focused tests first: `cargo test -p squalr-tests scan_command_tests`.
  - Then run broader regression: `cargo test -p squalr-tests`.
  - Fix unused imports/warnings introduced by module split.

## Important Information
Append important discoveries. Compact regularly.

Information found in initial audit:
- Current architecture has pointer and struct scans nested under `scan` command group:
  - API: `squalr-engine-api/src/commands/scan/scan_command.rs`.
  - Engine dispatch: `squalr-engine/src/command_executors/scan/scan_command_executor.rs`.
  - CLI response handling: `squalr-cli/src/response_handlers/scan/mod.rs`.
- README task definition explicitly requests decoupled namespaces:
  - `scan`, `pointer-scan`/`pscan`, and `struct-scan`/`sscan` at top command level.
- There is no existing local `pr/scan-commands` branch; work will start from `main`.
- Pointer scan executor is functional; struct scan executor is currently a metadata stub:
  - `squalr-engine/src/command_executors/scan/pointer_scan/pointer_scan_request_executor.rs`.
  - `squalr-engine/src/command_executors/scan/struct_scan/struct_scan_request_executor.rs`.
- GUI currently uses only element scan commands; pointer scanner UI is scaffolded but not wired:
  - `squalr/src/views/element_scanner/scanner/view_data/element_scanner_view_data.rs`.
  - `squalr/src/views/pointer_scanner/pointer_scanner_view.rs`.
- Test blast radius is concentrated in `squalr-tests/tests/scan_command_tests.rs` (parser and dispatch expectations).

Information discovered during iteration:
- `PrivilegedCommandResponse` currently wraps all scan families into a single `Scan(ScanResponse)` variant, so response envelope split is required for clean namespace separation.
- Direct blast radius confirmed across API/engine/CLI/tests is concentrated in scan command plumbing (11 directly-referencing files before module wiring updates), with parser + dispatch assertions primarily in `squalr-tests/tests/scan_command_tests.rs`.
- Naming correction applied to plan: `sscan` is treated only as CLI shorthand alias, never as internal module/type naming.
- Implemented split complete:
  - Added top-level API modules `commands/pointer_scan/*` and `commands/struct_scan/*`.
  - Removed nested `scan/pointer_scan/*` and `scan/struct_scan/*`.
  - `ScanCommand`/`ScanResponse` are now element-scan lifecycle only.
  - `PrivilegedCommand`/`PrivilegedCommandResponse` now expose `PointerScan` and `StructScan` top-level variants.
- Parser/test detail: `cargo test -p squalr-tests scan_command_tests` filters by test name and can run zero tests; `cargo test -p squalr-tests --test scan_command_tests` was run to execute the file directly.

## Agent Scratchpad and Notes
Append below and compact regularly to relevant recent notes, keep under ~20 lines.

- Most risky compile churn is cross-crate import paths and enum match exhaustiveness.
- Lowest-risk sequence: API enums first -> request/response conversion -> engine dispatch -> CLI response wiring -> tests.
- Prefer canonical command names in tests/docs (`pointer-scan`, `struct-scan`) and keep shorthand aliases as additive parser coverage.

### Concise Session Log
Append logs for each session here. Compact redundancy occasionally.
- Read `README.md`, `AGENTS.md`, and audited scan command architecture across API, engine, CLI, GUI call sites, and tests; drafted implementation/validation plan for `pr/scan-commands`.
- Re-audited concrete scan command touchpoints and corrected plan naming so `sscan` remains CLI alias-only while internal naming stays `StructScan` / `PointerScan`.
- Implemented command family split across API, engine, CLI, and tests; added top-level pointer/struct command and response envelopes with explicit routing.
- Updated parser/dispatch tests for top-level `pointer-scan` and `struct-scan`, added `pscan`/`sscan` alias coverage, and added rejection coverage for nested `scan pointer-scan` / `scan struct-scan`.
- Validation completed: `cargo fmt`, `cargo test -p squalr-tests scan_command_tests`, `cargo test -p squalr-tests --test scan_command_tests`, and `cargo test -p squalr-tests` all passed.
