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
- Keep CI target preflight aligned with the active Rust toolchain so Android compile-checks do not fail spuriously.
## Important Information

- `builtin.symbols.binary.populate-binary-symbols` stays format-generic and now populates Mach-O modules with typed header content, including concrete load-command layouts and typed segment section arrays instead of a raw `LoadCommands:u8[]` blob.
- Parameterized data type ids now support string semantics flags such as `string_utf8{null_terminated}`. The symbol registry resolves these through the registered base data type while preserving the parameterized id for display, parsing, and writes.
- Mach-O fixed C string fields such as `segname`, `sectname`, and inline dylib or rpath paths now use `string_utf8{null_terminated}` instead of raw `u8[]` buffers.
- UI data type rendering now normalizes parameterized ids back to their base data type label and icon, while symbol details expose string metadata such as fixed buffer size and null termination separately.
- `string_utf8` default values now allocate a single zero byte so fixed-size UTF-8 buffers expand to the correct byte count during preview and default-value construction instead of collapsing to zero-length reads.
- Preview formatting now treats string-format values separately from numeric arrays, so fixed string buffers such as Mach-O segment names and paths render as plain text with a wider truncation budget instead of bracketed array previews.
- Generic plugin execution coverage now includes both PE and Mach-O header population paths.
- `squalr-cli` now handles `ProcessResponse::Icon` instead of failing to compile when icon responses are enabled.
- `squalr-tui` preview formatting now uses the same three-argument `DataValuePreviewFormatOptions::new(array_elements, array_chars, string_chars)` call shape as the engine and GUI, so TUI builds stay compatible with the string preview formatter changes.
- Android compile preflight now probes installed Rust targets through `rustup +toolchain target list --installed` candidate toolchains instead of assuming a single active-toolchain identity, which keeps the nightly CI target check aligned with pinned or override-based installs.
- `os_behavior_command_tests` now distinguishes process open from explicit icon fetches: process open asserts the current non-icon-fetching query behavior, and icon fetch coverage lives in a dedicated `ProcessIconRequest` executor test.
