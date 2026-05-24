# Agentic Current Task
Our current task, from `README.md`, is:
`pr/todo`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- Completed: Revisited PE symbol generation for the new `string_utf8{null_terminated}` support. `IMAGE_SECTION_HEADER.Name` now uses an 8-byte fixed null-terminated UTF-8 field instead of `u8[8]`.

## Important Information

- Validation: `cargo fmt --all` completed with existing rustfmt deprecation warnings for `fn_args_layout`; `cargo test -p squalr-plugin-binary-symbols` passed 18 tests.
- Human verification: PE section names should now display as normal C-style short names while retaining the full 8-byte header field size.
