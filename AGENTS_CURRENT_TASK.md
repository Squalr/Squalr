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
- Audited symbolic struct representability against C/C++ layouts. Noted gaps: bitfields/sub-byte fields, ABI alignment metadata, faithful placed-size handling for overlapping/static-offset/dynamic layouts, first-class type aliases for layered pointers, and C++-specific semantic layouts.
- Follow-up audit produced concrete C++ struct samples that remain unrepresentable or semantically lossy in the current symbol model: bitfields, reusable unions/overlapping layouts with correct size, layered pointer aliases, empty/no-unique-address fields, and C++ inheritance/member-pointer semantics.
- Fixed the concrete union-like size bug for overlapping/static-offset layouts. Static struct sizing now uses layout span (`max(offset + size)`) instead of summing field sizes in domain struct sizing, Symbol Tree sizing, and module layout mutation sizing.
- Added first-class union support to reusable symbol layouts. `SymbolicStructDefinition` and `ValuedStruct` now carry a struct/union layout kind; union fields default to shared offset `0`, union size is the maximum field span, and valued union reads copy the same backing bytes into each member view. Needs human verification in the GUI.
- Renamed user-facing copy around the reusable type editor to "Symbol Layout Editor" / "Symbol Layouts" and added a Struct/Union selector while keeping existing Rust compatibility names for the editor internals.
- Follow-up rename pass moved the editor module/files/types to `symbol_layout_editor` / `SymbolLayoutEditor*`.

## Important Information

- Validated with `cargo test -p squalr-engine-domain symbolic_global_symbol_resolver --locked`, `cargo test -p squalr symbol_tree_entry --locked`, `cargo test -p squalr-engine project_symbol_layout_mutation --locked`, and `cargo test -p squalr-engine-api project_symbol_catalog --locked`.
- Audit validation ran `cargo test -p squalr-engine-domain symbolic_struct_resolver --locked` and `cargo test -p squalr symbol_tree_entry --locked`.
- Follow-up validation reran `cargo test -p squalr-engine-domain symbolic_struct_resolver --locked` and `cargo test -p squalr symbol_tree_entry --locked`.
- Union-size validation ran `cargo test -p squalr-engine-domain symbolic_struct_definition --locked`, `cargo test -p squalr-engine-domain symbolic_struct_resolver --locked`, `cargo test -p squalr symbol_tree_entry --locked`, and `cargo test -p squalr-engine project_symbol_layout_mutation --locked`.
- First-class union validation ran `cargo fmt --all`, `cargo test -p squalr-engine-domain symbolic_struct_definition --locked`, `cargo test -p squalr-engine-domain symbolic_struct_resolver --locked`, `cargo test -p squalr-engine-domain valued_struct --locked`, `cargo test -p squalr symbol_tree_entry --locked`, `cargo test -p squalr symbol_layout_editor --locked`, `cargo test -p squalr-engine project_symbol_layout_mutation --locked`, `cargo test -p squalr-engine promote_symbol --locked`, and `git diff --check`.
- Symbol Layout Editor rename validation ran `cargo fmt --all`, `cargo test -p squalr symbol_layout_editor --locked`, `cargo test -p squalr symbol_explorer --locked`, `cargo test -p squalr struct_viewer --locked`, a search for stale editor identifiers in `squalr` and `docs`, and `git diff --check`.
