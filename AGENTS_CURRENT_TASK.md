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
- `Populate PE Symbols` now treats the populated PE header span as authoritative and removes existing module fields that land in or overlap that span before inserting the generated fields.
- Symbol Tree fixed arrays now expand when the array element type is an expandable struct, so module fields like `win.pe.IMAGE_SECTION_HEADER[3]` expose section-header element nodes and nested fields instead of remaining as array leaves.
- Began dynamic struct support with first-class symbolic expressions on fields: users can represent shapes like `elements:Element[count] @ +8` and `unfilled:Element[capacity - count] @ +8 + count * sizeof(Element)`. Added a generic formulaic struct resolver that evaluates formulas from previously read scalar fields, reports per-field diagnostics, and clamps dynamic array preview counts. Field count/offset behavior is represented as explicit resolution states (`Inferred` / `Sequential` / expression), not optional expression wrappers.
- Symbol Tree now has a testable scalar-reader entrypoint for formulaic layouts. A PE-shaped dynamic layout test resolves `SectionHeaders:section_header[NumberOfSections] @ e_lfanew + 24 + SizeOfOptionalHeader` to concrete section-header entries when scalar values are supplied. The live GUI still uses the no-scalar-reader path, so the PE populate action remains on the concrete generated-field path until tree-time memory-backed scalar resolution is wired.
- Symbol Explorer right-click menus discover enabled Symbol Tree plugin actions through the plugin registry and dispatch them through `ProjectSymbolsExecutePluginActionRequest`. Current execution path uses the built-in registry on the unprivileged command side; future plugin enablement persistence/sync may need tightening if non-default enablement matters for client-side action discovery.
