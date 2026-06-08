# Agentic Current Task
Our current task, from `README.md`, is:
`pr/android-dbg`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.

## Current Tasklist
- Android native debugger find-what-reads/writes/accesses has an arm64 ptrace backend path and passed release native smoke on an authorized/rooted device; GUI workflow still needs human verification.

## Important Information
- Linux native debugger sessions now enumerate `/proc/<pid>/task`, attach traced TIDs only while the debugger is active, and mirror hardware data breakpoints across stopped threads.
- While running, the Linux backend periodically syncs newly-created threads and polls traced thread events with `waitpid` using the Linux all-traced-threads wait flag.
- `native_debugger_smoke` now validates Linux x86_64 watchpoints through a worker-thread write to a static `AtomicU64`; the Linux child main thread only idles, so the smoke requires cross-thread watchpoint coverage.
- WSL validation passed: `cargo run -p squalr-plugin-debuggers-native --example native_debugger_smoke --locked`, `cargo test -p squalr-plugin-debuggers-native --locked`, and `cargo build -p squalr --locked`.
- Windows regression validation passed: `cargo check -p squalr --locked` and `cargo test -p squalr-tests --locked`.
- Android arm64 native debugger support reuses the ptrace worker/session flow with Android-specific `PTRACE_GETREGSET`/`PTRACE_SETREGSET` register access, `NT_ARM_HW_WATCH` watchpoint programming, and `PTRACE_GETSIGINFO` fault-address matching.
- Android compile validation passed: `cargo ndk --target aarch64-linux-android build -p squalr-plugin-debuggers-native --locked`, `cargo ndk --target aarch64-linux-android build -p squalr-plugin-debuggers-native --example native_debugger_smoke --locked`, and `python scripts/build_and_deploy.py --compile-check --debug`.
- Live Android smoke passed on device `4C101FDKD000Z8` via `/data/local/tmp/native_debugger_smoke` under `su`; it attached, armed `ptrace-1`, captured an arm64 hardware data breakpoint trace with 34 registers and 4 instruction bytes, and detached cleanly.
- Android `NT_ARM_HW_WATCH` writes must size the `iovec` to the watchpoint slot count reported in `dbg_info`; writing all 16 architectural slots returned `ENOSPC` on the test device.
- Linux/Android ptrace watchpoints now temporarily suppress the hit breakpoint for 100ms, apply that suppression across traced threads, and re-arm after the delay. This is intended to prevent hot watched addresses from trapping continuously and making the target appear hung.
- Latest validation passed: `cargo test -p squalr-plugin-debuggers-native --locked`, `cargo check -p squalr-plugin-debuggers-native --locked`, WSL `cargo test -p squalr-plugin-debuggers-native --locked`, WSL `cargo run -p squalr-plugin-debuggers-native --example native_debugger_smoke --locked`, `cargo ndk --target aarch64-linux-android build -p squalr-plugin-debuggers-native --example native_debugger_smoke --locked --release`, Android release smoke via `/data/local/tmp/native_debugger_smoke`, `python scripts/build_and_deploy.py --release`, `cargo test -p squalr-tests --locked`, and `git diff --check`.
- Android deployments should continue using `python scripts/build_and_deploy.py --release`; current release deploy produced a debug-signed APK wrapper with release Rust artifacts and launched `com.squalr.android` plus privileged `/data/local/tmp/squalr-cli --ipc-mode`.
- Project open no longer queries/applies plugin state when the loaded project has no plugin overrides. The plugin sync timeout is 30 seconds for real override work because Android privileged commands can sit behind long scans; this path still needs human verification in the GUI.
- Latest project-open validation passed: `cargo test -p squalr-engine project_plugin_configuration_skips_empty_overrides_without_privileged_dispatch --locked`, `cargo test -p squalr-tests --locked`, `python scripts/build_and_deploy.py --release`, and `git diff --check`. Full `cargo test -p squalr-engine --locked` still has an unrelated instruction-preview expectation mismatch: `inc dword ptr [rax]` vs `inc dword [rax]`.
- Linux/Android debugger attach now tolerates `PTRACE_ATTACH` returning `ESRCH` for non-leader threads that disappear after `/proc/<pid>/task` enumeration, and reports attach operation names with the TID/process id. The process leader still fails normally if it is gone; Android GUI attach still needs human verification.
- Latest debugger attach validation passed: `cargo test -p squalr-plugin-debuggers-native --locked`, WSL `cargo test -p squalr-plugin-debuggers-native --locked`, WSL `cargo run -p squalr-plugin-debuggers-native --example native_debugger_smoke --locked`, `cargo ndk --target aarch64-linux-android build -p squalr-plugin-debuggers-native --example native_debugger_smoke --locked --release`, Android release smoke via `/data/local/tmp/native_debugger_smoke`, `python scripts/build_and_deploy.py --release`, `cargo test -p squalr-tests --locked`, and `git diff --check`.
