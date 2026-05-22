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

## Important Information

- Validation completed:
  - `cargo check -p squalr --locked`.
  - `cargo test -p squalr-engine-api command_line --locked`.
  - `cargo test -p squalr-tests --test scan_command_tests --locked`.
  - `cargo test -p squalr-tests --test scan_results_command_tests --locked`.
- Needs human verification in GUI: run text commands from the Output prompt for `scan new`, `scan element-scan ...`, and `scan_results query ...` against an opened process and confirm scanner/result panes reconcile as expected.
