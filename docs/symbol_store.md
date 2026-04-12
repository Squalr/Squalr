# Symbol Store Plan

## Purpose
This document is the working plan for turning project symbols into a real symbol store over the next few sprints.

The goal is not to copy Ghidra, Binary Ninja, or IDA. The goal is to give Squalr a practical symbol model that:
- fits the current project and IPC architecture,
- supports static and virtual modules,
- expands through structs, arrays, and pointers,
- keeps external-tool specifics in metadata instead of core types,
- lets raw address and pointer discoveries become durable project symbols.

## Current Reality In The Repo
- `ProjectSymbolCatalog` only stores reusable layouts. It does not store rooted symbols.
- `StructLayoutDescriptor` is currently just `{ id, definition }`.
- `SymbolicFieldDefinition` currently stores only `(data_type_ref, container_type)`. It does not yet carry a field name.
- Address and pointer project items already bind concrete addresses and pointer chains to a symbolic struct definition string.
- Virtual modules already have a good extension seam through `MemoryViewInstance`.
- Pointer scans already support more pointer encodings than symbolic layouts do today.
- Project symbols are authored on the unprivileged side, but the current privileged registry refresh still republishes the merged symbol catalog wholesale.

## Desired End State

### 1. Projects own a real symbol store
Projects should store two things:
- reusable symbol types,
- rooted symbols.

Reusable symbol types are layouts that can be referenced from many places.
Rooted symbols are named instances such as:
- `game.exe + 0x123456` of type `PlayerManager`,
- `Dolphin MEM1 + 0x80400000` of type `GameState`.

### 2. Layouts have named fields
This is a hard prerequisite.

If fields remain anonymous, we cannot build a meaningful symbol tree, stable child paths, good exports, or a useful explorer. The layout model needs named fields before the rest of the symbol-store work is worth much.

### 3. Rooted symbols stay sparse
We should persist only the authored roots.

Children should be derived lazily from:
- the root locator,
- the type layout,
- pointer/container semantics.

This keeps the stored model small and avoids inventing a giant persistent symbol graph we do not need.

### 4. Address and pointer items remain acquisition tools
`AddressItem` and `PointerItem` should stay.

They are good discovery tools. The project should gradually become a symbol workspace by letting users promote useful discoveries into rooted symbols.

### 5. Modules stay as strings for now
Do not introduce a new `ModuleId` abstraction yet.

The current memory layer already resolves modules by name, including virtual-module style sources. We can build the first real symbol store using module names as they exist today.

If module identity becomes painful later, we can add normalization then.

### 6. Pointer encoding should reuse the pointer-scan model
Do not build a fresh generic pointer plugin system first.

The immediate fix is to stop having one pointer model for pointer scans and a weaker one for symbolic layouts. Symbolic containers should reuse the existing pointer-scan encoding story first.

### 7. Export and import specifics live in metadata
Squalr should keep only what it needs natively.

If Ghidra, IDA, Dolphin, or other import/export workflows need extra facts, those should live in symbol metadata owned by import/export plugins or the project file, not in Squalr's core symbol model.

That means:
- no Ghidra-shaped core schema,
- no IDA-shaped namespace system,
- no exporter framework complexity until we actually need it.

### 8. The privileged side gets only execution data
The full authored symbol store should remain unprivileged-owned.

The privileged side should receive only what is needed for:
- typed reads and writes,
- resolving active project items,
- pointer expansion for active symbol views,
- cheap address labeling.

The current whole-catalog registry sync is acceptable as a temporary bridge, but not as the intended end state for a large symbol store.

## Core Model

### Symbol types
Keep the current reusable layout idea, but evolve it into a proper symbol type definition with:
- a project-local stable key,
- a display name,
- ordered named fields,
- field type/container information.

The key point is that the stable key is internal and boring. It does not need to become a giant identity system. It only needs to be stable enough that a rename does not break references.

### Rooted symbols
Add a first-class rooted symbol record with:
- a project-local stable key,
- a display name,
- a root locator,
- a referenced symbol type,
- optional metadata.

For the first implementation, root locator support should be:
- absolute address,
- module name + offset.

Virtual modules fit the same model because they already surface as modules through the memory-view layer.

### Metadata
Metadata should stay small and optional.

Start with a simple extension map for:
- import/export hints,
- comments,
- aliases if needed later.

Do not build a typed metadata framework up front.

## What We Are Explicitly Not Building Yet
- No full opaque `SymbolId` architecture with UUID-heavy plumbing.
- No separate `ModuleId` system.
- No persistent child symbol graph.
- No exporter/plugin framework redesign.
- No global reverse address-to-symbol pointer map.
- No replacement of all project item types with symbols immediately.

## Implementation Plan

### Sprint 1: Fix the model prerequisites
1. Add field names to symbolic layout definitions.
2. Update the layout parser, serialization, and struct-view editing flow to preserve field names.
3. Replace `ContainerType::Pointer32/Pointer64` with a container shape that reuses the existing pointer-scan pointer encoding model.
4. Keep migration compatibility for old layout data where possible.

### Sprint 2: Add rooted symbols
1. Replace or evolve `ProjectSymbolCatalog` into a project symbol store that contains both symbol types and rooted symbols.
2. Add rooted symbols for:
   - absolute address,
   - module name + offset.
