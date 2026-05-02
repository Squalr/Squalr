# Symbol Store Plan

## Purpose
This document is the working plan for turning project symbols into a real typed memory map.

The goal is not to copy Ghidra, Binary Ninja, or IDA. The goal is to give Squalr a practical symbol model that:
- treats modules as the visible roots of symbol space,
- lets users claim and progressively type chunks of module memory,
- expands through structs, arrays, and pointers,
- keeps external-tool specifics in metadata instead of core types,
- lets raw address and pointer discoveries reshape the typed memory map.

## Current Reality In The Repo
- `ProjectSymbolCatalog` stores reusable struct layouts and project symbol instances.
- `ProjectRootSymbol` is the current persisted symbol-instance type.
- `ProjectRootSymbolLocator` currently supports absolute addresses and module-name-plus-offset locators.
- `SymbolicFieldDefinition` has field names and shared pointer-size/container encoding.
- Address and pointer project items can be promoted into project symbols.
- Virtual modules already have a good extension seam through `MemoryViewInstance`.
- Project symbols are authored on the unprivileged side, but the privileged registry refresh still republishes the merged symbol catalog wholesale.

The main gap is no longer "we have no symbol store." The gap is that the store and UI still talk like symbols are floating roots. The better model is: modules are roots, and symbols are typed claims inside module memory.

## Desired End State

### 1. Modules are symbol roots
The Symbol Tree should start from modules, not from a flat list of "rooted symbols."

A module is a claimable address space:
- host modules such as `game.exe`,
- virtual modules such as `Dolphin MEM1`,
- synthetic or runtime-backed module-like spaces later.

Each module owns a typed layout map over its address range. Unknown space is still represented explicitly, usually as coarse `u8[]` chunks.

Example:

```text
Modules
  game.exe
    unknown_00000000: u8[0x123456]
    PlayerManager: PlayerManager @ +0x123456
      local_player
      entity_list
    unknown_001234A0: u8[0xCB60]
  Dolphin MEM1
    GameState: GameState @ +0x80400000
```

This gives the tree a physical meaning: every top-level child claims bytes from a parent module space.

### 2. Symbols are typed claims
A project symbol instance should be understood as a typed claim over a range.

The current `ProjectRootSymbol` type can remain as a compatibility bridge, but the product concept should become:
- stable symbol key,
- display name,
- module or absolute locator,
- referenced type/layout,
- claimed byte range,
- optional metadata.

The claimed byte range is derived from the symbol type when possible. For dynamically sized arrays or unresolved types, the claim can use an explicit length or stay conservative.

### 3. Unclaimed module space is real
Unknown space should not disappear from the tree.

When a module is first introduced into the symbol tree, the default representation can be one large unknown chunk:

```text
game.exe
  bytes: u8[module_size]
```

As symbols are created, promoted, deleted, resized, or retyped, that unknown chunk splits and merges around the typed claims.

This makes the symbol tree feel like a gradually-filled memory map instead of a bag of bookmarks.

### 4. Promotion transforms chunks
Promotion should not merely append another symbol record.

If the promoted address falls inside an unknown `u8[]` claim, promotion splits that claim:

```text
Before:
  unknown_00001000: u8[0x100]

Promote +0x104 as ptr64 Player*:
  unknown_00001000: u8[0x4]
  player_ptr: Player*(u64)
  unknown_0000100C: u8[0xF4]
```

If the promoted address exactly matches an existing static claim, promotion can reassign that claim in place:

```text
Before:
  unknown_00123456: u8[0x40]

Promote +0x123456 as PlayerManager:
  PlayerManager: PlayerManager
```

If the new claim overlaps an existing typed claim, the user needs an explicit conflict flow:
- replace the old claim,
- split the old claim if the old claim is splittable unknown bytes,
- reject the promotion,
- or create a separate pointer/runtime symbol if the address is not actually static module layout.

### 5. Struct fields are derived children
Struct fields, fixed-array elements, and pointer dereferences should remain derived tree nodes.

