# Symbols UX Plan

## Purpose
This document captures the symbol UX direction for Squalr.

Squalr is not modeling symbols as a flat global table. Symbols are a typed memory map: modules are visible roots, module contents are ordered fields, and reusable struct definitions describe the types placed into that memory.

## Window Model

### Symbol Tree
The Symbol Tree is the module struct tree.

Responsibilities:
- show manually added modules as top-level roots,
- expose a simple `+` action to add a module root,
- use the standard F2 mechanism for module and field rename,
- treat the module name as the resolver name,
- store each module root as only `module_name` plus `size`,
- show `u8[]` filler fields plus typed fields in module-offset order,
- expand fields into struct fields, arrays, and pointer targets lazily,
- support promote, retype, delete, and split flows,
- jump to memory, code, and details.

The tree should answer "what fields exist in this module struct?" and "what space is still raw `u8[]`?"

### SymbolStructEditor
The SymbolStructEditor window owns reusable type definitions.

Responsibilities:
- create and edit reusable layouts,
- rename types and fields,
- edit field containers and pointer sizes,
- show which fields use a layout,
- block or guide destructive type edits that would invalidate placed fields.

It should not own module occupancy. A reusable struct definition describes a type; the Symbol Tree places that type as a field inside a module struct.

### Details Viewer
The Details Viewer remains the focused inspector/editor for the selected field, derived child, or project item.

It is the place to inspect live values and edit writable fields, not the primary place to browse module layout.

### Project Explorer
The Project Explorer stays focused on acquisition and workflow artifacts:
- address items,
- pointer items,
- folders,
- scans and future workflow objects,
- promotion into symbol fields,
- conversion between acquisition items and symbol references when appropriate.

## Promotion UX

Promotion should feel like transforming bytes into typed data.

If promotion targets a module that is not yet in the Symbol Tree, promotion creates the module root first. If Squalr is attached and can query the module size, it stores that size and seeds the module root with one `u8[module_size]` filler field before splitting.

### Promoting Inside `u8[]` Bytes
When the selected range is inside a `u8[]` field, promotion splits that field around the new type.

```text
Before:
  u8_00100000: u8[0x100]

Promote +0x100020 as player_ptr: Player*(u64)

After:
  u8_00100000: u8[0x20]
  player_ptr: Player*(u64)
  u8_00100028: u8[0xD8]
```

### Promoting Over Typed Space
When promotion overlaps an existing typed field, the user needs a conflict flow:
- replace existing field,
- cancel,
- promote as a pointer/runtime target instead,
- split only if the existing field is `u8[]`.

Typed fields should not be silently broken apart.

### Promoting Derived Children
Promoting a struct field or pointer-derived child creates a real module field at that resolved location.

The derived child remains derivable from its parent. The promoted field becomes independently maintained in the module tree.

## Listing-Integrated Authoring
High-value authoring actions should be available directly in Memory Viewer and Code Viewer:
- `Place Symbol`,
- `Apply Type`,
- `Retype Field`,
- `Promote Selection`,
- `Jump to Symbol Tree`,
- `Rename Symbol`.

These actions should route through the same module-struct mutation engine as the Symbol Tree. There should not be separate viewer-only symbol mutation paths.

## Design Rules
- Do not expose `Rooted Symbol` as product terminology.
- Do treat modules as the visible roots of symbol space.
- Do make unclaimed module space visible as `u8[]` fields.
- Do make promotion reshape the typed memory map.
- Do keep reusable type authoring in SymbolStructEditor.
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
6. Make promotion split `u8[]` fields.
7. Add overlap/conflict tests before wiring the full UX.

## Bottom Line
Modules should be roots. The tree is empty until modules are added or promotion creates one. Each module root record is only `module_name` plus `size`; the address resolves live by name. Expanding a module shows its parent-struct contents: `u8[]` filler and typed fields. Promotion should create that root when needed, split filler, and turn raw bytes into meaningful data over time.
