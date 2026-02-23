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
- Release-profile Android builds fail fast when `[package.metadata.android.signing.release]` is missing.

## Platform Artifacts
Version placeholder: `${VERSION}` without `v` prefix.

### Linux (`linux-x86_64`)
- `squalr-${VERSION}-linux-x86_64.zip`
- `squalr-${VERSION}-linux-x86_64.manifest.json`
- `latest_version-${VERSION}-linux-x86_64.txt`
- `SHA256SUMS-linux-x86_64.txt`

### Windows (`windows-x86_64`)
- `squalr-${VERSION}-windows-x86_64.zip`
- `squalr-${VERSION}-windows-x86_64.manifest.json`
- `latest_version-${VERSION}-windows-x86_64.txt`
- `SHA256SUMS-windows-x86_64.txt`

### macOS (`macos-aarch64`)
- `squalr-${VERSION}-macos-aarch64.zip`
- `squalr-${VERSION}-macos-aarch64.manifest.json`
- `latest_version-${VERSION}-macos-aarch64.txt`
- `SHA256SUMS-macos-aarch64.txt`

### Android (`android-aarch64`)
- `squalr-android-aarch64.apk`
- `squalr-cli-android-aarch64`
- `squalr-${VERSION}-android-aarch64.manifest.json`
- `latest_version-${VERSION}-android-aarch64.txt`
- `SHA256SUMS-android-aarch64.txt`

## Publish Stage
- Aggregates all platform artifacts from workflow artifacts.
- Verifies required files exist before publish.
- Generates aggregate checksum file: `SHA256SUMS.txt`.
- Creates/updates GitHub Release as a draft and uploads all artifacts with `--clobber`.