Persist the top-level typed claims. Derive children lazily from:
- the claim locator,
- the referenced type layout,
- container semantics,
- pointer reads when the user expands pointer nodes.

Do not persist a giant child graph just because the UI can display one.

### 6. Address and pointer items remain acquisition tools
`AddressItem` and `PointerItem` should stay.

They are discovery tools. Promotion is the bridge from discovery into the typed module map:
- an address item promotes to a static module or absolute claim,
- a pointer item can promote either the pointer slot itself or the current pointed-to target,
- pointer provenance stays in metadata when useful.

After promotion, project items can reference the symbol claim instead of continuing to own long-term layout identity.

### 7. Modules stay name-backed for now
Do not introduce a new `ModuleId` abstraction yet.

The current memory layer already resolves modules by name, including virtual-module style sources. We can build the first claim-based tree using module names as they exist today.

If module identity becomes painful later, add normalization then.

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

These are reusable layouts, not claims over memory by themselves.

### Module symbol maps
Add a module-oriented symbol map concept over the existing catalog.

Conceptually, each module map contains ordered claims:
- `module_name`,
- `module_size` when known,
- ordered `SymbolClaim` records,
- synthesized unknown gaps.

Unknown gaps do not need to be persisted as full records if they can be derived from module size and existing claims. It is fine to synthesize them in the tree.

### Symbol claims
A symbol claim is the long-term replacement concept for "rooted symbol."

It should contain:
- stable symbol key,
- display name,
- locator,
- referenced symbol type,
- claim size or size policy,
- optional metadata.

For compatibility, this can initially be implemented by extending or renaming `ProjectRootSymbol`.

### Locators
The first claim locators should remain:
- absolute address,
- module name + offset.

Module-relative claims are the normal case. Absolute-address claims should render under an `Absolute / Unmapped` group, not as peers of real modules.

### Claim size policy
Most claim sizes come from the referenced type:
- primitive: primitive size,
- fixed array: element size times length,
- struct: sum of fields,
- pointer: pointer slot size.

Some cases need an explicit size:
- raw unknown bytes,
- dynamic arrays,
- imported ranges whose exact type is not known yet.

Start with explicit sizes only where the type system cannot derive one.

### Metadata
Metadata should stay small and optional.

Start with a simple extension map for:
- import/export hints,
- comments,
- aliases if needed later,
- promotion provenance.

Do not build a typed metadata framework up front.

## Claim Operations

### Create claim
Creating a claim at a module offset should:
1. compute the new claim range,
2. find overlapping existing claims,
3. split unknown byte claims when possible,
4. require confirmation for replacing typed claims,
5. insert the new typed claim into module order.

### Retype claim
Retyping a claim changes its type and recomputes its range.

If the new range is smaller, the trailing space becomes an unknown `u8[]` gap.
If the new range is larger, conflict handling is required for the newly covered range.

### Delete claim
Deleting a typed claim should return its bytes to unknown space.

Adjacent unknown byte chunks should merge.

### Promote discovery
Promotion should route through the same create/retype machinery.

It is not a separate storage path. It is a convenient entry point from a discovered address, pointer, scan result, memory selection, or derived child.

## What We Are Explicitly Not Building Yet
- No full opaque `SymbolId` architecture with UUID-heavy plumbing.
- No separate `ModuleId` system.
- No persistent derived child graph.
- No exporter/plugin framework redesign.
- No global reverse address-to-symbol pointer map.
- No replacement of all project item types with symbols immediately.
- No literal mega-struct allocation for every module in persisted project files.

## Implementation Plan

### Sprint 1: Rename the product model
1. Update docs and UI language from "rooted symbol" to "symbol claim" or plain "symbol."
2. Keep Rust compatibility names until the data model is ready to move.
3. Make module grouping the visible shape of the Symbol Tree.
4. Put absolute-address symbols under an `Absolute / Unmapped` group.

### Sprint 2: Add claim-range semantics
1. Add claim size calculation for existing symbol instances.
2. Synthesize unknown `u8[]` gaps from module size and existing claims.
3. Sort module claims by offset.
4. Detect overlaps and expose conflict information.

