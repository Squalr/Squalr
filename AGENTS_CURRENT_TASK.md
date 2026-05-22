# Agentic Current Task
Our current task, from `README.md`, is:
`Command/event hooks and command invocation response observation.`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- [x] Add shared `CommandInvocation` naming/types for command source, command payload, response payload, middleware decisions, and outcomes.
- [x] Route `EngineExecutionContext` typed request dispatch through `EngineUnprivilegedState` so normal `.send(...)` requests publish invocation outcomes.
- [x] Add unprivileged session hooks for command invocation middleware, response middleware, and response listeners.
- [x] Mark Output prompt commands with `CommandInvocationSource::Prompt`.
- [x] Have element scanner results observe scan and scan-results command responses so text commands can update GUI scan/result state instead of dropping responses.
- [x] Add coverage for command outcome publication and invocation rejection middleware.
- [x] Audit currently exposed command families against GUI state observers.
- [x] Add GUI response observers for current high-value gaps:
  - process `list`/`open`/`close` responses update the process selector,
  - plugin `list`/`set-enabled` responses update the plugin window,
  - general/memory/scan settings `list`/`set` responses update settings tabs,
  - pointer-scan `summary`/`start`/`validate`/`expand`/`reset` responses update the pointer scanner,
  - scan-results `refresh`/`freeze`/`set-property`/`delete`/`list` responses reconcile the element scanner results view.
- [x] Add plugin priority ordering:
  - registry iteration now follows project/session plugin priority order,
  - plugin priority can be changed through inline row up/down buttons and a right-click context menu in the plugin window,
  - `plugins set-order` command and responses propagate ordering through GUI/CLI/TUI paths,
  - project plugin configuration now persists enablement plus priority order under the existing `plugins` project field.

## Important Information

- Validation completed:
  - `cargo check -p squalr --locked`.
  - `cargo check -p squalr-cli --locked`.
  - `cargo check -p squalr-tui --locked`.
  - `cargo test -p squalr-engine-api command_line --locked`.
  - `cargo test -p squalr-engine-api plugin_configuration --locked`.
  - `cargo test -p squalr-engine-session plugin_registry --locked`.
  - `cargo test -p squalr-engine-projects project_info_round_trip_preserves_plugin_configuration --locked`.
  - `cargo test -p squalr-tests --test scan_command_tests --locked`.
  - `cargo test -p squalr-tests --test scan_results_command_tests --locked`.
- Current audit notes:
  - Already covered by engine events: process open/close process-change handling, plugin enablement change events, project catalog/item/close refresh events, and scan-result update events.
  - Response-only before this pass: `process list`, `plugins list`, settings `list`/`set`, pointer-scan commands, and several scan-results mutations.
  - Still intentionally output/side-effect oriented unless a concrete GUI owner exists: raw `memory read/query/write/freeze`, `registry get/set-project-symbols`, `struct_scan`, and `trackable_tasks`.
- Needs human verification in GUI: run text commands from the Output prompt for `process list`, `plugins list`, `settings scan set ...`, `pointer_scan summary/start/expand`, `scan new`, `scan element-scan ...`, and `scan_results query/freeze/set-property/delete ...` against an opened process and confirm panes reconcile as expected.
- Needs human verification in GUI: use the inline plugin row up/down buttons and right-click `Increase priority` / `Decrease priority`, reopen/save projects, and confirm priority order affects plugin selection/action order as expected.
- Needs human verification in GUI: hover inline plugin priority buttons and confirm their tooltips use the Squalr tooltip style with normal delayed tooltip behavior.
