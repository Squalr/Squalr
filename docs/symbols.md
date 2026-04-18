# Symbols UX Plan

## Purpose
This document captures the immediate findings from reviewing Ghidra's symbol and data-type UX, then turns those findings into a concrete Squalr plan.

This is not a "copy Ghidra exactly" document. The goal is to understand which use cases Ghidra separates cleanly, where Squalr currently overloads one window with too many jobs, and how to evolve Squalr without throwing away the current symbol-store backend direction in `docs/symbol_store.md`.

## Related Squalr Context
- `squalr/src/views/main_window/main_window_view.rs` currently treats `Project Explorer`, `Symbol Explorer`, `Struct Viewer`, `Memory Viewer`, and `Code Viewer` as peer docked windows.
- `squalr/src/views/symbol_explorer/symbol_explorer_view.rs` currently mixes rooted-symbol browsing, derived child browsing, inline authoring, preview, and navigation.
- `squalr/src/views/struct_viewer/view_data/struct_viewer_focus_target.rs` and `squalr/src/views/struct_viewer/view_data/struct_viewer_view_data.rs` currently make `Struct Viewer` the real detail/editor sink for multiple explorers.
- `squalr-engine-api/src/commands/project_symbols/project_symbols_command.rs` already gives us a real rooted-symbol command lane.
- `squalr-engine-api/src/structures/projects/project_items/built_in_types/project_item_type_symbol_ref.rs` already lets project items resolve through rooted symbols instead of only raw addresses.
- `squalr-engine-domain/src/registries/symbols/symbol_registry.rs` already acts as both a symbol/type registry and an execution-time resolver.

So the main problem is no longer "we have no symbol architecture." The main problem is that the UX split is still muddled.

## What Ghidra Actually Separates

### Symbol Table
Ghidra's Symbol Table is the authoritative flat maintenance surface, not the hierarchical browser.

Use cases it covers:
- browse all symbols in one place,
- sort and filter by source, symbol type, and advanced criteria,
- see symbol attributes like address, type, namespace, and references,
- perform direct maintenance actions like rename, delete, pin, and selection-driven navigation,
- view symbols that are awkward or impossible to understand from a tree alone, including dynamic-symbol-heavy cases.

Important takeaway:
- this is where users answer "what symbols do I have?" and "show me the weird or bulk-edit cases."

### Symbol Tree
Ghidra's Symbol Tree is the hierarchical browsing and navigation surface.

Use cases it covers:
- browse symbols by namespace/category,
- expand functions into parameters and locals,
- navigate code by clicking symbols in a tree,
- understand structural organization like imports, labels, classes, and namespaces,
- keep clutter under control with group nodes when a category is huge.

Important takeaway:
- this is where users answer "how is this program organized?" and "what lives under this scope?"
- Ghidra explicitly does not treat the tree as the only symbol display. Some symbols are intentionally better surfaced in the Symbol Table.

### Data Type Manager
Ghidra's Data Type Manager is much broader than a struct editor.

Use cases it covers:
- create and edit structures, unions, enums, typedefs, and function definitions,
- organize data types into categories and archives,
- find types by name, size, structure size, or offset,
- copy, move, rename, delete, and resolve conflicts,
- commit, update, revert, and disassociate archive-backed types,
- capture function definitions from a program and apply them to other programs,
- export data types as headers and create labels from enums.

Important takeaway:
- this is where users answer "what reusable types do I own?" and "how do I manage type reuse and conflicts across programs?"

### Listing and Browser Integration
Ghidra does not force users to leave the listing every time they discover something.

Use cases it covers:
- create and apply structures directly in the browser,
- rename structure fields from the browser,
- apply data types to memory locations, parameters, locals, and symbols,
- expand and collapse structures and arrays inline in the listing,
- use decompiler-driven structure creation and class filling workflows.

Important takeaway:
- this is where users answer "I just found something; let me name/type/fold it right here."

