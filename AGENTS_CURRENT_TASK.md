# Agentic Current Task
Our current task, from `README.md`, is:
`pr/todo`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- Investigated symbol cycle handling. Global symbol field resolution already uses a resolver session stack; added an indirect global-cycle regression test.

## Important Information

- Validated with `cargo test -p squalr-engine-domain symbolic_global_symbol_resolver --locked`, `cargo test -p squalr symbol_tree_entry --locked`, `cargo test -p squalr-engine project_symbol_layout_mutation --locked`, and `cargo test -p squalr-engine-api project_symbol_catalog --locked`.
