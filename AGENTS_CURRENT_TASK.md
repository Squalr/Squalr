# Agentic Current Task
Our current task, from `README.md`, is:
`pr/todo`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- Continue target/scanning boundary cleanup after the target/native crate split.

## Important Information

- Current target-abstraction refactor pass split target provider traits and shared process-query types into `squalr-engine-targets`, renamed the native platform backend crate to `squalr-engine-targets-native`, and updated the Dolphin memory-view plugin/tests/session routing to depend on the new target/native crate boundary. `squalr-engine-session` still owns memory-view routing around those providers. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo check -p squalr-cli --locked`, `cargo check -p squalr-tui --locked`, `cargo test -p squalr-engine-targets --locked`, `cargo test -p squalr-engine-targets-native --locked`, `cargo test -p squalr-engine-session --locked`, `cargo test -p squalr-plugin-memory-view-dolphin --locked`, `cargo test -p squalr-tests --test os_behavior_command_tests --locked`, `cargo test -p squalr-tests --test process_command_tests --locked`, `cargo test -p squalr-tests --test memory_command_tests --locked`, `cargo test -p squalr-engine process --lib --locked`, and legacy-native-crate-name search. Needs human verification for native process open/list, memory read/write/query, and Dolphin memory-view target routing.
