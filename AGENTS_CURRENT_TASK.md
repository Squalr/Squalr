# Agentic Current Task
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.

## Current Tasklist
- 

## Important Information
- Linux WSL sanity check:
  - Used WSL distro `Ubuntu` from the Windows workspace mount `/mnt/c/Projects/squalr_workspace`.
  - `cargo build -p squalr --locked` completed successfully in WSL and produced a Linux ELF GUI binary at `target/debug/squalr`.
  - Direct launch with `timeout 20s ./target/debug/squalr` reached WSLg/OpenGL initialization and Squalr startup logs, then the timeout stopped it. Interactive GUI behavior still needs human verification after implementation and validation.
  - For an interactive visible run from Windows, launch a visible host with `cmd.exe /k wsl.exe -d Ubuntu -- bash -lc "cd /mnt/c/Projects/squalr_workspace && ./target/debug/squalr"`. Plain background jobs inside a short-lived `wsl.exe` invocation were torn down when the launcher exited.
  - Linux-side `git status` reports many files modified from the mounted checkout even when Windows-side Git is clean; observed full-file line-ending churn such as `README.md` showing `353/353` numstat. Do not normalize or commit those WSL-only line-ending artifacts.
