# AGENTS.MD
Always scan the README.md file to get a good project overview.

## Coding Conventions
- All variable names should be coherent. This means 'i' is forbidden. 'idx' is forbidden. In fact, even 'index' is often bad, because you should generally say what the index is for, ie 'snapshot_index'. You are a systems programmer, not an academic.
- No unhandled unwraps, panics, etc. On failure, either return a result, or log an error or warning with the log! macro.
- Comments end with a period.
- All code is to be formatted by the default rust formatter. This project already has a bundled .rustfmt.toml, so this should get picked up automatically by tooling.
- Commenting functions with intellisense friendly comments is preferred when possible.

## Agentic TODO list
These are TODO list items. To prevent stomping on your own work, create a separate branch for each task. If the branch exists, you probably already have done the task, and should move onto another.
- pr/cli-bugs - The cli build currently does not even spawn a window. The cli should be able to spawn visibly and execute commands. It has not been functional for many months, causing drift. Observe the gui project (squalr) for reference to functional code. Both projects leverage squalr-engine / squalr-engine-api for the heavy lifting.
- pr/error_handling - We currently have a mixed use of Result<(), String>, anyhow! based errors, and error enums. Update the project to the following: Within squalr-engine, errors should be struct based. Within squalr-cli and squalr, anyhow! based errors are fine (ignore squalr-android and squalr-tui for now).
- pr/unit-tests - The project has no unit tests yet. Given the command response architecture, surely this is not too hard to do. The tricky part is doing dependency injection over the OS APIs to emulate what an OS might return. OS APIs are abstracted, so this is fine.

## Agentic Off Limits List
These are not ready to be picked up yet.
- pr/tui - We want a TUI project at some point. Would be good to get this working loosely based on both the cli and 