3. Keep module names as plain strings.
4. Keep metadata optional and minimal.

### Sprint 3: Add promotion from existing project items
1. Keep address and pointer items exactly as the bootstrap UX.
2. Add `Promote to Symbol`.
3. Promotion should:
   - create a rooted symbol,
   - reuse the current type/layout reference,
   - default the new symbol name from the final tail when promoting from a pointer path,
   - avoid inventing intermediate stored symbols unless the user explicitly promotes them later.

### Sprint 4: Add lazy symbol expansion
1. Build symbol-tree expansion from rooted symbols plus layouts.
2. Derive inline children immediately.
3. Resolve pointer children on demand.
4. Do not persist derived children by default.
5. Only persist extra child state if there is a real edit that cannot live on the root or type.

### Sprint 5: Put symbols to work in the UI
1. Add a symbol explorer window.
2. Show exact-match symbol labels where cheap:
   - rooted symbol addresses,
   - safe inline child offsets of rooted symbols,
   - explicitly expanded pointer branches if cached.
3. Keep scan augmentation conservative. Do not attempt a full process-wide reverse pointer graph.

### Sprint 6: Clean up the transport boundary
1. Stop treating the privileged registry catalog as the permanent transport for the full authored symbol store.
2. Move toward sending compact execution-oriented symbol data instead of the entire authored model.
3. Keep authoring-only metadata and tree state unprivileged-side.

## GUI Shape

### Primary symbol UI: a dedicated Symbol Explorer
The main symbol-authoring surface should be its own dockable window.

It should feel much closer to the project explorer than to the memory viewer or code viewer:
- tree of rooted symbols on the left,
- lazy expansion into derived children,
- context actions on nodes,
- selection-driven detail/preview panel,
- explicit refresh for pointer-derived branches.

This should be the place where users:
- browse authored roots,
- inspect child paths,
- rename symbols,
- edit symbol metadata,
- promote a child into a real rooted symbol,
- jump to memory view or code view,
- create address or pointer project items from symbol nodes when useful.

### What the Symbol Explorer should show
Each node should show, at minimum:
- display name,
- resolved address when available,
- type name,
- container shape,
- current preview value if cheap to fetch.

The window should support:
- expanding inline children immediately,
- expanding pointer children on demand,
- caching expanded pointer branches until refresh,
- a breadcrumb or full path display for the selected node.

### Memory Viewer role
The memory viewer should remain page-oriented.

It is still the right place to walk raw memory pages, inspect bytes, and operate by address. Symbol support there should be lightweight:
- show exact-match symbol labels beside the address when available,
- optionally show inline child labels for visible ranges when resolution is cheap,
- allow jump from a byte selection to the corresponding symbol if known,
- allow promote-to-symbol from the current address selection.

The memory viewer should consume symbol information, not become the primary symbol browser.

### Code Viewer role
The code viewer should remain code-oriented.

Symbol integration there should mirror the memory viewer:
- annotate exact symbol matches where useful,
- show symbol names for known code/data references when cheap,
- jump from an instruction/reference to the corresponding symbol,
- allow promotion of discovered addresses into symbols.

It should not be the main place to browse large symbol trees.

### Project Explorer role
The project explorer should remain focused on project items and workflow artifacts.

In the short term:
- address items and pointer items remain acquisition tools,
- symbols get their own explorer,
- promotion is the bridge between the two.

This avoids mixing "raw discoveries to work from" with "authored symbolic model" into one overloaded tree too early.

## Practical Design Decisions

### Identity
We do need stable references for types and rooted symbols, but this should stay simple.

Use a project-local stable key for each stored type and each rooted symbol. Do not use the user-visible display name as the only identity, because rename support and references from project items need something stable.

This does not need to become a complicated generalized id system.

### Naming
Display names should be human-facing and easy to export.

Qualified names and child paths are for display and export. They should not force us to assume a C++, C#, or other source-language namespace model.

### Children
Children are derived views, not stored facts, unless the user explicitly turns one into a real symbol later.

### External tools
If an import/export plugin needs extra naming hints, original-source names, namespace mappings, comments, or tags, those belong in metadata associated with that plugin or format.

### Managed runtimes
If we later want strong support for CLR-heavy, JVM-heavy, or other managed-runtime targets, the current plan should extend by adding a runtime-aware symbol provider layer rather than reshaping the whole core model.

The intended shape is:
- virtual modules still describe the visible memory space,
- data-type plugins still describe value encodings,
- a future runtime/symbol-provider plugin can contribute runtime-specific roots and resolution logic.

That future plugin would be the place for things like:
- managed statics,
- GC handles or pinned-object style roots,
- metadata-token based lookup,
- runtime-specific symbol metadata.

The core symbol store should stay runtime-agnostic and let those quirks live in plugin-provided locators and metadata when the time is right.

## Bottom Line
The symbol-store plan should stay practical:
- named layouts,
- rooted symbols,
- promote-from-address/pointer,
- lazy children,
- string module names for now,
- reused pointer-scan pointer encodings,
- metadata for external-tool specifics,
- compact privileged execution data later.

That is enough to make projects gradually become symbol workspaces without building an over-generalized database first.
