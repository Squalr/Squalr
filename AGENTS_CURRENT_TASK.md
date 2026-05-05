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

- Plugin extensibility is moving toward coarse permissions plus registration surfaces. Added `PluginPermission` for `Read/WriteSymbolStore`, `Read/WriteSymbolTreeWindow`, and `Read/WriteProcessMemory`; added Symbol Tree plugin action traits and an engine-backed project symbol store implementation for plugin actions. Validated with focused API/session tests; needs integration into the Symbol Tree context menu and a concrete PE/OS symbol population plugin.
