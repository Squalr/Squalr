# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/cli-installer`

# Notes from Owner (Readonly Section)
- Assume any unstaged/uncomitted file changes are from a previous iteration (or if this file, probably the human author giving guidance), and can be kept if they look good. Do not ask me about them.
- Assume any connected android devices are rooted, and assume MacOS has SIP disabled.
- You don't get to declare things as fixed. Only "need human verification".

## WONTFIX (For now)
- Add GUI process list search/filter input parity with TUI process selector (`squalr/src/views/process_selector`) including in-memory filtering and refresh-aware state behavior.
- Add GUI project selector search/filter parity with TUI project list workflows (`squalr/src/views/project_explorer/project_selector`) so large project lists can be searched quickly.
- Add GUI output window controls parity with TUI (`squalr/src/views/output/output_view.rs`): clear log action and configurable max-line cap.
- Complete GUI settings parity with TUI for missing controls in memory/scan tabs (`squalr/src/views/settings/settings_tab_memory_view.rs`, `squalr/src/views/settings/settings_tab_scan_view.rs`), including start/end address editing, memory alignment, memory read mode, and floating-point tolerance.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Need human verification: Windows installer reveals a styled `Launch Squalr` button on install completion and that action launches the installed GUI before closing the installer.
- Need human verification: Windows installer window at smaller heights keeps the footer visible and lets the log panel collapse to remaining space.
- Need human verification: Windows Start Menu registration now surfaces Squalr in Start/search after install.
- Need human verification: `target/release/squalr.exe` launches as the GUI app without spawning a console window in the packaged release flow.
- Need human verification: GitHub release Android output is a single `squalr-<version>-android-aarch64.zip` bundle containing the APK, CLI, README, and install scripts.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- 2026-03-15: Installer action buttons now use an installer-local port of the main GUI button state-layer styling. The completion-state `Launch Squalr` action was moved into the install status card and launches the installed GUI before closing the installer. Need human verification in the installer UI.
- 2026-03-15: `squalr.exe` was still being built as a console app in release because `windows_subsystem = "windows"` only existed on `squalr/src/lib.rs`; moving the attribute to `squalr/src/main.rs` produced a PE with `IMAGE_SUBSYSTEM_WINDOWS_GUI`. Need human verification in the packaged release flow.
- 2026-03-15: Installer log overflow came from a hard-coded 290px log body minimum. The installer layout now allocates the log section from remaining height so footer/status stay visible at smaller window sizes. Need human verification in the installer UI.
- 2026-03-15: Windows shortcut registration now resolves the Start Menu Programs folder via `SHGetKnownFolderPath(FOLDERID_Programs)` and stamps the shortcut with `com.squalr.desktop` as the AppUserModelID. Need human verification that Windows Start/search picks it up reliably.
- 2026-03-15: Android release artifacts are now intended to ship as one `squalr-<version>-android-aarch64.zip` bundle instead of free-floating files. Need human verification in GitHub Actions/releases.
- 2026-03-15: GitHub Actions Node 20 deprecation follow-up. `actions/checkout@v4`, `actions/upload-artifact@v4`, and `actions/download-artifact@v4` have newer Node 24-compatible majors available. `android-actions/setup-android` is still on `v3.2.2` and its published action metadata still declares `runs.using: node20`, so the Android jobs were switched to a repo-local shell setup instead. Need human verification in GitHub Actions.
