# AGENTS.MD

## Workflow
- Always scan `README.md` first for the project overview + architecture constraints.
- Then read `AGENTS_CURRENT_TASK.md` and continue from there.
- End each session by:
  - removing unused imports / dead helpers,
  - running relevant tests,
  - checkpointing with a commit,
  - Update the relevant fields in `AGENTS_CURRENT_TASK.md`, compacting information as necessary.

## Coding Conventions
- Variable names must be coherent and specific. `i`, `idx`, and generic `index` are forbidden. Use names like `snapshot_index`, `range_index`, `scan_result_index`, etc. You are a systems programmer, not an academic.
- No unhandled `unwrap()`, panics, etc. On failure: return a `Result` or log via `log!` macros.
- Comments end with a period.
- Format with default Rust formatter (repo includes `.rustfmt.toml`).
- Prefer rustdoc/intellisense-friendly function comments where practical.
- Remove unused imports.
- Prefer single-responsibility principal. Do not inline structs that do not belong in the file. Make separate files for separate concerns.

## Unit Tests
If it makes sense to test it, add test cases. Test should generally be robust for things like the squalr-api.

As for CLI/GUI, tests do not need be as robust. The CLI is just a simple command invoker, and the GUI is a complex command invoker. Commands are of course the important bits to test.

If you must patch source files to fix bugs while testing, that's acceptable as long as we are adhering to the architecture in `README.md` and `AGENTS_CURRENT_TASK.md`.

If something is too hard to test:
- Stub the test, and write down **why** (architecture limitation) + what would need to change.
- Keep notes short and aligned with the README architecture plan.
- Do not keep it in the tasklist anymore.
