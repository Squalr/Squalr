# Agentic Current Task
Our current task, from `README.md`, is:
`pr/android-fixes-v2 PR cleanup`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist
- Completed: Android privileged IPC keeps the worker alive without stdin, uses Serde-compatible JSON command/event payloads, and the deploy script clears stale workers before launch.
- Completed: Android privileged-worker logs forward through structured logging events; the unprivileged host re-logs them with a `[privileged]` prefix for the Output dock.
- Completed: Android logging suppresses repeated `Notifying Input Available` records and does not forward debug/trace scan progress logs over privileged IPC.
- Completed: Android privileged IPC response callbacks run outside the request-handle lock, preventing scan response callbacks from deadlocking follow-up element scans.
- Completed: Android process selection, process switching, project creation, and editable widgets use Android-specific paths for the rooted GUI workflow.
- Completed: Android GUI packaging uses the `eframe`/`android-activity` GameActivity backend and a generated Gradle APK under `target/android-gameactivity-gradle`.
- Completed: Android scan region collection keeps the two-phase query/read model while pruning unreadable ranges, obvious device-backed mappings, zero-RSS smaps entries, and `io`/`pf` smaps mappings before value collection.
- Completed: Android process-memory read failures tombstone failed ranges without per-read log spam, and scan metadata reports collected bytes rather than virtual snapshot span.
- Completed: Android soft-keyboard input is handled in the local `winit 0.30.13` patch by forwarding GameActivity text commits through synthetic text-bearing key events, suppressing duplicate printable key events while IME is active, clearing the hidden text buffer after commits, and treating subsequent empty text events as Backspace.
- Completed: PR cleanup removed obsolete input-iteration notes from this task file, removed text-content debug logging from the winit bridge, and clarified the README Android patch note.

## Important Information
- Latest validation after PR cleanup: `cargo fmt --all` and `cargo fmt --manifest-path third_party/winit-0.30.13/Cargo.toml` completed with existing `fn_args_layout` warnings; `cargo check -p squalr` passed; `cargo ndk --target aarch64-linux-android test --manifest-path third_party/winit-0.30.13/Cargo.toml --no-run --no-default-features --features android-game-activity,rwh_06` compiled the Android winit test harness; `python -m py_compile .\scripts\build_and_deploy.py` passed; `git diff --check` passed with line-ending warnings only. `python .\scripts\build_and_deploy.py --release --launch-log-seconds 8 --launch-log-file .\target\android-launch-logcat-pr-cleanup-release.txt` built and deployed the release Android CLI/APK path to the attached device, launched the app, detected privileged worker PID `3449`, and completed smoke validation. The saved logcat had no matches for `Failed to read process memory`, `Notifying Input Available`, `PayloadDeserializationFailed`, `Broken pipe`, `TextEvent`, `Unknown android_activity input event`, `Committing GameActivity`, or `Treating empty GameActivity`.
- Human verification needed: On Android release builds, confirm insert/delete behavior in project rename, project item rename, Output command input, and data value boxes; confirm repeated Backspace works on existing text; confirm element scans start, complete, clear the result spinner, and do not flood logs or collect obvious device/kernel mappings. This needs human verification.
