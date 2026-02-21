# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- 

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- [x] Investigate macOS run/build viability using current workspace targets.
- [x] Document macOS-specific runtime/security requirements in `README.md`.
- [x] Reduce macOS process listing stall in process query path.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- `cargo build -p squalr-cli --locked`, `cargo build -p squalr-tui --locked`, and `cargo build -p squalr --locked` all succeed on macOS.
- macOS runtime process access depends on Mach APIs (`task_for_pid`, `mach_vm_read_overwrite`, `mach_vm_write`), so Developer Tools authorization is a primary whitelist requirement.
- The GUI updater performs HTTPS calls to `api.github.com`; restrictive firewall/proxy environments may need an allow rule.
- macOS process list performance fix: replaced per-process window scans with one shared CoreGraphics window scan per query, moved icon loading after filter checks, and cached icons by executable path within the process.
- macOS icon correctness fix: switched icon resolution to `NSRunningApplication` by PID (with executable-path fallback), which avoids generic executable/terminal icons returned by `iconForFile` for many processes.
- macOS open-process diagnostics improved: `OpenProcessFailed` now includes details, and `task_for_pid` failures report the concrete kern status so permission/target-protection issues are visible in logs/UI.
