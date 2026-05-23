# Agentic Current Task
Our current task, from `README.md`, is:
`pr/todo`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- 

## Important Information

- Added a repo-level `rust-toolchain.toml` pinned to `nightly` so local builds stop depending on per-directory rustup overrides. Needs human verification against the exact Windows nightly date/hash before tightening to `nightly-YYYY-MM-DD`.
