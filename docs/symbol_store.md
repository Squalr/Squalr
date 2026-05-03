# Symbol Store Plan

## Purpose
This document is the working plan for turning project symbols into a real typed memory map.

The goal is not to copy Ghidra, Binary Ninja, or IDA. The goal is to give Squalr a practical symbol model that:
- treats modules as the visible roots of symbol space,
- models each module root expansion as one literal struct instance,
- lets promotion split `u8[]` filler into typed fields,
- expands through structs, arrays, and pointers,
- lets raw address and pointer discoveries reshape the typed module struct.

## Current Reality In The Repo
- `ProjectSymbolCatalog` stores reusable struct layouts and project symbol fields.
- `ProjectSymbolClaim` is the current persisted symbol-instance type.
- `ProjectSymbolLocator` currently supports absolute addresses and module-name-plus-offset locators.
- `SymbolicFieldDefinition` has field names and shared pointer-size/container encoding.
- Address and pointer project items can be promoted into project symbols.
- Virtual modules already have a good extension seam through `MemoryViewInstance`.
- Project symbols are authored on the unprivileged side, but the privileged registry refresh still republishes the merged symbol catalog wholesale.

The main gap is no longer "we have no symbol store." The gap is that the store and UI still talk like symbols are floating roots. The better model is: modules are roots, and each module expands like one parent struct whose fields gradually replace raw `u8[]` filler.

## Desired End State

### 1. Modules are symbol roots
The Symbol Tree should start from manually authored module roots, not from a flat list of "rooted symbols."

A module root represents a named parent struct:
- host modules such as `game.exe`,
- virtual modules such as `Dolphin MEM1`,
- synthetic or runtime-backed module-like spaces later.

The stored module root has exactly:
- `module_name`,
- `size`.

There is no module identifier, no stored base address, no path, no load state, and no metadata for the basic Symbol Tree add flow. The module name is the resolver key. The base address comes from the attached process at read/write time. If the user renames `winmine.exe`, that new name is what module-offset fields resolve against.

The size can usually be derived when attached, but it is still stored because target patches can change the runtime module size. The project should keep the authored size unless the user or a promotion flow explicitly updates it.

Modules should not appear automatically just because the user attached to a process. The Symbol Tree is empty by default. Users add module roots with the normal `+` action, rename them with the standard F2 mechanism, and expand them like any other tree struct.

Each module owns a struct layout over offsets `0..size`. Unclaimed space is represented as concrete `u8[]` fields inside that struct.

Example:

```text
Modules
  game.exe
    u8_00000000: u8[0x123456]
    player_manager: PlayerManager @ +0x123456
      local_player
      entity_list
    u8_001234A0: u8[0xCB60]
  Dolphin MEM1
    game_state: GameState @ +0x400000
```

This gives the tree a physical meaning: every child is a field inside the parent module struct.

### 2. Symbols are typed fields
A project symbol instance should be understood as a typed field in a module struct.

The product concept should become:
- field name,
- module offset,
- referenced type/layout,
- field byte range.

The byte range is derived from the symbol type when possible. For raw unclaimed bytes, the field is simply a `u8[]` range.

### 3. Unclaimed module space is real
Unclaimed `u8[]` space should not disappear from the tree.

When a module is first introduced manually, the user supplies the module name and size. When promotion creates a module root, Squalr should derive the size from the attached process virtual page and store that value. Either way, the module starts as one large `u8[]` chunk:

```text
game.exe
  bytes: u8[module_size]
```

As symbols are created, promoted, deleted, resized, or retyped, that `u8[]` chunk splits and merges around typed fields.

This makes the symbol tree feel like a gradually-filled memory map instead of a bag of bookmarks.

### 4. Promotion transforms the module struct
Promotion should not merely append another symbol record.

If the target module root does not exist yet, promotion creates it. For example, if the user attached to `winmine.exe`, scanned, added a static address to the project, and then promoted it, promotion should create the `winmine.exe` module struct, query and store the module size, seed it with `u8[module_size]`, and then split that filler around the promoted field.

If the promoted address falls inside a `u8[]` field, promotion splits that field:

```text
Before:
  u8_00001000: u8[0x100]

Promote +0x104 as ptr64 Player*:
  u8_00001000: u8[0x4]
  player_ptr: Player*(u64)
  u8_0000100C: u8[0xF4]
```

If the promoted address exactly matches an existing `u8[]` field, promotion can replace that field in place:

```text
Before:
  u8_00123456: u8[0x40]

Promote +0x123456 as PlayerManager:
  PlayerManager: PlayerManager
```

If the new field overlaps an existing typed field, the user needs an explicit conflict flow:
- replace the old field,
- split the old field if the old field is splittable `u8[]`,
- reject the promotion,
- or create a separate pointer/runtime symbol if the address is not actually static module layout.

