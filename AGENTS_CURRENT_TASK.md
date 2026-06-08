# Agentic Current Task
Our current task, from `README.md`, is:
`pr/android-dbg`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.

## Current Tasklist
- Android native debugger find-what-reads/writes/accesses is still pending; Linux native debugger watchpoints are now thread-aware and need human verification in the GUI workflow.

## Important Information
- Linux native debugger sessions now enumerate `/proc/<pid>/task`, attach traced TIDs only while the debugger is active, and mirror hardware data breakpoints across stopped threads.
- While running, the Linux backend periodically syncs newly-created threads and polls traced thread events with `waitpid` using the Linux all-traced-threads wait flag.
- `native_debugger_smoke` now validates Linux x86_64 watchpoints through a worker-thread write to a static `AtomicU64`; the Linux child main thread only idles, so the smoke requires cross-thread watchpoint coverage.
- WSL validation passed: `cargo run -p squalr-plugin-debuggers-native --example native_debugger_smoke --locked`, `cargo test -p squalr-plugin-debuggers-native --locked`, and `cargo build -p squalr --locked`.
- Windows regression validation passed: `cargo check -p squalr --locked` and `cargo test -p squalr-tests --locked`.