### Sprint 3: Make promotion reshape module space
1. Promote address selections by splitting unknown chunks.
2. Promote scan/address project items through the same claim-creation path.
3. Promote pointer items as either pointer-slot claims or pointed-target claims.
4. Add overwrite/split/reject conflict UX.

### Sprint 4: Support retyping and deletion
1. Retype existing claims in place.
2. Recompute claim ranges after type changes.
3. Return deleted claims to unknown space.
4. Merge adjacent unknown gaps in the derived view.

### Sprint 5: Keep derived children lazy
1. Expand struct fields from claim type layouts.
2. Expand fixed arrays on demand.
3. Resolve pointer children on demand.
4. Promote derived children into top-level claims only when explicitly requested.

### Sprint 6: Clean up the transport boundary
1. Stop treating the privileged registry catalog as the permanent transport for the full authored symbol map.
2. Move toward sending compact execution-oriented symbol data instead of the entire authored model.
3. Keep authoring-only metadata and tree state unprivileged-side.

## GUI Shape

### Symbol Tree
The Symbol Tree should be a module memory map.

Responsibilities:
- show modules as top-level roots,
- show ordered typed claims and synthesized unknown gaps,
- lazily expand struct, array, and pointer children,
- promote unknown chunks or derived children into typed claims,
- retype, resize, rename, and delete claims,
- jump claims to Memory Viewer or Code Viewer.

The tree is not just a namespace browser. It is the visible typed occupancy map for a module.

### Symbol Table
The Symbol Table should remain the flat maintenance surface.

Responsibilities:
- list all user-authored symbol claims,
- filter by name, module, locator, type, source, and metadata,
- bulk rename/delete/update where practical,
- show claim size and overlap/conflict status,
- jump to the corresponding Symbol Tree row.

Unknown synthesized gaps do not need to appear in the table by default.

### Symbol Structs
The Symbol Structs window owns reusable layout authoring.

It should not own module occupancy. It defines what a claim means once placed.

### Memory Viewer and Code Viewer
Viewer-side symbol actions should feed the same claim operations:
- assign symbol at current address,
- apply type to current address or selection,
- retype existing claim,
- promote selection into a claim,
- jump to the matching Symbol Tree row.

The viewers are where discoveries happen. The Symbol Tree is where those discoveries become durable layout.

### Project Explorer
The Project Explorer remains focused on acquisition and workflow artifacts.

Address and pointer items can promote into claims, then optionally become symbol references.

## Practical Design Decisions

### Identity
Use project-local stable keys for stored types and stored symbol claims.

Do not use the user-visible display name as the only identity, because rename support and references from project items need something stable.

### Naming
Avoid exposing "rooted symbol" in UI copy.

Use:
- `Symbol` for normal user-facing rows and actions,
- `Symbol Claim` when discussing occupied module ranges,
- `Module` for top-level symbol tree roots,
- `Unknown Bytes` or `u8[]` for unclaimed gaps.

### Children
Children are derived views, not stored facts, unless the user explicitly promotes one into a claim.

### Project items versus symbols
Symbols and project items should not collapse into one concept.

For now:
- address items and pointer items are workflow and acquisition objects,
- symbol claims are authored typed memory ranges,
- promotion is the bridge from the former to the latter.

### External tools
If an import/export plugin needs extra naming hints, original-source names, namespace mappings, comments, or tags, those belong in metadata associated with that plugin or format.

### Managed runtimes
If we later want strong support for CLR-heavy, JVM-heavy, or other managed-runtime targets, the current plan should extend by adding a runtime-aware symbol provider layer rather than reshaping the whole core model.

The intended shape is:
- virtual modules still describe the visible memory space,
- data-type plugins still describe value encodings,
- a future runtime/symbol-provider plugin can contribute runtime-specific modules, claims, and resolution logic.

## Bottom Line
The symbol-store plan should move from "sparse rooted symbols" to "typed claims inside modules."

Modules are the roots. Unknown module space is explicit. Promotion is a chunk split/retype operation. Struct fields and pointer targets stay lazily derived unless the user promotes them into real claims.
