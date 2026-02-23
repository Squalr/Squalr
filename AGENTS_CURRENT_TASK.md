# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/release-test`

Our current task is to create git workflows to:
- Build for all platforms and valid combinations when a PR is rasied
- Block merging to main until these builds complete
- Support releasing to all platforms in a heavily automated way (ie `scripts/release.py` exists to bump versions, but we may want something CI friendly if we cant compile for all platforms locally?)
    - This needs auditing and a clear strategy.

# Notes from Owner (Readonly Section)
- Assume any unstaged/uncomitted file changes are from a previous iteration (or if this file, probably the human author giving guidance), and can be kept if they look good. Do not ask me about them.
- Assume any connected android devices are rooted, and assume MacOS has SIP disabled.
- You don't get to declare things as fixed. Only "need human verification".

## WONTFIX (For now)
- Add multi-data-type scan parity to GUI element scanner (`squalr/src/views/element_scanner/scanner/view_data/element_scanner_view_data.rs`) so one scan request can include multiple selected data types like TUI.
- Add GUI process list search/filter input parity with TUI process selector (`squalr/src/views/process_selector`) including in-memory filtering and refresh-aware state behavior.
- Add GUI project selector search/filter parity with TUI project list workflows (`squalr/src/views/project_explorer/project_selector`) so large project lists can be searched quickly.
- Add GUI output window controls parity with TUI (`squalr/src/views/output/output_view.rs`): clear log action and configurable max-line cap.
- Complete GUI settings parity with TUI for missing controls in memory/scan tabs (`squalr/src/views/settings/settings_tab_memory_view.rs`, `squalr/src/views/settings/settings_tab_scan_view.rs`), including start/end address editing, memory alignment, memory read mode, and floating-point tolerance.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Need human verification: rerun `.github/workflows/pr-validation.yml` and `.github/workflows/release.yml` Android jobs in GitHub Actions to confirm installing `platforms;android-30` and `build-tools;30.0.3` resolves the previous `Platform '30' is not installed` failure.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Existing workflows are branch-specific (`pr/linux`, `pr/unit-tests`) and do not currently provide a required `main` PR gate.
- Existing workflows cover Linux builds, `squalr-tests`, warning-baseline checks, and nightly workspace tests only.
- Added `.github/workflows/pr-validation.yml` for required `main`/`release/**` PR checks: Linux, Windows, macOS, Android compile-check, `squalr-tests`, and warning-baseline.
- Added `.github/workflows/release.yml` for tag/manual release automation with desktop+Android matrix packaging, artifact contract validation, and draft release publication.
- Refactored `scripts/release.py` into CI-callable phases (`version-bump`, `build-package`, `release-publish`) with `--release-type`, `--non-interactive`, `--no-version-bump`, and `--dry-run`.
- Added `docs/release-artifact-contract.md` documenting per-platform artifact names, checksums, and release safety controls.
- Local validation evidence captured: `python -m py_compile scripts/release.py`, `python scripts/release.py --step build-package ... --dry-run`, and `python scripts/release.py --step release-publish ... --dry-run`.
- Android build automation path exists at `python ./scripts/build_and_deploy.py --compile-check`; CI reuses it with `--debug` to avoid prompts.
- Updated `.github/workflows/pr-validation.yml` toolchain installs to `dtolnay/rust-toolchain@nightly` with `toolchain: nightly-2026-02-07`.
- Updated `.github/workflows/release.yml` build jobs to `dtolnay/rust-toolchain@nightly` with `toolchain: nightly-2026-02-07`.
- Updated `.github/workflows/workspace-nightly.yml` and `.github/workflows/squalr-tests-pr.yml` to the same pinned nightly (`nightly-2026-02-07`) for consistency.
- Updated `.github/workflows/pr-validation.yml` and `.github/workflows/release.yml` Android jobs to install SDK tools + `ndk;27.0.12077973` and export resolved `ANDROID_HOME` / `ANDROID_SDK_ROOT` / `ANDROID_NDK_ROOT` at runtime.
- Updated `scripts/build_and_deploy.py` preflight to conditionally require `adb`; compile-check mode now skips `adb` requirement.
- Local validation evidence captured: `python -m py_compile scripts/build_and_deploy.py`.
- Updated `scripts/build_and_deploy.py` workspace resolution to use repository root (`Path(__file__).resolve().parent.parent`) so Android APK build runs from `squalr/` instead of the invalid `scripts/squalr` path.
- Local validation evidence captured (2026-02-23): `python -m py_compile scripts/build_and_deploy.py scripts/release.py`, `python scripts/build_and_deploy.py --help`, and explicit path probe confirming `android_manifest_directory == <repo>/squalr`.
- Merge blocking must be enforced in GitHub branch protection settings after required checks are finalized (human-admin action).
- Local validation evidence captured (2026-02-23, revalidated): `python -m py_compile scripts/build_and_deploy.py scripts/release.py` and `cargo test -p squalr-tests --locked` (141 tests passed locally across the `squalr-tests` integration suites).
- Added Python cache ignore rules in `.gitignore` (`__pycache__/`, `*.pyc`) to prevent transient local artifacts from polluting git status.
- Updated `.github/workflows/pr-validation.yml` and `.github/workflows/release.yml` Android setup to install `platforms;android-30` and `build-tools;30.0.3` alongside `ndk;27.0.12077973`, matching `squalr/Cargo.toml` `target_sdk_version = 30`.
- Local validation evidence captured (2026-02-23): `cargo test -p squalr-tests --locked` passed after workflow updates (141 tests passed).
