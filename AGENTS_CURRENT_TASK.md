# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/plugins-data-types`

# Notes from Owner (Readonly Section)
- Assume any unstaged/uncomitted file changes are from a previous iteration (or if this file, probably the human author giving guidance), and can be kept if they look good. Do not ask me about them.
- Assume any connected android devices are rooted, and assume MacOS has SIP disabled.
- You don't get to declare things as fixed. Only "need human verification".

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Need human verification: toggle the 24-bit data-type plugin on/off in the GUI, confirm `u24`, `u24be`, `i24`, and `i24be` appear and disappear from scan selectors, and confirm pointer scanner only exposes `u24`/`u24be` when enabled.
- Need human verification: save a project with the 24-bit plugin enabled, reopen it, and confirm the plugin state is restored from project config.
- TUI element scanner still uses a fixed data-type list and does not yet dynamically hide plugin-gated types.


## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Added a built-in data-type plugin for 24-bit integers: `builtin.data-type.24bit-integers`.
- Added concrete `u24`, `u24be`, `i24`, and `i24be` data types with scalar scan support and formatting/parsing helpers.
- 24-bit types are scalar-only for scans; the planner now falls back from vector to scalar when no vector comparer exists.
- Pointer scanner now supports `u24` and `u24be`, including pointer reads and scalar search routing.
- Visible data types are filtered through the privileged registry catalog based on enabled plugins.
- Enabled plugin ids are now persisted per-project and reapplied on project open.
