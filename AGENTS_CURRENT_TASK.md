# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/installer-issues`

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

- Need human verification: Windows installer completion view shows the primary-blue `Launch Squalr` button and clicking it launches the installed GUI after the installer closes, without hanging.
- Need human verification: Windows installer window at smaller heights keeps the install/launch action row left aligned, keeps the footer visible, and lets the log panel collapse to remaining space.
- Need human verification: `target/release/squalr.exe` launches as the GUI app without spawning a console window and no longer crashes during the initial process-list/icon refresh in the packaged release flow.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- 2026-03-15: Release-mode startup crashes came from the Windows process icon extraction path. Rendering extracted `HICON`s into a 32-bit DIB with explicit process/icon/DC cleanup made `squalr.exe` and `squalr-cli process list -i` stable again in release. Need human verification in the packaged release flow.
- 2026-03-15: Installer completion-state launch now returns a post-render action; `InstallerApp` performs the spawn after releasing the UI-state mutex, and `UpdateOperationLaunch` no longer terminates the parent process. Launch failures now render in the completion card. Need human verification in the installer UI.
- 2026-03-15: The completion-state `Launch Squalr` button now uses the installer's primary blue button styling instead of the success-green override. Need human verification in the installer UI.
- 2026-03-15: Installer log overflow came from a hard-coded 290px log body minimum. The installer layout now allocates the log section from remaining height so footer/status stay visible at smaller window sizes. Need human verification in the installer UI.
