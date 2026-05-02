# Symbols UX Plan

## Purpose
This document captures the symbol UX direction for Squalr and turns it into a concrete product plan.

This is not a "copy Ghidra exactly" document. The goal is to understand which workflows should be separated, then adapt that separation to Squalr's dynamic-analysis-first model.

The important shift is: the Symbol Tree should not be a flat list of "rooted symbols." It should be a literal module tree where each module root record is just `module_name` plus `size`, and expanding that root shows the parent-struct contents: `u8[]` filler and typed fields.

## Related Squalr Context
- `squalr/src/views/main_window/main_window_view.rs` treats `Project Explorer`, `Symbol Tree`, `Symbol Table`, `SymbolStructEditor`, `Details Viewer`, `Memory Viewer`, and `Code Viewer` as peer docked windows.
- `squalr/src/views/symbol_explorer/symbol_explorer_view.rs` is the current Symbol Tree implementation.
- `squalr/src/views/symbol_table/symbol_table_view.rs` is the current flat symbol maintenance implementation.
- `squalr/src/views/symbol_struct_editor/symbol_struct_editor_view.rs` owns reusable struct layout authoring.
- `squalr-engine-api/src/commands/project_symbols/project_symbols_command.rs` gives us a project-symbol command lane.
- `squalr-engine-api/src/structures/projects/project_root_symbol.rs` is the current persisted symbol-instance shape.
- `squalr-engine-domain/src/registries/symbols/symbol_registry.rs` acts as both a symbol/type registry and an execution-time resolver.

The architecture is usable, but the language and tree shape need to move from "rooted symbols" toward "module roots as editable structs."

## What Ghidra Actually Separates

### Symbol Table
Ghidra's Symbol Table is the authoritative flat maintenance surface, not the hierarchical browser.

Use cases it covers:
- browse all symbols in one place,
- sort and filter by source, symbol type, and advanced criteria,
- see symbol attributes like address, type, namespace, and references,
- perform direct maintenance actions like rename, delete, pin, and selection-driven navigation,
- view symbols that are awkward or impossible to understand from a tree alone.

Squalr equivalent:
- optional flat list of authored symbol fields,
- fast filtering by module, type, name, and offset,
- maintenance actions,
- jump to tree/memory/code/details.

The Symbol Table should not become a static-import staging system. Static symbols should become real tree fields only when the user promotes or imports them into a module struct.

### Symbol Tree
Ghidra's Symbol Tree is a hierarchical browsing and navigation surface.

For Squalr, this should be more physical: the tree should start with manually authored modules, then expand each module root like one struct.

Squalr equivalent:

```text
Modules
  game.exe
    unknown_00000000: u8[0x1200]
    player_manager: PlayerManager @ +0x1200
      local_player
      entity_count
    unknown_00001240: u8[0x200]
  Dolphin MEM1
    game_state: GameState @ +0x80400000
Absolute / Unmapped
  scratch_object: DebugStruct @ 0x7FF612340000
```

Important takeaway:
- the tree should answer "what fields exist in this module struct?"
- it should also answer "what space is still raw `u8[]`?"
- promoted discoveries should reshape the tree, not merely add another row.

Attach should not automatically fill this tree. A fresh project attached to `winmine.exe` can still show no module roots. The user can add `winmine.exe` with `+`, or promotion from a static address can create that module root on demand.

A module root has only `module_name` and `size`. There is no separate module id and no stored base address. The module name resolves to the current base address through the attached process.

### Data Type Manager / SymbolStructEditor
Ghidra's Data Type Manager owns reusable type definitions.

Squalr currently has a narrower but useful `SymbolStructEditor` window. That window should remain focused on reusable type authoring:
- create and edit structures,
- manage field names, offsets, data types, arrays, and pointers,
- handle type reuse and conflicts later.

It should not own module occupancy. A reusable struct definition describes a type; the Symbol Tree places that type as a field inside a module struct.

### Listing and Viewer Integration
Ghidra does not force users to leave the listing every time they discover something.

Squalr should follow that lesson in Memory Viewer and Code Viewer:
- assign a symbol at the current address,
- apply a type to the current address or selection,
- retype an existing field,
- promote a scan result or pointer target,
- jump from a byte/instruction selection to the corresponding Symbol Tree field.

Discovery should happen where the user is looking. Durable organization should happen in the symbol/type windows.