### 5. Struct fields are derived children
Struct fields, fixed-array elements, and pointer dereferences should remain derived tree nodes.

Persist the top-level module fields. Derive children lazily from:
- the module name plus field offset,
- the referenced type layout,
- container semantics,
- pointer reads when the user expands pointer nodes.

Do not persist a giant child graph just because the UI can display one.

### 6. Address and pointer items remain acquisition tools
`AddressItem` and `PointerItem` should stay.

They are discovery tools. Promotion is the bridge from discovery into the typed module map:
- an address item promotes to a module field or absolute field,
- a pointer item can promote either the pointer slot itself or the current pointed-to target,
- pointer provenance can remain on acquisition/project-item state when useful.

After promotion, project items can reference the symbol field instead of continuing to own long-term layout identity.

### 7. Modules stay name-backed for now
Do not introduce a new `ModuleId` abstraction.

The current memory layer already resolves modules by name, including virtual-module style sources. We can build the first module tree using module names as they exist today.

If module identity becomes painful later, solve that later. Do not add an identifier to the Symbol Tree module root now.

### 8. Pointer encoding reuses the pointer-scan model
Do not build a fresh generic pointer plugin system first.

Symbolic containers should continue to reuse the existing pointer-scan pointer-size model, including unusual encodings such as `u24be`.

### 9. The privileged side gets only execution data
The full authored symbol map should remain unprivileged-owned.

The privileged side should receive only what is needed for:
- typed reads and writes,
- resolving active project items,
- pointer expansion for active symbol views,
- cheap address labeling.

The current whole-catalog registry sync is acceptable as a temporary bridge, but not as the intended end state for a large typed module map.

## Core Model

### Symbol types
Reusable type definitions should continue to describe structure:
- project-local stable key,
- display name,
- ordered named fields,
- field type/container information.

These are reusable layouts, not placed fields over memory by themselves.

### Module root records
Add a module-oriented root record over the existing catalog.

The persisted root record contains only:
- `module_name`,
- `size`.

Everything else is expansion content under that root. Typed fields and `u8[]` filler are children in the module struct tree, not extra identifying fields on the module root record.

The module root itself contains no stored address. `module_name` resolves to the current base address through the active memory view. Stored fields are offsets inside `size`.

### Symbol fields
A symbol field is the long-term replacement concept for "rooted symbol."

It should contain:
- field name,
- module offset,
- referenced symbol type,
- field size or size policy.

In the current code this is represented by `ProjectSymbolClaim`, but the product concept should be "field in a module struct."

### Locators
The normal field locator is module name + offset.

Module-relative fields are the normal case. Absolute-address symbols can render under an `Absolute / Unmapped` group, but they should not drive the main UX.

### Field size policy
Most field sizes come from the referenced type:
- primitive: primitive size,
- fixed array: element size times length,
- struct: sum of fields,
- pointer: pointer slot size.

Some fields need an explicit size:
- raw unclaimed bytes,
- dynamic arrays.

Start with explicit sizes only where the type system cannot derive one.

## Struct Operations

### Add module
Adding a module should be as simple as pressing `+` in the Symbol Tree.

The new root has exactly:
1. a module name,
2. a size.

Rename uses the standard F2 tree rename flow. The name is the resolver name.

### Promote field
Promoting a static address should:
1. resolve the module name and module offset,
2. create the module struct if it does not exist,
3. query and store the module size when the module struct is created from a live process,
4. create an initial `u8[]` filler field from the stored size,
5. split the filler around the new typed field,
6. insert the new typed field in offset order.

### Retype field
Retyping a field changes its type and recomputes its range.

If the new range is smaller, the trailing space becomes a `u8[]` gap.
If the new range is larger, conflict handling is required for the newly covered range.

### Delete field
Deleting a typed field should return its bytes to `u8[]` space.

Adjacent `u8[]` chunks should merge.

### Promote discovery
Promotion should route through the same module-struct mutation machinery.

It is not a separate storage path. It is a convenient entry point from a discovered address, pointer, scan result, memory selection, or derived child.

## What We Are Explicitly Not Building Yet
- No full opaque `SymbolId` architecture with UUID-heavy plumbing.
- No separate `ModuleId` system.
- No stored module base address.
- No stored module path/hash/load state for the basic tree add flow.
- No persistent derived child graph.
- No exporter/plugin framework redesign.
- No global reverse address-to-symbol pointer map.
- No replacement of all project item types with symbols immediately.
- No static-symbol candidate database.
- No attach-time module population.
- No import workflow before the manual module tree exists.

## Implementation Plan

