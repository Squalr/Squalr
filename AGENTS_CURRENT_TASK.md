# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/release-test`

Our current task is to create git workflows to:
- Build for all platforms and valid combinations when a PR is rasied
- Block merging to main until these builds complete
- Support releasing to all platforms in a heavily automated way (ie `scripts/release.py` exists to bump versions, but we may want something CI friendly if we cant compile for all platforms locally?)
    - This needs auditing and a clear strategy.

# Notes from Owner (Readonly Section)
- Assume any unstaged/uncomitted file changes are from a previous iteration, and can be kept if they look good. Do not ask me about them.
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

- [x] Baseline CI/release audit: mapped current workflows (`linux-build.yml`, `squalr-tests-pr.yml`, `workspace-nightly.yml`), documented missing PR coverage for `main`, and captured required build matrix (desktop CLI/TUI/GUI on Windows/Linux/macOS plus Android compile checks). Need human verification.
- [x] Created a single required PR validation workflow targeting `main` and `release/**` with path filters covering engine/app crates and workflow files (`.github/workflows/pr-validation.yml`). Need human verification.
- [x] Implemented Linux PR job parity with README build contract (`cargo build -p squalr-cli --locked`, `cargo build -p squalr-tui --locked`, `cargo build -p squalr --locked`) including native package install + cache. Need human verification.
- [x] Implemented Windows PR job parity for CLI/TUI/GUI locked builds with retained logs via uploaded artifacts. Need human verification.
- [x] Implemented macOS PR job parity for CLI/TUI/GUI locked builds including GUI build scope. Need human verification.
- [x] Implemented Android PR compile-check job via existing script (`python ./scripts/build_and_deploy.py --compile-check --debug`) with non-interactive flags/env setup. Need human verification.
- [x] Kept unit/integration checks in required PR flow (`cargo test -p squalr-tests --locked` and warning-baseline gate for touched crates) in unified workflow. Need human verification.
- [x] Added branch-protection runbook with required checks, branch rules, and explicit GitHub branch-protection merge-blocking note (`docs/branch-protection-runbook.md`). Need human verification.
- [x] Replace interactive release flow with CI-friendly release pipeline: tag/manual trigger that runs cross-platform build matrix, creates deterministic artifacts, and publishes a GitHub Release draft with checksums (`.github/workflows/release.yml`, `scripts/release.py`). Need human verification.
- [x] Refactor `scripts/release.py` for automation mode (`--release-type`, `--non-interactive`, optional `--no-version-bump`) and split responsibilities: version bump, build/package, and release-publish steps callable from CI. Need human verification.
- [x] Define artifact contract per platform (naming, archive format, included resources, `latest_version` handling), then enforce it in CI with validation steps (`docs/release-artifact-contract.md`, `release.yml` validate step, script manifests/checksums). Need human verification.
- [x] Add rollback/safety controls for release workflow (dry-run mode, draft-only publish default, idempotent tag handling, and fail-fast on missing artifacts/signing config). Need human verification.
- [ ] Validate workflows locally where feasible (`act`/script dry-runs) and via PR test branches; record evidence and mark each checklist item as "need human verification" before closure. Local script dry-runs completed; PR test-branch workflow runs still need human verification.

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
- Merge blocking must be enforced in GitHub branch protection settings after required checks are finalized (human-admin action).
