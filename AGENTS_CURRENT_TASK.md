# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/linux`

# Notes from Owner (Readonly Section)
- 

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Open/update PR for `pr/linux` with scoped commits and a follow-up checklist for remaining platform parity work.
- Follow up Linux runtime parity gap: `start_monitoring` is not implemented on Linux, so CLI/TUI/GUI bootstrap currently exits before normal command help/shell flow.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Existing WIP implementation PR: https://github.com/Squalr/Squalr/pull/6/changes
- The referenced PR targets an outdated build/version of Squalr and should be treated as historical context, not a directly mergeable baseline.
- Workspace manifest contained an unstable Cargo feature flag (`#![feature(profile-rustflags)]`) that can break stable toolchains; removed.
- `squalr-engine-operating-system` now gates `windows-sys` under `target_os = "windows"` target dependencies.
- Added Linux CI build workflow at `.github/workflows/linux-build.yml` to build GUI/CLI/TUI on `pr/linux`.
- README now documents Linux native package prerequisites and standardized Linux build commands.
- Safety hardening: replaced `static mut` + `unwrap_unchecked()` singleton initialization in `squalr-engine-operating-system` (`memory_reader`, `memory_writer`, `memory_queryer`, `memory_settings_config`) with `OnceLock`-based initialization.
- Environment revalidation at 2026-02-18 05:42:02Z: `cargo 1.95.0-nightly` and `rustc 1.95.0-nightly` are installed; `gh auth status` is logged in as `zcanann`.
- Linux build prerequisites were installed from README on 2026-02-18 (`pkg-config`, ALSA, udev, xkbcommon, Wayland, X11/Xcursor/Xi/Xrandr/Xinerama development packages).
- Validation at 2026-02-18: `cargo fmt --all -- --check` passed and `cargo test --locked` passed.
- Clean Linux build validation at 2026-02-18: `cargo clean` then `cargo build -p squalr-cli --locked`, `cargo build -p squalr-tui --locked`, and `cargo build -p squalr --locked` all succeeded.
- Smoke-run validation at 2026-02-18: `./target/debug/squalr-cli --help`, `./target/debug/squalr-tui --help`, and `./target/debug/squalr --help` each exit with code `1` due to Linux engine bootstrap failure: `start_monitoring` not implemented.
- Linux path normalization fix landed for project item selection/reorder flows: Windows-style absolute paths (`C:/...`) are now treated as absolute on non-Windows hosts.
- CLI response handling now covers `MemoryResponse::Freeze` via a dedicated handler to keep API/CLI response matching exhaustive.
