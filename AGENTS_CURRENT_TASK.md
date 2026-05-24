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
- Completed: Output dock command input now draws a 1px border and uses a compact 28px height with 4px spacing above it.
- Completed: Relative element scan constraints now omit hidden value-box contents, preventing empty literal parsing after Collect Values when scanning increased/decreased/changed/unchanged.
- Completed: Removed the Symbol Tree per-node manifest display-format persistence path.
- Completed: Symbol layout fields can now store an optional preferred display format, exposed only when the field data type reports supported display formats.
- Completed: Symbol Tree runtime/preview values consume the layout-owned preferred display format but do not allow display-format edits from the Symbol Tree details view.
- Completed: Symbol Tree module-root layout presentation now consumes promoted symbol claims that correspond to module-root layout fields without appending duplicate claim nodes or inserting extra undefined segments for claim-relocated sequential layout fields. A winmine-shaped regression covers the exact `PE Headers`, undefined, `winmine_exe_0x579C`, undefined order.
- Completed: Prompt command error/help usage now strips executable/crate names for GUI and interactive CLI session mode, showing top-level prompt usage as `<COMMAND>` while preserving executable-style usage for CLI one-shot errors.

## Important Information

- Validation: `cargo fmt --all` completed with existing rustfmt deprecation warnings for `fn_args_layout`; `cargo test -p squalr build_scan_constraints` passed 2 targeted tests; `cargo test -p squalr` passed 31 tests. Prompt validation: `cargo test -p squalr-engine-api` passed 302 tests; `cargo test -p squalr-cli` passed 14 tests; `cargo test -p squalr output_command` passed 4 targeted tests. Prior broader validation: `cargo test -p squalr-engine` passed 147 tests.
- Human verification: Reopen the winmine project and confirm the Symbol Tree shows `PE Headers`, one undefined segment, `winmine_exe_0x579C`, and one trailing undefined segment under `winmine.exe`, matching the single layout field shown by right-click edit layout. Also confirm the previous display-format and Output dock checks still behave as expected. In the GUI Output dock, entering `24` should report `Usage: <COMMAND>` rather than `Usage: squalr-engine-api <SUBCOMMAND>`.
