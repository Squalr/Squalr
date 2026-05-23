# Agentic Current Task
Our current task, from `README.md`, is:
`pr/todo`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- Investigate macOS usermode memory filtering without relying on brittle heuristics.
- Keep binary symbol population generic by detecting the module header format instead of assuming host OS.
## Important Information

- `builtin.symbols.binary.populate-binary-symbols` now detects Mach-O and populates a `Mach-O Headers` root with parsed fixed header layouts plus raw load-command bytes.
- Generic plugin execution coverage now includes both PE and Mach-O header population paths.