### Sprint 1: Rename the product model
1. Update docs and UI language from "rooted symbol" to "symbol field" or plain "symbol."
2. Keep Rust compatibility names until the data model is ready to move.
3. Make module grouping the visible shape of the Symbol Tree.
4. Put absolute-address symbols under an `Absolute / Unmapped` group.

### Sprint 2: Make module roots editable
1. Add `+` module creation to the Symbol Tree.
2. Add standard F2 rename for module roots.
3. Store module roots as only `module_name` plus `size`.
4. Keep empty projects empty until a module is added or promotion creates one.

### Sprint 3: Make promotion reshape module structs
1. Promote static address selections by creating the module struct when missing.
2. Query module size during promotion when the process is attached.
3. Seed the module with `u8[module_size]`.
4. Split `u8[]` filler around promoted typed fields.
5. Add overwrite/split/reject conflict UX only for typed overlaps.

### Sprint 4: Support retyping and deletion
1. Retype existing fields in place.
2. Recompute field ranges after type changes.
3. Return deleted fields to `u8[]` space.
4. Merge adjacent `u8[]` gaps in the derived view.

### Sprint 5: Keep derived children lazy
1. Expand struct fields from field type layouts.
2. Expand fixed arrays on demand.
3. Resolve pointer children on demand.
4. Promote derived children into module fields only when explicitly requested.

### Sprint 6: Clean up the transport boundary
1. Stop treating the privileged registry catalog as the permanent transport for the full authored symbol map.
2. Move toward sending compact execution-oriented symbol data instead of the entire authored model.
3. Keep authoring-only tree state unprivileged-side.

## GUI Shape

### Symbol Tree
The Symbol Tree should be the module struct editor for placed symbols.

Responsibilities:
- show manually added modules as top-level roots,
- expose a `+` action to add a module,
- store each module root as only module name plus size,
- support F2 rename for module roots and fields,
- show ordered typed fields and `u8[]` filler fields,
- lazily expand struct, array, and pointer children,
- promote `u8[]` chunks or derived children into typed fields,
- retype, resize, rename, and delete fields,
- jump fields to Memory Viewer or Code Viewer.

The tree is not just a namespace browser. It is the literal parent struct for each module.

### SymbolStructEditor
The SymbolStructEditor window owns reusable layout authoring.

It should not own module occupancy. It defines what a field means once placed.

### Memory Viewer and Code Viewer
Viewer-side symbol actions should feed the same module-field operations:
- assign symbol at current address,
- apply type to current address or selection,
- retype existing field,
- promote selection into a field,
- jump to the matching Symbol Tree row.

The viewers are where discoveries happen. The Symbol Tree is where those discoveries become durable layout.

### Project Explorer
The Project Explorer remains focused on acquisition and workflow artifacts.

Address and pointer items can promote into module fields, then optionally become symbol references.

## Practical Design Decisions

### Identity
Module roots do not have separate identities.

For a Symbol Tree module root:
- `module_name` is the resolver name,
- `size` is the authored module size,
- the current base address is resolved live from the attached process.

Stored symbol fields live inside that module by offset. Do not add a module UUID or stored address to make this feel more database-like.

### Naming
Avoid exposing "rooted symbol" in UI copy.

Use:
- `Symbol` for normal user-facing rows and actions,
- `Symbol Field` when discussing occupied module ranges,
- `Module` for top-level symbol tree roots,
- `u8[]` for unclaimed gaps.

### Children
Children are derived views, not stored facts, unless the user explicitly promotes one into a module field.

### Project items versus symbols
Symbols and project items should not collapse into one concept.

For now:
- address items and pointer items are workflow and acquisition objects,
- symbol fields are authored typed memory ranges,
- promotion is the bridge from the former to the latter.

### External tools
If an import/export plugin later needs extra naming hints, original-source names, namespace mappings, comments, or tags, keep that outside the core module-root shape.

### Managed runtimes
If we later want strong support for CLR-heavy, JVM-heavy, or other managed-runtime targets, the current plan should extend by adding a runtime-aware symbol provider layer rather than reshaping the whole core model.

The intended shape is:
- virtual modules still describe the visible memory space,
- data-type plugins still describe value encodings,
- a future runtime/symbol-provider plugin can contribute runtime-specific modules, fields, and resolution logic.

## Bottom Line
The symbol-store plan should move from "sparse rooted symbols" to "module roots as literal structs."

Modules are the roots. The Symbol Tree starts empty until the user adds a module or promotion creates one. Each module root record is only `module_name` plus `size`; the address resolves at runtime by name. Expanding that root shows the module struct contents: `u8[]` filler and typed fields. Promotion creates the module root when needed, queries and stores the size when possible, and splits filler into meaningful data.