This is probably the most important UX lesson for Squalr.

## Sources Reviewed
- Symbol Table: <https://www.ghidradocs.com/9.2_PUBLIC/help/Base/help/topics/SymbolTablePlugin/symbol_table.htm>
- Symbol Tree: <https://www.ghidradocs.com/9.2.3_PUBLIC/help/Base/help/topics/SymbolTreePlugin/SymbolTree.htm>
- Data Type Manager overview: <https://www.ghidradocs.com/9.1_PUBLIC/help/Base/help/topics/DataTypeManagerPlugin/data_type_manager_description.htm>
- Data Type Manager window: <https://www.ghidradocs.com/10.3_PUBLIC/help/Base/help/topics/DataTypeManagerPlugin/data_type_manager_window.html>
- Code Browser: <https://www.ghidradocs.com/11.0_PUBLIC/help/Base/help/topics/CodeBrowserPlugin/CodeBrowser.htm>
- Data plugin: <https://www.ghidradocs.com/11.2_PUBLIC/help/Base/help/topics/DataPlugin/Data.htm>
- Advanced class notes: <https://ghidradocs.com/12.0.1_PUBLIC/docs/GhidraClass/Advanced/improvingDisassemblyAndDecompilation.pdf>

## Where Squalr Feels Frustrating Today
Squalr currently blends four separate jobs together:
- symbol browsing,
- symbol maintenance,
- data-type management,
- symbol/type authoring at the place of discovery.

The current shape roughly looks like this:
- `Project Explorer` is still part acquisition tool, part organization surface, and part symbol conversion surface.
- `Symbol Explorer` is trying to be both a symbol table and a symbol tree.
- `Struct Viewer` is acting as the real details/editor pane for whichever explorer currently owns focus.
- `Memory Viewer` and `Code Viewer` can consume symbols, but they are still relatively weak symbol-authoring surfaces.

That means users have to remember which pane currently "really owns" a symbol workflow instead of being guided by a clearer mental model.

## Recommended Window Model

### 1. Symbol Table
Add a dedicated flat symbol window backed by `project_symbols`.

Responsibilities:
- authoritative list of rooted symbols,
- search, sort, and filter,
- bulk rename/delete/update operations,
- columns for at least name, locator, type, preview value, and source metadata,
- direct jump to memory/code,
- selection syncing into `Struct Viewer`.

This should become the best maintenance surface.

### 2. Symbol Tree
Re-scope the current `Symbol Explorer` into the hierarchical browser.

Responsibilities:
- rooted symbols as top-level authored nodes,
- lazy derived children from type layout,
- on-demand pointer expansion,
- grouping by source/module/namespace if useful later,
- promotion of a derived child into a new rooted symbol,
- jump to memory/code,
- selection syncing into `Struct Viewer`.

This should become the best exploration surface.

### 3. Data Type Manager
Add a first-class type-management window on top of the existing registry/layout work.

Responsibilities:
- browse reusable symbol/type definitions,
- create and rename types,
- edit field names, offsets, and containers,
- organize types into categories later,
- search by type name and size,
- handle conflict resolution and future import/export/archive flows,
- capture and apply reusable function signatures if we support that later.

This should become the best type-authoring surface.

### 4. Struct Viewer
Keep `Struct Viewer`, but narrow its role.

Responsibilities:
- detail/editor sink for the currently selected symbol, symbol child, or project item,
- live value preview and editing,
- field-level property editing,
- side-by-side inspector, not the primary owner of browsing/navigation.

This should stop feeling like the secret place where symbol details really live.

### 5. Project Explorer
Keep `Project Explorer` focused on acquisition and workflow artifacts.

Responsibilities:
- address items, pointer items, folders, scans, and future workflow objects,
- promotion into rooted symbols,
- conversion between acquisition items and symbol refs when appropriate,
- not the primary long-term symbol-authoring surface.

This keeps raw discoveries separate from the authored symbolic model.

