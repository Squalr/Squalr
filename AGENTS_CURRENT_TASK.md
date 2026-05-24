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
- Completed: Symbol Tree runtime value display formats are now persisted in the project manifest by symbol node key and auto-save after format changes.
- Completed: Output dock command input now draws a 1px border.

## Important Information

- Validation: `cargo fmt --all` completed with existing rustfmt deprecation warnings for `fn_args_layout`; `cargo test -p squalr-engine-api` passed 292 tests; `cargo test -p squalr` passed 29 tests; `cargo test -p squalr-engine` passed 147 tests.
- Human verification: Update a project item value display format and a Symbol Tree value display format, then confirm the selected dec/hex/bin/etc. format stays selected after refresh/reopen. Visually confirm the Output dock command input border.
