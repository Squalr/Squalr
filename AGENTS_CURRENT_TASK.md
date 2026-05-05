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
- Added built-in `builtin.symbols.pe` plugin. It contributes a root-only `Populate PE Symbols` Symbol Tree action that reads module memory, validates `MZ`/`PE\0\0`, uses `e_lfanew`, and adds a formulaic `PE Headers` module field with PE32/PE32+ root descriptors.
- `Populate PE Symbols` now treats the populated PE header span as authoritative and removes existing module fields that land in or overlap that span before inserting the generated fields.
- Symbol Tree fixed arrays now expand when the array element type is an expandable struct, so module fields like `win.pe.IMAGE_SECTION_HEADER[3]` expose section-header element nodes and nested fields instead of remaining as array leaves.
- Began dynamic struct support with first-class symbolic expressions on fields: users can represent shapes like `elements:Element[count] @ +8` and `unfilled:Element[capacity - count] @ +8 + count * sizeof(Element)`. Added a generic formulaic struct resolver that evaluates formulas from previously read scalar fields, reports per-field diagnostics, and clamps dynamic array preview counts. Field count/offset behavior is represented as explicit resolution states (`Inferred` / `Sequential` / expression), not optional expression wrappers.
- Symbol Tree now has a testable scalar-reader entrypoint for formulaic layouts. A PE-shaped dynamic layout test resolves `SectionHeaders:section_header[NumberOfSections] @ e_lfanew + 24 + SizeOfOptionalHeader` to concrete section-header entries when scalar values are supplied.
- The live Symbol Explorer now feeds formulaic layout resolution from a dedicated scalar-value virtual snapshot lane. Expanded structs schedule integer scalar reads, cache the results by locator/field/type, and replay those values into `build_symbol_tree_entries_with_scalar_reader` on later frames. This lets dynamic arrays converge in the GUI without a second memory-read pipeline.
- The PE populate action now relies on formulaic `PE Headers` children for `DOSStub`, `NTHeaders`, and `SectionHeaders`. The plugin still chooses the PE32 versus PE32+ root descriptor during memory analysis; the symbol expression system does not yet have first-class conditional type/layout selection from `OptionalHeader.Magic`.
- Memory-read struct materialization now treats fields whose type id resolves to a struct layout as nested valued structs instead of empty default data values. Registry lookup misses are quiet because lookups are frequently used as probes; this avoids spam when opening nested PE fields like `NTHeaders`.
- Symbol Explorer right-click menus discover enabled Symbol Tree plugin actions through the plugin registry and dispatch them through `ProjectSymbolsExecutePluginActionRequest`. Current execution path uses the built-in registry on the unprivileged command side; future plugin enablement persistence/sync may need tightening if non-default enablement matters for client-side action discovery.
- Symbol Explorer context menus can now add resolved non-root Symbol Tree entries, including derived PE struct fields, to the project. Root module fields still prefer symbolic project-item targets; derived rows use their resolved locator and full path.
- Struct-typed Symbol Tree rows now expose `Edit Struct Layout...` in the context menu. The action opens/selects the Symbol Struct Editor and enters edit mode for the row's project struct layout. Needs human verification in the live GUI.
- Symbol Struct Editor usage counts now include module fields and nested struct field references, and layout renames update direct module-field type references.
- Validation: `cargo test -p squalr --lib` passed with 347 tests on 2026-05-04.
- Cleaned up symbolic expressions so `SymbolicExpression` is now the parsed AST, not stored source text. Text remains only as parse/display/serde boundary data; `sizeof(...)` stores `DataTypeRef`, identifiers are wrapped in `SymbolicExpressionIdentifier`, and evaluation walks the AST directly.
- Validation: `cargo test -p squalr-engine-domain --lib`, `cargo test -p squalr --lib`, and `cargo test -p squalr-plugin-symbols-pe` passed on 2026-05-04.
- Decoupled dynamic-layout scalar reads from hardcoded type-id decoding. `DataType` now explicitly opts into scalar integer layout values and owns byte interpretation through the registry, with a shared scalar integer byte decoder for odd widths such as plugin `u24`/`i24`. The Symbol Explorer scalar snapshot lane now asks the registry whether a field type can feed expressions and reads values through `DataValue` + `DataType`, not `AnonymousValueString` or GUI-local type-id matches.
- Validation: `cargo test -p squalr-engine-domain --lib`, `cargo test -p squalr-engine-api --lib`, `cargo test -p squalr-engine-session --lib`, `cargo test -p squalr-plugin-data-types-24bit`, `cargo test -p squalr-plugin-symbols-pe`, and `cargo test -p squalr --lib` passed on 2026-05-04.
- Symbol Struct Editor can now round-trip and author formulaic layout fields. Field drafts preserve dynamic array count expressions and offset expressions as UI-boundary text, then parse them back into `SymbolicExpression` AST values when saving. The editor adds a `Dynamic Array` container mode for expression counts and an `Offset` mode selector for sequential versus expression offsets. Needs human verification for live GUI ergonomics and information density.
- Validation: `cargo test -p squalr --lib symbol_struct_editor` and `cargo test -p squalr --lib` passed on 2026-05-04.
