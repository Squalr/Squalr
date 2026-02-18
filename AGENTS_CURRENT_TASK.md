# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/linux`

# Notes from Owner (Readonly Section)
- 

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Validate with clean Linux builds and smoke-run core binaries (currently blocked locally until Rust toolchain is available).
- Open/update PR with scoped commits and follow-up checklist for remaining platform parity work (blocked locally until `gh auth login` is completed).

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Existing WIP implementation PR: https://github.com/Squalr/Squalr/pull/6/changes
- The referenced PR targets an outdated build/version of Squalr and should be treated as historical context, not a directly mergeable baseline.
- Local environment blocker: `cargo` is not installed in this execution environment, so build reproduction/validation cannot be executed locally here.
- Workspace manifest contained an unstable Cargo feature flag (`#![feature(profile-rustflags)]`) that can break stable toolchains; removed.
- `squalr-engine-operating-system` now gates `windows-sys` under `target_os = "windows"` target dependencies.
- Added Linux CI build workflow at `.github/workflows/linux-build.yml` to build GUI/CLI/TUI on `pr/linux`.
- README now documents Linux native package prerequisites and standardized Linux build commands.
- Validation attempt on 2026-02-18 confirmed `cargo` and `rustc` are unavailable in this environment (`command not found`), so local build/test execution remains blocked.
- GitHub CLI is installed, but PR inspection/push workflows are blocked in this environment because no GitHub account is authenticated (`gh auth status`: not logged in, checked on 2026-02-18).
- Safety hardening: replaced `static mut` + `unwrap_unchecked()` singleton initialization in `squalr-engine-operating-system` (`memory_reader`, `memory_writer`, `memory_queryer`, `memory_settings_config`) with `OnceLock`-based initialization.
- Re-validation attempt on 2026-02-18: `cargo fmt --all -- --check`, `cargo test --locked`, and `cargo build -p squalr-cli --locked` all remain blocked by missing `cargo` (`command not found`).
