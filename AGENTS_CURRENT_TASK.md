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
- Completed: Valueless relative scan constraints now compile into real relative constraints instead of being dropped, so Collect Values followed by Increased/Decreased filters against the collected baseline rather than retaining every result.
- Completed: Linux windowed-process detection now recognizes display-connected client sockets as a best-effort Wayland/X11 signal, so the Linux GUI process appears in `--require-windowed` process queries under WSLg.
- Completed: GUI logging now suppresses noisy dependency debug/span targets and the repeated WSLg Wayland cursor warning target while preserving Squalr root debug logging.

## Important Information

- Validation: `cargo fmt --all` completed with existing rustfmt deprecation warnings for `fn_args_layout`; `cargo test -p squalr build_scan_constraints` passed 2 targeted tests; `cargo test -p squalr` passed 31 tests. Prompt validation: `cargo test -p squalr-engine-api` passed 303 tests; `cargo test -p squalr-cli` passed 14 tests; `cargo test -p squalr output_command` passed 4 targeted tests. Scan validation: `cargo test -p squalr-engine` passed 148 tests. Linux validation: WSL `cargo test -p squalr-engine-targets-native linux_process_query -- --nocapture` passed 14 tests; `cargo test -p squalr-engine-session logging -- --nocapture` passed 1 test; WSL `cargo build -p squalr -p squalr-cli` passed; runtime WSL check launched `squalr`, verified `squalr-cli process list --require-windowed --search-name squalr` returned `is_windowed: true`, and verified zero `winit::Window::`, `Failed to set cursor`, or `tracing::span` matches in the captured GUI log.
- Human verification: Reopen the winmine project and confirm the Symbol Tree shows `PE Headers`, one undefined segment, `winmine_exe_0x579C`, and one trailing undefined segment under `winmine.exe`, matching the single layout field shown by right-click edit layout. Also confirm the previous display-format and Output dock checks still behave as expected. In the GUI Output dock, entering `24` should report `Usage: <COMMAND>` rather than `Usage: squalr-engine-api <SUBCOMMAND>`. In the scanner, Collect Values followed by Increased should only retain entries whose scan-time current value is greater than scan-time previous value. On Linux/WSL, confirm the GUI's windowed process filter includes the running Squalr process and the Output log is not flooded with winit span/cursor messages during normal idle interaction.
