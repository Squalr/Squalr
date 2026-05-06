# Agentic Current Task
Our current task, from `README.md`, is:
`pr/symbol-authoring-2`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- [X] Add resolver descriptors to the project symbol catalog as a new reusable symbol-authoring store.
- [X] Keep the initial resolver execution scope local to the current struct instance: local scalar fields plus type sizes.
- [X] Build a dedicated Symbol Resolvers window that edits resolver descriptors as a project-explorer-like tree, not as expression text. Needs human verification for live GUI ergonomics.
- [X] In the Struct Editor, let dynamic array counts and expression offsets pick a resolver from the global resolver list. Needs human verification for live GUI ergonomics.
- [X] Migrate PE header formulas to built-in/project resolver descriptors so `PE Headers` no longer depends on hardcoded inline expression strings.
- [X] Run focused domain/API/GUI tests for resolver serialization, evaluation, struct editor integration, Symbol Explorer resolution, and PE population.
- [ ] Replace inline symbolic-expression authoring in the Symbol Struct Editor with a first-class resolver model completely. The old text escape hatch remains temporarily.
- [ ] Add local resolver dependency validation and cycle diagnostics at the resolver-descriptor/catalog level.
- [ ] Remove or demote the current expression text builder UI after the resolver picker/tree editor path is human-verified.

## Important Information

- The current symbolic-expression system is real but too low-level for users. It stores inline AST expressions on `SymbolicFieldDefinition` for dynamic array counts and explicit offsets, with text only at parse/display/serde boundaries.
- Added `SymbolicResolverDefinition` / `SymbolicResolverDescriptor`. Resolver nodes are currently literal, local field, type size, and binary arithmetic operation. The current `local field` node is really a relative field reference against the struct instance evaluating the resolver, and should be renamed/refined as `relative field`. Binary operations are intentionally fixed to two children for simpler editing and operator swapping.
- `ProjectSymbolCatalog` now stores reusable symbolic resolver descriptors.
- `SymbolicFieldCountResolution` and `SymbolicFieldOffsetResolution` now support `Resolver(String)` in addition to inline `Expression(...)`. The string boundary format is `resolver(id)`.
- The current resolver evaluator is intentionally narrow. It does not provide global project symbol-chain inputs, pointer dereference inside expressions, or conditional type/layout selection.
- The current runtime resolver evaluates formulas/resolvers against scalar fields collected from the same struct instance. Symbol Explorer feeds those scalar values through a dedicated virtual snapshot lane so formulaic layouts can converge over frames.
- The current cycle detection only rejects local field-name dependency cycles inside a struct draft. It is not a project-wide dependency graph.
- Added a Symbol Resolvers dock window. It is titled `Symbol Resolvers`, defaults into the same tab group as Project Explorer, and uses the themed project controls instead of raw egui buttons/combos/text edits. Needs human verification for live GUI ergonomics.
- The Symbol Resolvers window now has distinct Project Explorer-style surfaces. List mode is a flat resolver list; the top bar `+` enters create-name mode, a row edit button enters rename-name mode, and double-clicking a row opens the resolver tree. Selecting a resolver does not implicitly begin editing. Needs human verification for live GUI ergonomics.
- Resolver create mode only asks for a name and creates the default rooted expression after save. Resolver rename mode only edits the name and has the Project Editor-style cancel/save/delete surface for existing resolvers. Needs human verification for live GUI ergonomics.
- Resolver opened mode shows the rooted expression tree. There is no tree add menu or morph button; node kind/operator/type details route through the existing Details Viewer (`StructViewerViewData`). Changing a node to an operation creates the fixed two child slots, and changing it back removes nesting through the same Details Viewer combo path. Changing node kind refocuses Details Viewer because the selected node's editable fields change shape. Needs human verification for live GUI ergonomics.
- Resolver list rows draw the edit button at the far right edge. Literal resolver node details use a numeric `i64` value field with bin/dec/hex display formats and hex as the default Details Viewer format. Needs human verification for live GUI ergonomics.
- The Struct Editor now offers resolver pickers for dynamic array counts and expression offsets, while keeping the old expression text editor as a temporary escape hatch.
- The Resolver Editor should eventually own resolver construction, validation, dependency visualization, and preview diagnostics.
- Global symbol inputs should be modeled as a structured symbol chain, not a free-text path. It should work like the existing ProjectItemAddress symbolic offset chain UX/model, except every chain entry must be a valid symbol segment and numeric bin/dec/hex pointer offsets are invalid. Pointer dereferences, module-base references, and conditional layout/type selection should be deferred until the resolver model is stable.
- Built-in PE population is now backed by PE resolver descriptors for dynamic offsets/counts. PE32 versus PE32+ selection is still plugin analysis today; resolver-based conditional layout selection is a later feature.
- Validation passed on 2026-05-05: `cargo test -p squalr-engine-domain --lib`, `cargo test -p squalr-engine-api --lib`, `cargo test -p squalr-plugin-symbols-pe`, and `cargo test -p squalr --lib`.
- Validation passed on 2026-05-06 after the Symbol Resolvers UI rewrite: `cargo test -p squalr --lib symbol_resolver_editor`, `cargo test -p squalr --lib dockable_window_settings`, and `cargo test -p squalr --lib`.
- Validation passed on 2026-05-06 after the tree/details correction: `cargo test -p squalr --lib symbol_resolver_editor` and `cargo test -p squalr --lib`.
- Validation passed on 2026-05-06 after simplifying Symbol Resolvers back to toolbar + tree only: `cargo test -p squalr --lib symbol_resolver_editor` and `cargo test -p squalr --lib`.
- Validation passed on 2026-05-06 after changing Symbol Resolvers to a single add menu and details node-type combo: `cargo test -p squalr --lib symbol_resolver_editor` and `cargo test -p squalr --lib`.
- Validation passed on 2026-05-06 after routing Symbol Resolver edits through Details Viewer instead of an in-window details strip: `cargo test -p squalr --lib symbol_resolver_editor` and `cargo test -p squalr --lib`.
- Validation passed on 2026-05-06 after refreshing Details Viewer when resolver node kind changes: `cargo test -p squalr --lib symbol_resolver_editor` and `cargo test -p squalr --lib`.
- Validation passed on 2026-05-06 after changing Symbol Resolvers to list mode plus explicit create/edit takeover: `cargo test -p squalr --lib symbol_resolver_editor` and `cargo test -p squalr --lib`.
- Validation passed on 2026-05-06 after splitting Symbol Resolvers into list, create-name, rename-name, and opened-tree surfaces: `cargo test -p squalr --lib symbol_resolver_editor` and `cargo test -p squalr --lib`.
- Validation passed on 2026-05-06 after right-aligning resolver row edit buttons and making literal details numeric: `cargo test -p squalr --lib symbol_resolver_editor` and `cargo test -p squalr --lib`.
