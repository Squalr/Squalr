# Agentic Current Task
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.

## Current Tasklist
- Linux x86_64 native debugger watchpoints are implemented in `squalr-plugin-debuggers-native` with ptrace hardware data breakpoints for read/write/access tracing. GUI workflow still needs human verification after implementation and validation.

## Important Information
- Linux debugger implementation status:
  - The native debugger plugin now advertises Linux x86_64 attach support for x86/x64 targets only.
  - The Linux backend uses `PTRACE_ATTACH`, `PTRACE_GETREGS`, `PTRACE_SETREGS`, `PTRACE_PEEKUSER`, `PTRACE_POKEUSER`, and x86_64 DR0-DR3/DR6/DR7 hardware watchpoints.
  - Linux x64 data watchpoint hits report the post-trap RIP; the backend includes a trace message because instruction attribution may point after the accessing instruction.
  - Automated WSL smoke passed with `cargo run -p squalr-plugin-debuggers-native --example native_debugger_smoke --locked`: attach, register snapshot, write watchpoint, trace event, and detach completed.
  - Windows regression `cargo test -p squalr-tests --locked` passed, including the memory write response tests.
  - Rebuilt and launched current WSL GUI and watch-value helper visibly for manual attach testing. At the time of update, WSL showed `./target/debug/squalr` PID 1960 and `./target/linux-tools/squalr-watch-value` PID 1957.
- Linux WSL sanity check:
  - Used WSL distro `Ubuntu` from the Windows workspace mount `/mnt/c/Projects/squalr_workspace`.
  - `cargo build -p squalr --locked` completed successfully in WSL and produced a Linux ELF GUI binary at `target/debug/squalr`.
  - Direct launch with `timeout 20s ./target/debug/squalr` reached WSLg/OpenGL initialization and Squalr startup logs, then the timeout stopped it. Interactive GUI behavior still needs human verification after implementation and validation.
  - For an interactive visible run from Windows, launch a visible host with `cmd.exe /k wsl.exe -d Ubuntu -- bash -lc "cd /mnt/c/Projects/squalr_workspace && ./target/debug/squalr"`. Plain background jobs inside a short-lived `wsl.exe` invocation were torn down when the launcher exited.
  - Added `scripts/linux_watch_value.c` as a disposable Linux target with a static volatile `watch_value`. Build it with `wsl.exe -d Ubuntu -- bash -lc 'cd /mnt/c/Projects/squalr_workspace && mkdir -p target/linux-tools && cc -O0 -g -fno-pie -no-pie -Wall -Wextra -o target/linux-tools/squalr-watch-value scripts/linux_watch_value.c'`, then launch visibly with `cmd.exe /k wsl.exe -d Ubuntu -- bash -lc "cd /mnt/c/Projects/squalr_workspace && ./target/linux-tools/squalr-watch-value"`.
  - Linux-side `git status` reports many files modified from the mounted checkout even when Windows-side Git is clean; observed full-file line-ending churn such as `README.md` showing `353/353` numstat. Do not normalize or commit those WSL-only line-ending artifacts.
