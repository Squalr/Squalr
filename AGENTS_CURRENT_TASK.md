# Agentic Current Task
Our current task, from `README.md`, is:
`release updater transient missing asset handling`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist
- Completed: The current Rust updater treats a latest GitHub release that is visible before its required platform bundle asset as a transient skip instead of a hard update failure.
- Completed: Release asset lookup is shared through `GitHubReleaseInfo::find_asset_by_name`, and the installer download fallback uses the same case-insensitive matching.
- Completed: Added tests for missing latest-release assets and normal platform bundle resolution.

## Important Information
- The reported stack trace is from the legacy Squirrel updater path, where GitHub exposed `Squalr v0.4.0` before the `RELEASES` asset was visible. In the current Rust updater, the analogous case is a latest release without `squalr-<version>-<os>-<arch>.zip`.
- Validation: `cargo fmt --all` completed with existing `fn_args_layout` warnings; `cargo test -p squalr-engine update_asset_download_url -- --nocapture` passed 2 targeted tests; `cargo check -p squalr-engine` passed; `cargo test -p squalr-engine app_provisioner -- --nocapture` passed 11 app-provisioner tests. One attempted command, `cargo test -p squalr-engine latest_release_url_targets_live_squalr_repository release_bundle_asset_name_uses_current_target -- --nocapture`, failed because Cargo accepts only one test filter.
- Human verification needed: Publish or simulate a latest release that lacks the current platform bundle and confirm the updater logs a warning and leaves the existing install running without surfacing a failed update. This needs human verification.