## Sources Reviewed
- Symbol Table: <https://www.ghidradocs.com/9.2_PUBLIC/help/Base/help/topics/SymbolTablePlugin/symbol_table.htm>
- Symbol Tree: <https://www.ghidradocs.com/9.2.3_PUBLIC/help/Base/help/topics/SymbolTreePlugin/SymbolTree.htm>
- Data Type Manager overview: <https://www.ghidradocs.com/9.1_PUBLIC/help/Base/help/topics/DataTypeManagerPlugin/data_type_manager_description.htm>
- Data Type Manager window: <https://www.ghidradocs.com/10.3_PUBLIC/help/Base/help/topics/DataTypeManagerPlugin/data_type_manager_window.html>
- Code Browser: <https://www.ghidradocs.com/11.0_PUBLIC/help/Base/help/topics/CodeBrowserPlugin/CodeBrowser.htm>
- Data plugin: <https://www.ghidradocs.com/11.2_PUBLIC/help/Base/help/topics/DataPlugin/Data.htm>
- Advanced class notes: <https://ghidradocs.com/12.0.1_PUBLIC/docs/GhidraClass/Advanced/improvingDisassemblyAndDecompilation.pdf>

## Where Squalr Feels Frustrating Today
Squalr has already split several windows, but the concept boundary is still fuzzy:
- the Symbol Tree still treats symbol instances as roots instead of module structs,
- the Symbol Table still speaks in rooted-symbol language,
- promotion currently reads like symbol creation instead of typed memory transformation,
- unknown module space is invisible,
- users cannot see what portions of a module are typed fields versus raw `u8[]`.

That makes symbols feel like bookmarks. They should feel like a growing typed model of the target process.

## Recommended Window Model

### 1. Symbol Tree
The Symbol Tree should be the module struct tree.

Responsibilities:
- show manually added modules as top-level roots,
- expose a simple `+` action to add a module root,
- use the standard F2 mechanism for module and field rename,
- treat the module name as the resolver name,
- store each module root as only `module_name` plus `size`,
- show `u8[]` filler fields plus typed fields in module-offset order,
- expand fields into struct fields, arrays, and pointer targets lazily,
- support promote/retype/delete/split flows,
- jump to memory/code/details.

User-facing copy should prefer `Symbol`, `Field`, `Unknown Bytes`, and `Module`. Avoid `Rooted Symbol`.

### 2. Symbol Table
The Symbol Table should be secondary to the tree. It can remain a flat maintenance surface for authored fields.

Responsibilities:
- list all authored symbol fields,
- filter by module, offset, type, and name,
- bulk delete/rename/update where practical,
- show field size and conflict status,
- jump to Symbol Tree, Memory Viewer, Code Viewer, and Details Viewer.

Unknown gaps should usually stay out of this table unless the user explicitly enables a module-map/debug view.

Do not add a separate `Static Candidates` mode. That is too much product surface for now. Imports and promotions should mutate the module struct directly.

### 3. SymbolStructEditor
The SymbolStructEditor window should own reusable type definitions.

Responsibilities:
- create and edit reusable layouts,
- rename types and fields,
- edit field containers and pointer sizes,
- show which fields use a layout,
- block or guide destructive type edits that would invalidate placed fields.

### 4. Details Viewer
The Details Viewer should remain the focused inspector/editor for the selected field, derived child, or project item.

It is the place to inspect live values and edit writable fields, not the primary place to browse module layout.

### 5. Project Explorer
The Project Explorer should stay focused on acquisition and workflow artifacts.

Responsibilities:
- address items,
- pointer items,
- folders,
- scans and future workflow objects,
- promotion into symbol fields,
- conversion between acquisition items and symbol references when appropriate.

## Promotion UX

Promotion should feel like transforming bytes into typed data.

If promotion targets a module that is not yet in the Symbol Tree, promotion creates the module root first. If Squalr is attached and can query the module size, it stores that size and seeds the module root with one `u8[module_size]` filler field before splitting.

### Promoting inside unknown bytes
When the selected range is inside an unknown `u8[]` field, promotion splits that field around the new type.

```text
Before:
  unknown_00100000: u8[0x100]

Promote +0x100020 as player_ptr: Player*(u64)

After:
  unknown_00100000: u8[0x20]
  player_ptr: Player*(u64)
  unknown_00100028: u8[0xD8]
```

