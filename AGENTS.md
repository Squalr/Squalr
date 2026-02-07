# AGENTS.MD
Always scan the README.md file to get a good project overview.

You should look at the Agentic Current Task section to pick up on your previous work.

After each session, you should attempt to checkpoint your work with a commit, and fill out the relevant sections of Agentic Current Task.

## Coding Conventions
- All variable names should be coherent. This means 'i' is forbidden. 'idx' is forbidden. In fact, even 'index' is often bad, because you should generally say what the index is for, ie 'snapshot_index'. You are a systems programmer, not an academic.
- No unhandled unwraps, panics, etc. On failure, either return a result, or log an error or warning with the log! macro.
- Comments end with a period.
- All code is to be formatted by the default rust formatter. This project already has a bundled .rustfmt.toml, so this should get picked up automatically by tooling.
- Commenting functions with intellisense friendly comments is preferred when possible.

## Agentic Current Task
We are working on a pr/unit-tests branch, to create squalr-tests as a project within this repository. The goal is to test the commands sent by the gui/cli. In other words, we are testing our command/response system, and ensuring the commands do what we expect them to.

The tricky part is doing dependency injection over the OS APIs to emulate what an OS might return. OS APIs are abstracted, so this is should be extensible.

This should cover various cases like scanning for different data types, writing memory, reading memory, querying memory ranges, etc...

If functionality is too hard to test, note down why its better to not have the test yet and instead wait for the implementation to change. Always cross-reference this with the Architecture Plan. If the plan seems too complicated, cut scope. If the tests can't comply due to poor architecture, just note it down and leave the test stubbed.

#### Scratchpad (Agents can modify this!)

#### Architecture Plan (Agents can modify this!)
Iterate on this section with the architecture plan. Prefer simplicty, while staying within the bounds of the README.md plan.

#### Concise Session logs (Agents can modify this!)
For each PR, append to this section a summary of the work accomplished.

## Agentic Eventually TODO list
- pr/cli-bugs - The cli build currently does not even spawn a window. The cli should be able to spawn visibly and execute commands. It has not been functional for many months, causing drift. Observe the gui project (squalr) for reference to functional code. Both projects leverage squalr-engine / squalr-engine-api for the heavy lifting.
- pr/error_handling - We currently have a mixed use of Result<(), String>, anyhow! based errors, and error enums. Update the project to the following: Within squalr-engine, errors should be struct based. Within squalr-cli and squalr, anyhow! based errors are fine (ignore squalr-android and squalr-tui for now).


## Agentic Off Limits List
These are not ready to be picked up yet.
- pr/tui - We want a TUI project at some point. Would be good to get this working loosely based on both the cli and 
