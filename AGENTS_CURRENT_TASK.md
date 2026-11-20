# Agentic Current Task
Our current task, from `README.md`, is:
`pr/session-architecture`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.

## Current Tasklist
- Pointer scan results use 16-byte heap slots. API and scanning tests cover graph IDs, browsing, and compatibility with existing saved data.
- PR #53 borrows the active graph during validation and skips terminal heap scan tasks during discovery.
- Implement the architecture audit findings for session ownership, command completion, and HTTP worker access. Windows and Android compilation validated. The Android device flow needs human verification.

## Important Information
- Attachments invalidate scan, pointer, and freeze state synchronously. Providers own target liveness and handle release. Standalone and HTTP commands capture the attachment generation before queued execution.
- The CLI, GUI, and TUI consume the same worker responses. IPC delivers callbacks outside the reader, resolves interrupted requests, and signals disconnection to the worker binary. Embedded sessions own periodic worker shutdown. Interactive apps opt into update checks.
- HTTP protocol v1 uses authenticated loopback requests, one controller credential per worker, client-owned projects, and an event cursor taken from the returned batch. GUI and TUI network-client selection is still planned.
- Windows validation passed the full workspace test suite, live HTTP authentication and command failure checks, and a live named-pipe registry request followed by clean worker termination. Android arm64 CLI and GUI passed the NDK compile check. No Android device was attached.
- PR validation selects the runner's JDK 17 for Gradle 7.5.1. All APK assembly paths inherit the configured Java environment.