### Promoting an exact static chunk
When the selected chunk exactly matches an existing unknown field, promotion replaces the field with the new type.

```text
Before:
  unknown_00123456: u8[0x40]

After:
  player_manager: PlayerManager
```

### Promoting over typed space
When promotion overlaps an existing typed field, the user needs a conflict flow:
- replace existing field,
- cancel,
- promote as a pointer/runtime target instead,
- or split only if the existing field is unknown bytes.

Typed fields should not be silently broken apart.

### Promoting derived children
Promoting a struct field or pointer-derived child creates a real module field at that resolved location.

The derived child remains derivable from its parent. The promoted field becomes independently maintained and discoverable in the Symbol Table.

## Listing-Integrated Authoring
The highest-value authoring actions should be available directly in Memory Viewer and Code Viewer:
- `Place Symbol`,
- `Apply Type`,
- `Retype Field`,
- `Promote Selection`,
- `Jump to Symbol Tree`,
- `Rename Symbol`.

These actions should route through the same module-struct mutation engine as the Symbol Tree. There should not be separate viewer-only symbol mutation paths.

## Recommended Phasing

### Phase 1: Product language cleanup
1. Replace user-facing `Rooted Symbol` text with `Symbol` or `Symbol Field`.
2. Keep compatibility names in Rust until the storage rename is worth doing.
3. Update CLI/TUI status text where it appears in user output.

### Phase 2: Editable module roots
1. Add a `+` action to create module roots.
2. Support F2 rename using the existing tree rename mechanics.
3. Treat module name as the module resolver name.
4. Store each module root as only `module_name` plus `size`.
5. Keep fresh attached projects empty until a module is added or promotion creates one.

### Phase 3: Module struct fields
1. Store ordered fields under each module root.
2. Represent untyped ranges as `u8[]` fields.
3. Hide `u8[]` filler from Symbol Table by default.
4. Add tests for field ordering, split behavior, and overlap detection.

### Phase 4: Promotion creates and mutates module structs
1. Route static promotion through module struct mutation.
2. Create the module root if it does not exist.
3. Query and store module size, then seed `u8[module_size]` when attached.
4. Split unknown fields around promoted types.
5. Replace exact unknown fields with promoted types.
6. Show conflict UX for overlaps with typed fields.

### Phase 5: Retype and delete semantics
1. Retype fields in place.
2. Recompute field sizes from type layout.
3. Convert deleted fields back into unknown gaps.
4. Merge adjacent unknown fields in the tree.

### Phase 6: Viewer authoring
1. Add `Place Symbol` and `Apply Type` to Memory Viewer.
2. Add matching actions to Code Viewer.
3. Add jump-to-tree from viewer selections.
4. Keep details editing in Details Viewer.

## Design Rules For Squalr
- Do not expose `Rooted Symbol` as product terminology.
- Do treat modules as the visible roots of symbol space.
- Do make unknown module space visible as `u8[]` fields.
- Do make promotion reshape the typed memory map.
- Do keep reusable type authoring in SymbolStructEditor.
- Do keep Symbol Table flat, secondary, and maintenance-oriented.
- Do keep derived children lazy unless explicitly promoted.
- Do keep module roots to only `module_name` plus `size`.
- Do resolve module base addresses live by name.
- Do not populate modules automatically on attach.
- Do not add module ids, stored module addresses, paths, hashes, or load-state fields to the basic Symbol Tree module root.
- Do not add a static-candidate workflow before the module tree exists.
- Do not silently split existing typed fields during promotion.

## Proposed Immediate Next Steps
1. Rename user-facing rooted-symbol copy.
2. Add manual module-root creation to Symbol Tree.
3. Add F2 rename for module roots using the standard rename mechanism.
4. Store module roots as `module_name` plus `size`.
5. Make promotion create a missing module root and seed `u8[module_size]` when runtime size is known.
6. Make promotion split unknown `u8[]` fields.
7. Add overlap/conflict tests before wiring the full UX.

## Bottom Line
Squalr does not need a symbol-backend reset, but it does need a simpler symbol-tree model.

Modules should be roots. The tree is empty until modules are added or promotion creates one. Each module root record is only `module_name` plus `size`; the address resolves live by name. Expanding a module shows its parent-struct contents: `u8[]` filler and typed fields. Promotion should create that root when needed, split filler, and turn raw bytes into meaningful data over time.