## Listing-Integrated Authoring
This is the biggest Ghidra lesson and probably the highest-value UX shift.

Squalr should support more symbol/type actions directly from `Code Viewer` and `Memory Viewer`:
- assign symbol at current address,
- rename symbol at current address,
- apply type to current address or selection,
- create type from selection,
- jump from a byte/instruction selection to the corresponding symbol,
- expand and collapse applied structured data inline when the listing model is ready.

Short version:
- discovery should happen in the viewer,
- durable organization should happen in symbol/type windows,
- details should land in `Struct Viewer`.

## Recommended Phasing

### Phase 1: Clarify the information architecture
1. Keep the current backend direction from `docs/symbol_store.md`.
2. Add `docs/symbols.md` as the UX companion document.
3. Treat `project_symbols` as the source of truth for rooted-symbol authoring.
4. Stop growing `Symbol Explorer` as if it must solve every symbol use case by itself.

### Phase 2: Split table from tree
1. Add a dedicated `Symbol Table` window backed by the existing `project_symbols list/create/rename/delete/update` lane.
2. Re-scope the existing `Symbol Explorer` into the hierarchical `Symbol Tree`.
3. Keep `Struct Viewer` as the shared detail pane for both windows.

This is the lowest-regret first product step because it improves clarity without demanding a model rewrite.

### Phase 3: Add a real Data Type Manager
1. Add a dedicated reusable-type window on top of the existing symbol/type registry.
2. Move reusable type management out of ad hoc symbol flows.
3. Give types first-class creation, search, edit, and conflict-handling UX.

This is the step that most directly closes the gap with Ghidra's "Data Type Manager" workflow.

### Phase 4: Move authoring closer to discovery
1. Add `Assign Symbol`, `Rename Symbol`, and `Apply Type` actions to `Code Viewer`.
2. Add equivalent address/selection actions to `Memory Viewer`.
3. Support `Create Type From Selection` where the data model is ready.
4. Keep these actions routed through the same symbol/type backends as the side panes.

This is the step that likely makes the product feel much less frustrating.

### Phase 5: Inline structure expansion and collapse
1. Teach the code/listing model to render structured data as expandable/collapsible ranges.
2. Start with safe exact-match rooted symbols and inline children.
3. Avoid trying to auto-expand huge pointer-derived graphs up front.

This should be deferred until the listing model is strong enough to own it cleanly.

## Design Rules For Squalr
- Do not replace the symbol-store backend plan with a Ghidra-shaped backend schema.
- Do not overload `Project Explorer` with long-term symbol management.
- Do not keep `Symbol Explorer` as both the authoritative table and the hierarchical browser.
- Do not make `Struct Viewer` the only place where symbol details can truly be edited.
- Do move symbol/type authoring closer to `Code Viewer` and `Memory Viewer`.
- Do keep rooted-symbol storage sparse and derived children lazy, as described in `docs/symbol_store.md`.
- Do treat Ghidra's separation of concerns as the main lesson, not its exact terminology.

## Proposed Immediate Next Steps
1. Add a `Symbol Table` design sketch and required columns/actions.
2. Rename the current `Symbol Explorer` conceptually in planning docs to `Symbol Tree`.
3. Define the minimum viable `Data Type Manager` scope for Squalr v1.
4. Add context-menu or toolbar entry points in `Code Viewer` for `Assign Symbol` and `Rename Symbol`.
5. Defer inline collapse/expand until after the symbol table/tree/type-manager split is in place.

## Bottom Line
Squalr does not appear to need a symbol-backend reset.

It does appear to need a clearer UX split:
- `Symbol Table` for maintenance.
- `Symbol Tree` for hierarchy and navigation.
- `Data Type Manager` for reusable types.
- `Code Viewer` and `Memory Viewer` for authoring at the point of discovery.
- `Struct Viewer` for focused detail editing.

That is the part of Ghidra we should emulate.
