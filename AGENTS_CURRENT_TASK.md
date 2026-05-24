# Agentic Current Task
Our current task, from `README.md`, is:
`pr/todo`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- Completed: Project item runtime value display formats are now persisted separately from the preview value, and Project Explorer refresh/detail focus preserves the chosen format.
- Completed: Output dock command input now draws a 1px border.
- Completed: Removed the Symbol Tree per-node manifest display-format persistence path.
- Completed: Symbol layout fields can now store an optional preferred display format, exposed only when the field data type reports supported display formats.
- Completed: Symbol Tree runtime/preview values consume the layout-owned preferred display format but do not allow display-format edits from the Symbol Tree details view.

## Important Information

- Validation: `cargo fmt --all` completed with existing rustfmt deprecation warnings for `fn_args_layout`; `cargo test -p squalr-engine-api` passed 297 tests; `cargo test -p squalr` passed 29 tests; `cargo test -p squalr-engine` passed 147 tests.
- Human verification: In the Symbol Layout Editor, select a primitive field and confirm the Display Format dropdown appears with the supported dec/hex/bin/etc. options, then switch the field to a symbol layout and confirm the field is hidden. Save/reopen and confirm the Symbol Tree value/preview uses the layout-owned format while the Symbol Tree details view does not allow editing it. Also re-check project item display format persistence and the Output dock command input border.
