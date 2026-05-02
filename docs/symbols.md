# Symbols UX Plan

## Purpose
This document captures the symbol UX direction for Squalr and turns it into a concrete product plan.

This is not a "copy Ghidra exactly" document. The goal is to understand which workflows should be separated, then adapt that separation to Squalr's dynamic-analysis-first model.

The important shift is: the Symbol Tree should not be a flat list of "rooted symbols." It should be a true module tree where module memory is gradually claimed, split, retyped, and expanded.

## Related Squalr Context
- `squalr/src/views/main_window/main_window_view.rs` treats `Project Explorer`, `Symbol Tree`, `Symbol Table`, `SymbolStructEditor`, `Details Viewer`, `Memory Viewer`, and `Code Viewer` as peer docked windows.
- `squalr/src/views/symbol_explorer/symbol_explorer_view.rs` is the current Symbol Tree implementation.
- `squalr/src/views/symbol_table/symbol_table_view.rs` is the current flat symbol maintenance implementation.
- `squalr/src/views/symbol_struct_editor/symbol_struct_editor_view.rs` owns reusable struct layout authoring.
- `squalr-engine-api/src/commands/project_symbols/project_symbols_command.rs` gives us a project-symbol command lane.
- `squalr-engine-api/src/structures/projects/project_root_symbol.rs` is the current persisted symbol-instance shape.
- `squalr-engine-domain/src/registries/symbols/symbol_registry.rs` acts as both a symbol/type registry and an execution-time resolver.

The architecture is usable, but the language and tree shape need to move from "rooted symbols" toward "module-owned typed claims."

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
- flat list of authored symbol claims,
- fast filtering by module, type, name, locator, and metadata,
- maintenance actions,
- jump to tree/memory/code/details.

### Symbol Tree
Ghidra's Symbol Tree is a hierarchical browsing and navigation surface.

For Squalr, this should be more physical: the tree should start with modules as address spaces, then show typed claims and unknown gaps inside those modules.

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
- the tree should answer "what has been claimed in this module?"
- it should also answer "what space is still unknown?"
- promoted discoveries should reshape the tree, not merely add another row.

### Data Type Manager / SymbolStructEditor
Ghidra's Data Type Manager owns reusable type definitions.

Squalr currently has a narrower but useful `SymbolStructEditor` window. That window should remain focused on reusable type authoring:
- create and edit structures,
- manage field names, offsets, data types, arrays, and pointers,
- handle type reuse and conflicts later.

It should not own module occupancy. A struct is a reusable definition; a symbol claim is a placed use of that definition.

### Listing and Viewer Integration
Ghidra does not force users to leave the listing every time they discover something.

Squalr should follow that lesson in Memory Viewer and Code Viewer:
- assign a symbol at the current address,
- apply a type to the current address or selection,
- retype an existing claim,
- promote a scan result or pointer target,
- jump from a byte/instruction selection to the corresponding Symbol Tree claim.

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
- the Symbol Tree still treats symbol instances as roots,
- the Symbol Table still speaks in rooted-symbol language,
- promotion currently reads like symbol creation instead of typed memory transformation,
- unknown module space is invisible,
- users cannot see what portions of a module have been claimed.

That makes symbols feel like bookmarks. They should feel like a growing typed model of the target process.

## Recommended Window Model

### 1. Symbol Tree
The Symbol Tree should be the module occupancy map.

Responsibilities:
- show modules as top-level roots,
- synthesize unknown `u8[]` gaps for unclaimed module space,
- show authored symbol claims in module-offset order,
- expand claims into struct fields, arrays, and pointer targets lazily,
- support promote/retype/delete/split flows,
- jump to memory/code/details.

User-facing copy should prefer `Symbol`, `Claim`, `Unknown Bytes`, and `Module`. Avoid `Rooted Symbol`.

### 2. Symbol Table
The Symbol Table should be the flat maintenance surface.

Responsibilities:
- list all authored symbol claims,
- filter by module, address, type, name, and metadata,
- bulk delete/rename/update where practical,
- show claim size and conflict status,
- jump to Symbol Tree, Memory Viewer, Code Viewer, and Details Viewer.

Unknown gaps should usually stay out of this table unless the user explicitly enables a module-map/debug view.

### 3. SymbolStructEditor
The SymbolStructEditor window should own reusable type definitions.

Responsibilities:
- create and edit reusable layouts,
- rename types and fields,
- edit field containers and pointer sizes,
- show which claims use a layout,
- block or guide destructive type edits that would invalidate placed claims.

