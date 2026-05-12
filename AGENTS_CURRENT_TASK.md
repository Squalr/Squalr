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
- Polished the Symbol Layout Editor union workflow: new-layout create copy now uses a single "New Symbol Layout" groupbox, layout kind buttons are centered, row edit icons are flat, details view exposes a Struct/Union combo, and union edit rows render as variant selectors with tree affordances. Needs human verification in the GUI.
- Fixed the CLI project-symbol response handler for the newer `ExecutePluginAction` response variant.
- TODO: Replace synthesized `u8[n]` module gaps with `UNASSIGNED[n]` tree/editor spans. `UNASSIGNED` should mean "no project symbol owns these bytes" and should not be assignable from normal data type selectors.
- TODO: Add an explicit raw byte storage type such as `db`/`bytes` for claimed-but-uninterpreted memory. Use this for intentional padding, union tail bytes, blobs, and other user-owned raw storage instead of overloading `u8`.
- TODO: Refactor module splitting/deletion flows so removed fields become synthesized `UNASSIGNED` gaps instead of persisted `u8[n]` filler fields. Defining a field should insert into a non-overlapping unassigned range; deleting explicit `db` should reveal `UNASSIGNED` again.

## Important Information

- Current `u8[n]` overload sites to audit include `SymbolTreeEntryKind::U8Segment`, `append_u8_segment_entry`, `begin_define_field_from_u8_segment`, `rename_u8_segment`, tests named around "u8 segment", and project-symbol mutation helpers such as `delete_module_ranges_to_u8_fields`, `replace_u8_array_field_span`, `find_containing_u8_array_field_position`, and `resolve_u8_array_length`.
- Desired semantics: `UNASSIGNED[n]` is synthesized from gaps between assigned module fields/claims and module bounds; `db[n]`/`bytes[n]` is a real persisted assignment; `u8` remains an actual one-byte unsigned integer interpretation only.
- Deleted symbol-layout fallback should stop retargeting to `u8` by default. Prefer removing ownership to expose `UNASSIGNED`, retargeting to explicit `db[n]` when a claimed raw span must remain, or requiring a replacement if size cannot be inferred.
- Validated with `cargo test -p squalr-engine-domain symbolic_global_symbol_resolver --locked`, `cargo test -p squalr symbol_tree_entry --locked`, `cargo test -p squalr-engine project_symbol_layout_mutation --locked`, and `cargo test -p squalr-engine-api project_symbol_catalog --locked`.
- Audit validation ran `cargo test -p squalr-engine-domain symbolic_struct_resolver --locked` and `cargo test -p squalr symbol_tree_entry --locked`.
- Follow-up validation reran `cargo test -p squalr-engine-domain symbolic_struct_resolver --locked` and `cargo test -p squalr symbol_tree_entry --locked`.
- Union-size validation ran `cargo test -p squalr-engine-domain symbolic_struct_definition --locked`, `cargo test -p squalr-engine-domain symbolic_struct_resolver --locked`, `cargo test -p squalr symbol_tree_entry --locked`, and `cargo test -p squalr-engine project_symbol_layout_mutation --locked`.
- First-class union validation ran `cargo fmt --all`, `cargo test -p squalr-engine-domain symbolic_struct_definition --locked`, `cargo test -p squalr-engine-domain symbolic_struct_resolver --locked`, `cargo test -p squalr-engine-domain valued_struct --locked`, `cargo test -p squalr symbol_tree_entry --locked`, `cargo test -p squalr symbol_layout_editor --locked`, `cargo test -p squalr-engine project_symbol_layout_mutation --locked`, `cargo test -p squalr-engine promote_symbol --locked`, and `git diff --check`.
- Symbol Layout Editor rename validation ran `cargo fmt --all`, `cargo test -p squalr symbol_layout_editor --locked`, `cargo test -p squalr symbol_explorer --locked`, `cargo test -p squalr struct_viewer --locked`, a search for stale editor identifiers in `squalr` and `docs`, and `git diff --check`.
- Symbol Layout Editor union workflow validation ran `cargo fmt --all`, `cargo test -p squalr symbol_layout_editor --locked`, `cargo test -p squalr struct_viewer --locked`, `cargo test -p squalr symbol_explorer --locked`, and `git diff --check`.
- CLI project-symbol response validation ran `cargo fmt --all`, `cargo build -p squalr-cli --locked`, and `git diff --check`.
