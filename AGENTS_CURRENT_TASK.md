# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- Assume any unstaged/uncomitted file changes are from a previous iteration (or if this file, probably the human author giving guidance), and can be kept if they look good. Do not ask me about them.
- Assume any connected android devices are rooted, and assume MacOS has SIP disabled.
- You don't get to declare things as fixed. Only "need human verification".

## WONTFIX (For now)
- Add multi-data-type scan parity to GUI element scanner (`squalr/src/views/element_scanner/scanner/view_data/element_scanner_view_data.rs`) so one scan request can include multiple selected data types like TUI.
- Add GUI process list search/filter input parity with TUI process selector (`squalr/src/views/process_selector`) including in-memory filtering and refresh-aware state behavior.
- Add GUI project selector search/filter parity with TUI project list workflows (`squalr/src/views/project_explorer/project_selector`) so large project lists can be searched quickly.
- Add GUI output window controls parity with TUI (`squalr/src/views/output/output_view.rs`): clear log action and configurable max-line cap.
- Complete GUI settings parity with TUI for missing controls in memory/scan tabs (`squalr/src/views/settings/settings_tab_memory_view.rs`, `squalr/src/views/settings/settings_tab_scan_view.rs`), including start/end address editing, memory alignment, memory read mode, and floating-point tolerance.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Audit GUI vs TUI for non-WONTFIX parity gaps and add concrete follow-up tasks.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- `squalr-installer` now defers installation until user clicks `Install Squalr`; startup no longer auto-runs install.
- Installer UI now pre-fills default install directory but allows editing before install.
- Installer UI now includes warning-colored text: existing contents of selected install directory are deleted before install (matches `AppInstaller::prepare_install_directory` behavior).
- PR validation Windows job now compiles `squalr-installer` and uploads `windows-squalr-installer.log`; `build-windows` previously only built CLI/TUI/GUI crates.
- Installer footer `Launch Squalr` now launches GUI and closes installer window; need human verification.
- App installer/updater now prefer platform bundle assets (`squalr-<version>-<os>-<arch>.zip`) with legacy fallback to `squalr.zip`; need human verification against live releases.
- Release packaging writes top-level text manifests (`MANIFEST-<target>.txt`) and keeps version/hash metadata as top-level `.txt` files; need human verification in GitHub release output.
