# Agentic Current Task
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- 

## Important Information

- Plugin extensibility now has coarse permissions for `Read/WriteSymbolStore`, `Read/WriteSymbolTreeWindow`, and `Read/WriteProcessMemory`, plus Symbol Tree plugin action traits.
- Added built-in `builtin.symbols.pe` plugin. It contributes a root-only `Populate PE Symbols` Symbol Tree action that reads module memory, validates `MZ`/`PE\0\0`, uses `e_lfanew`, and adds `DOS Header`, `DOS Stub`, `NT Headers`, and `Section Headers` fields with PE32/PE32+ struct descriptors.
- Symbol Explorer right-click menus discover enabled Symbol Tree plugin actions through the plugin registry and dispatch them through `ProjectSymbolsExecutePluginActionRequest`. Current execution path uses the built-in registry on the unprivileged command side; future plugin enablement persistence/sync may need tightening if non-default enablement matters for client-side action discovery.
