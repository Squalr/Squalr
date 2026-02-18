# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/linux`

# Notes from Owner (Readonly Section)
- 

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Runtime verify Linux icon fidelity on real desktop sessions (Wayland + X11): confirm `StartupWMClass`/desktop-ID matching resolves expected app icons and document any mismatches.

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
- Historical smoke-run (pre-fix) at 2026-02-18: CLI/TUI/GUI `--help` paths exited early from Linux bootstrap because `start_monitoring` was `not_implemented`.
- Linux path normalization fix landed for project item selection/reorder flows: Windows-style absolute paths (`C:/...`) are now treated as absolute on non-Windows hosts.
- CLI response handling now covers `MemoryResponse::Freeze` via a dedicated handler to keep API/CLI response matching exhaustive.
- Linux process monitoring parity fix at 2026-02-18 05:46:38Z: `LinuxProcessQuery::start_monitoring` and `stop_monitoring` now return `Ok(())`, matching the immediate-operation model used by other platforms.
- Revalidation at 2026-02-18 05:46:38Z: `cargo fmt --all`, `cargo test -p squalr-engine-operating-system --locked`, and Linux builds for CLI/TUI/GUI passed; startup no longer fails on `start_monitoring` not implemented. Remaining runtime issues observed: CLI `--help` still exits `1` with usage error text, TUI requires interactive terminal, GUI fails in headless/no-GL environment.
- CLI `--help` behavior fix at 2026-02-18 05:48:11Z: parser now injects a synthetic binary argv token and treats `structopt` help/version parse outcomes as successful display output, so one-shot help exits `0` with clean command help text instead of usage error failure.
- Validation at 2026-02-18 05:48:11Z: `cargo test -p squalr-cli --locked` passed (including new CLI parse tests) and `cargo run -p squalr-cli -- --help` exited `0`.
- Linux process open/close parity fix at 2026-02-18 05:49:56Z: `LinuxProcessQuery::open_process` now returns `OpenedProcessInfo` (handle `0`, `Bit64`) and `close_process` now returns `Ok(())`, matching immediate-operation behavior used by other non-Windows platforms.
- Linux process query tests added at 2026-02-18 05:49:56Z: validates `open_process` field mapping and `close_process` success semantics.
- Validation at 2026-02-18 05:49:56Z: `cargo fmt --all` and `cargo test -p squalr-engine-operating-system --locked` passed.
- Linux stub hygiene at 2026-02-18 05:49:56Z: removed unused imports/dead helper and unused parameter warnings in `linux_memory_reader`, `linux_memory_writer`, and `linux_memory_queryer`.
- Linux runtime parity implementation at 2026-02-18: implemented `LinuxProcessQuery::get_processes` with `sysinfo` filtering support and ELF-based bitness detection from `/proc/<pid>/exe`; Linux open-process handles now map to PID values for consistent downstream memory operations.
- Linux memory I/O implementation at 2026-02-18: `linux_memory_reader` now uses `process_vm_readv` and `linux_memory_writer` now uses `process_vm_writev`, with strict full-length transfer checks for `read`, `read_struct`, `read_bytes`, and `write_bytes`.
- Linux memory query implementation at 2026-02-18: `linux_memory_queryer` now parses `/proc/<pid>/maps`, applies protection/type filters with range bounds handling, evaluates writability, and resolves modules (`get_modules`, `address_to_module`, `resolve_module`) from executable file-backed mappings.
- Validation at 2026-02-18: `cargo fmt --all`, `cargo test -p squalr-engine-operating-system --locked`, `cargo build -p squalr-cli --locked`, `cargo build -p squalr-tui --locked`, and `cargo build -p squalr --locked` all passed after Linux runtime parity changes.
- Linux `require_windowed` fidelity improvement at 2026-02-18 05:55:55Z: process windowed detection now identifies active GUI clients by matching `/proc/<pid>/fd` socket inodes against Wayland/X11 sockets parsed from `/proc/net/unix`, replacing the prior environment-variable heuristic.
- Linux process query tests added at 2026-02-18 05:55:55Z: coverage now includes parsing socket inode targets and display-socket inode filtering for Wayland/X11 detection.
- Validation at 2026-02-18 05:55:55Z: `cargo fmt --all`, `cargo test -p squalr-engine-operating-system --locked`, `cargo build -p squalr-cli --locked`, `cargo build -p squalr-tui --locked`, and `cargo build -p squalr --locked` all passed after Linux windowed detection changes.
- Linux process icon parity implementation at 2026-02-18 06:00:43Z: `LinuxProcessQuery::get_processes` now honors `fetch_icons` by mapping `/proc/<pid>/exe` names to `.desktop` entries (`Exec`/`Icon`) and loading raster icon assets from standard user/system icon directories into `ProcessIcon`.
- Linux process icon parser tests added at 2026-02-18 06:00:43Z: coverage now includes shell-style exec token parsing (`env` prefix + quotes) and desktop-entry extraction/NoDisplay filtering.
- Validation at 2026-02-18 06:00:43Z: `cargo fmt --all`, `cargo test -p squalr-engine-operating-system --locked`, `cargo build -p squalr-cli --locked`, `cargo build -p squalr-tui --locked`, and `cargo build -p squalr --locked` passed after Linux icon-fetch changes.
- Linux icon fidelity follow-up landed at 2026-02-18 06:06:26Z: desktop-entry icon lookup now additionally indexes `StartupWMClass`, desktop file stem (desktop ID), and full desktop filename; process icon lookup now tries `/proc/<pid>/exe`, `/proc/<pid>/cmdline`, `/proc/<pid>/comm`, and runtime process name keys.
- Linux SVG icon support landed at 2026-02-18 06:06:26Z: icon resolver now accepts `.svg` and decodes vector icons into RGBA using `resvg` (`usvg` + `tiny-skia`) when themes do not provide raster assets.
- Validation at 2026-02-18 06:06:26Z: `cargo fmt --all`, `cargo test -p squalr-engine-operating-system`, `cargo build -p squalr-engine-operating-system --locked`, `cargo build -p squalr-cli --locked`, `cargo build -p squalr-tui --locked`, and `cargo build -p squalr --locked` passed.