### 4. Details Viewer
The Details Viewer should remain the focused inspector/editor for the selected claim, derived child, or project item.

It is the place to inspect live values and edit writable fields, not the primary place to browse module layout.

### 5. Project Explorer
The Project Explorer should stay focused on acquisition and workflow artifacts.

Responsibilities:
- address items,
- pointer items,
- folders,
- scans and future workflow objects,
- promotion into symbol claims,
- conversion between acquisition items and symbol references when appropriate.

## Promotion UX

Promotion should feel like transforming bytes into typed data.

### Promoting inside unknown bytes
When the selected range is inside an unknown `u8[]` chunk, promotion splits the unknown chunk around the new type.

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
When the selected chunk exactly matches an existing unknown claim, promotion replaces the chunk with the new type.

```text
Before:
  unknown_00123456: u8[0x40]

After:
  player_manager: PlayerManager
```

### Promoting over typed space
When promotion overlaps an existing typed claim, the user needs a conflict flow:
- replace existing claim,
- cancel,
- promote as a pointer/runtime target instead,
- or split only if the existing claim is unknown bytes.

Typed claims should not be silently broken apart.

### Promoting derived children
Promoting a struct field or pointer-derived child creates a real top-level claim at that resolved location.

The derived child remains derivable from its parent. The promoted claim becomes independently maintained and discoverable in the Symbol Table.

## Listing-Integrated Authoring
The highest-value authoring actions should be available directly in Memory Viewer and Code Viewer:
- `Place Symbol`,
- `Apply Type`,
- `Retype Claim`,
- `Promote Selection`,
- `Jump to Symbol Tree`,
- `Rename Symbol`.

These actions should route through the same claim engine as the Symbol Tree. There should not be separate viewer-only symbol mutation paths.

## Recommended Phasing

### Phase 1: Product language cleanup
1. Replace user-facing `Rooted Symbol` text with `Symbol` or `Symbol Claim`.
2. Keep compatibility names in Rust until the storage rename is worth doing.
3. Update CLI/TUI status text where it appears in user output.

### Phase 2: Module-grouped tree
1. Group existing module-relative symbols under module rows.
2. Group absolute symbols under `Absolute / Unmapped`.
3. Sort claims by module offset.
4. Keep existing derived child expansion under each claim.

### Phase 3: Unknown chunks
1. Use memory-query module sizes to synthesize unknown `u8[]` gaps.
2. Hide unknown chunks from Symbol Table by default.
3. Add tests for gap generation, ordering, and overlap detection.

### Phase 4: Claim-transform promotion
1. Route promotion through claim creation.
2. Split unknown chunks around promoted types.
3. Replace exact unknown chunks with promoted types.
4. Show conflict UX for overlaps with typed claims.

### Phase 5: Retype and delete semantics
1. Retype claims in place.
2. Recompute claim sizes from type layout.
3. Convert deleted claims back into unknown gaps.
4. Merge adjacent unknown gaps in the derived tree.

### Phase 6: Viewer authoring
1. Add `Place Symbol` and `Apply Type` to Memory Viewer.
2. Add matching actions to Code Viewer.
3. Add jump-to-tree from viewer selections.
4. Keep details editing in Details Viewer.

## Design Rules For Squalr
- Do not expose `Rooted Symbol` as product terminology.
- Do treat modules as the visible roots of symbol space.
- Do make unknown module space visible as `u8[]` chunks.
- Do make promotion reshape the typed memory map.
- Do keep reusable type authoring in SymbolStructEditor.
- Do keep Symbol Table flat and maintenance-oriented.
- Do keep derived children lazy unless explicitly promoted.
- Do not persist a giant module-sized struct just to render the tree.
- Do not silently split existing typed claims during promotion.

## Proposed Immediate Next Steps
1. Rename user-facing rooted-symbol copy.
2. Group Symbol Tree rows by module and absolute/unmapped space.
3. Add a module-claim/gap builder behind the Symbol Tree view data.
4. Make promotion call into a shared claim operation that can split unknown `u8[]` chunks.
5. Add overlap/conflict tests before wiring the full UX.

## Bottom Line
Squalr does not need a symbol-backend reset, but it does need a stronger symbol-tree model.

Modules should be roots. Unknown module bytes should be visible. Symbols should be typed claims over chunks of module memory. Promotion should split or reassign those chunks, turning raw `u8[]` space into meaningful data over time.
