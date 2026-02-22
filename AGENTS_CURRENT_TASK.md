# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- Assume any unstaged file changes are from a previous iteration, and can be kept if they look good
- The android device is rooted.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- The standard squalr bulid now crashes on boot.
- The pr/android branch now contains a ton of bloaty changes that are questionable. Diff vs main. (formatting changes are OK)

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Fixed Android windowed filtering false positives by requiring a process to be the primary package process (`cmdline == package`) before it can be considered windowed.
- This excludes colon-suffixed auxiliary/service processes (for example `com.app:worker`) from windowed-only results.
- Added Android unit tests for primary-package process classification in `android_process_query.rs`.
- Validation run: `cargo fmt --all`, `cargo test -p squalr-tests --locked`, `cargo check -p squalr-engine-operating-system --target aarch64-linux-android --locked`.
