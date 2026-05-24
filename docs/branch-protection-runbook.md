# Branch Protection Runbook

This runbook documents the required GitHub checks and branch rules for PR merge blocking.

## Required PR Check Workflow

Configure these jobs from `.github/workflows/pr-validation.yml` as required status checks:

- `build-linux`
- `build-windows`
- `build-macos`
- `build-android-compile-check`
- `squalr-tests`
- `warning-baseline`

## Branch Rules

Apply branch protection to:

- `main`
- `release/**`

Recommended settings:

- Require a pull request before merging.
- Require status checks to pass before merging.
- Require branches to be up to date before merging.
- Include administrators (optional but recommended for consistency).
- Do not allow force pushes.
- Do not allow deletions.

## Merge Blocking Note

Merge blocking is enforced by GitHub branch protection configuration, not by workflow YAML alone. Repository admins must configure the rule in GitHub settings and select the required checks listed above.
