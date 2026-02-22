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

- [ ] Baseline CI/release audit: map current workflows (`linux-build.yml`, `squalr-tests-pr.yml`, `workspace-nightly.yml`), document missing PR coverage for `main`, and capture required build matrix (desktop CLI/TUI/GUI on Windows/Linux/macOS plus Android compile checks).
- [ ] Create a single required PR validation workflow targeting `main` (and any release branch policy) with path filters that still include all engine/app crates and workflow files.
- [ ] Implement Linux PR job parity with README build contract (`cargo build -p squalr-cli --locked`, `cargo build -p squalr-tui --locked`, `cargo build -p squalr --locked`) including native package install + cache.
- [ ] Implement Windows PR job parity for CLI/TUI/GUI locked builds; ensure artifacts or logs are retained when failures occur.
- [ ] Implement macOS PR job parity for CLI/TUI/GUI locked builds; keep GUI build in scope to catch platform-specific regressions.
- [ ] Implement Android PR compile-check job using existing script (`python ./build_and_deploy.py --compile-check`) with non-interactive flags/env setup so CI never prompts.
- [ ] Keep unit/integration checks in required PR flow (`cargo test -p squalr-tests` and warning-baseline gate for touched crates), consolidating existing logic into the main PR workflow or a reusable called workflow.
- [ ] Add branch-protection runbook in docs: required checks list, expected branch rules, and explicit note that merge blocking is configured in GitHub branch protection (human-admin action).
- [ ] Replace interactive release flow with CI-friendly release pipeline: tag/manual trigger that runs cross-platform build matrix, creates deterministic artifacts, and publishes a GitHub Release draft with checksums.
- [ ] Refactor `scripts/release.py` for automation mode (`--release-type`, `--non-interactive`, optional `--no-version-bump`) and split responsibilities: version bump, build/package, and release-publish steps callable from CI.
- [ ] Define artifact contract per platform (naming, archive format, included resources, `latest_version` handling), then enforce it in CI with validation steps.
- [ ] Add rollback/safety controls for release workflow (dry-run mode, draft-only publish default, idempotent tag handling, and fail-fast on missing artifacts/signing config).
- [ ] Validate workflows locally where feasible (`act`/script dry-runs) and via PR test branches; record evidence and mark each checklist item as "need human verification" before closure.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Existing workflows are branch-specific (`pr/linux`, `pr/unit-tests`) and do not currently provide a required `main` PR gate.
- Existing workflows cover Linux builds, `squalr-tests`, warning-baseline checks, and nightly workspace tests only.
- `scripts/release.py` is currently interactive and Windows-centric (`squalr.exe`, `squalr-installer.exe` assumptions), so it is not CI-ready for multi-platform release automation.
- Android build automation path already exists via `python ./build_and_deploy.py --compile-check`; CI integration should reuse this instead of duplicating logic.
- Merge blocking must be enforced in GitHub branch protection settings after required checks are finalized (human-admin action).
