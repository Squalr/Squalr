# Release Artifact Contract

The CI release workflow (`.github/workflows/release.yml`) enforces this artifact contract for tag/manual releases.

## Trigger Model
- Tag trigger: `push` tag `v<major>.<minor>.<patch>` publishes a draft release.
- Manual trigger: `workflow_dispatch` requires `tag_name` and defaults to `dry_run=true`.
- Manual trigger can optionally create a missing tag with `create_tag_if_missing=true`.

## Safety Controls
- Draft-only publishing is always used.
- `dry_run=true` validates and packages artifacts without creating/editing a GitHub release.
- Tag format must match `v<major>.<minor>.<patch>`.
- Manual runs fail if the tag is missing and `create_tag_if_missing=false`.
- Release-profile Android builds fall back to a debug APK when `[package.metadata.android.signing.release]` is missing.

## Platform Artifacts
Version placeholder: `${VERSION}` without `v` prefix.

### Linux (`linux-x86_64`)
- `squalr-${VERSION}-linux-x86_64.zip`

### Windows (`windows-x86_64`)
- `squalr-${VERSION}-windows-x86_64.zip`

### macOS (`macos-aarch64`)
- `squalr-${VERSION}-macos-aarch64.zip`

### Android (`android-aarch64`)
- `squalr.apk`
- `squalr-cli`
- `README-android.md`
- `install-android.sh`
- `install-android.ps1`

## Publish Stage
- Aggregates all platform artifacts from workflow artifacts.
- Verifies required files exist before publish.
- Generates aggregate checksum file: `SHA256SUMS.txt`.
- Creates/updates GitHub Release as a draft and uploads all artifacts with `--clobber`.

## Updater Contract
- Desktop installer/updater resolves the latest GitHub release JSON from the GitHub Releases API.
- Desktop installer/updater requires only the matching platform bundle asset: `squalr-${VERSION}-<os>-<arch>.zip`.
- Per-platform manifests, version marker files, and per-platform checksum files are not part of the auto-update flow.
