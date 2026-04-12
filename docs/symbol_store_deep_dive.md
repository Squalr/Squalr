# Symbol Store Deep Dive

## Goal
Design a project-owned symbol store that can:
- Label static modules and plugin-defined virtual modules.
- Expand from those roots through nested structs, arrays, and pointer/container types.
- Stay extensible for unusual pointer widths and memory layouts.
- Respect the privileged/unprivileged split without making Android or IPC-heavy targets miserable.
- Support future UX such as a full symbol tree explorer and symbolic naming in scan/project views.

## Current State

### What already exists
- Projects already persist a `ProjectSymbolCatalog`, but it currently stores only `Vec<StructLayoutDescriptor>`.
- `StructLayoutDescriptor` currently contains only an id and a `SymbolicStructDefinition`.
- `SymbolicStructDefinition` models a named layout as `symbol_namespace + fields`.
- `SymbolicFieldDefinition` already models a field as `(data_type_ref, container_type)`.
- Project open/close already synchronizes the whole project symbol catalog into the privileged symbol registry with `RegistrySetProjectSymbolsRequest`.
- The privileged registry already exports a full `PrivilegedRegistryCatalog`, and the unprivileged side refreshes its cached copy by generation.
- Project address/pointer items already carry a `symbolic_struct_definition_reference`, so the UI already has a place to bind an address or pointer to a symbolic type.
- The struct viewer already has a light authoring path for container mode and fixed array length edits by rewriting the symbolic field definition string.
- Memory-view plugins already provide the extension seam for virtual modules, virtual pages, address-to-module resolution, and module-offset resolution.
- Pointer scans already support nonstandard pointer sizes better than the struct/container model does today: `u24`, `u24be`, `u32`, `u32be`, `u64`, `u64be`.

### What does not exist yet
- There is no first-class symbol record for "this label is rooted at module X + offset Y" or "this label is rooted at virtual module Y + offset Z".
- There is no symbol graph/tree, only reusable struct layouts.
- There is no notion of a typed instance path such as `module + offset -> field -> pointer deref -> field -> array[index]`.
- There is no metadata envelope for exports, comments, ownership, provenance, aliases, tags, or plugin-specific annotations.
- There is no partial or incremental registry sync. Registry refresh is still whole-catalog by generation.
- Container modeling is still hardcoded to `Pointer32` and `Pointer64`, so symbolic structs are behind pointer-scan support for weird pointer widths.
- There is no dedicated symbol explorer window.
- Scan results and memory views do not currently resolve addresses back to user-authored symbol paths.

## Architectural Reading Of The Current Repo

### Ownership boundary
- Project-authored symbols are already effectively unprivileged-owned source-of-truth.
- The privileged side only needs enough symbol data to materialize typed reads/writes, pointer previews, scan formatting, and similar execution-time tasks.
- That split is healthy and should be preserved.

### Important caveat
- The current privileged registry catalog export includes all struct layouts known to the privileged side, and `create_registry_catalog()` builds that list from the merged registry, including project-authored symbols.
- On the unprivileged side, registry generation refresh is a whole-catalog fetch, not a diff.
- That means "keep project symbols unprivileged-owned" is true conceptually, but today the transport still republishes them through the privileged catalog path.
- If the symbol store grows large, this becomes one of the main scalability hazards for Android and other split-brain configurations.

### Existing extension seams we should reuse
- `MemoryViewInstance` is already the right home for virtual-module identity and guest/host address translation logic.
- `ProjectItemTypeAddress` and `ProjectItemTypePointer` already provide the main UX binding from concrete addresses/pointer chains to symbolic types.
- The struct viewer already knows how to expose and edit some symbolic-field properties.
- `PointerScanPointerSize` already proves the codebase is willing to model odd pointer sizes and endian variants explicitly.

## Recommended Data Model

The biggest missing piece is that the repo currently stores only reusable layouts, not symbol instances. A useful symbol store needs both.

### 1. Keep layout definitions
Retain reusable type/layout definitions, but evolve them into a more extensible form:
- `SymbolTypeDefinition`.
- Stable id / namespace.
- Ordered field list.
- Optional size/alignment policy.
- Optional metadata bag.

This is close to the current `SymbolicStructDefinition`.

### 2. Add symbol instances
Add first-class rooted symbols that bind a name to an address expression plus a type:
- `StaticAddressSymbol`.
- `PointerPathSymbol`.
- `DerivedFieldSymbol`.
- `VirtualModuleSymbol`.

Minimum properties:
- Stable symbol id.
- Display name.
- Root locator.
- Type reference.
- Optional parent symbol id.
- Optional metadata/extensions.

### 3. Separate root locator from type
The root locator should be an addressing expression, not a free-form string.

Recommended shape:
- `AddressRoot::Absolute { address }`
- `AddressRoot::ModuleOffset { module_id, offset }`
- `AddressRoot::VirtualModuleOffset { module_id, offset }`
- `AddressRoot::SymbolField { parent_symbol_id, field_path }`
- `AddressRoot::PointerChain { base, hops, pointer_encoding }`

This cleanly separates "where is it" from "what type does it have".

### 4. Generalize container/pointer encoding
The current `ContainerType` is too narrow for the user’s weird-architecture constraint.

Instead of `Pointer32` and `Pointer64`, prefer something like:
- `Pointer(PointerEncodingId)`
- `ArrayDynamic`
- `ArrayFixed(length)`
- `Inline`

Where `PointerEncodingId` can map to:
- width in bits or bytes,
- endian,
- signedness rules if needed,
- canonical data type id for materialization,
- architecture/plugin ownership.

This can either wrap the existing `PointerScanPointerSize` or replace it with a more general shared type. The important part is that symbolic structs and pointer scans should stop having separate pointer-size universes.

### 5. Add metadata as a typed extension bag
Exports and ecosystem integrations will need extra facts that Squalr itself should not hardcode into the base schema.

Recommended metadata layers:
- Core optional fields: comment, tags, source, confidence, aliases, last_verified_at.
- Export hints: preferred namespace, section hint, size hint, signedness hint.
- Plugin extension bag: `HashMap<String, serde_json::Value>` keyed by plugin/exporter id.

This gives us a safe base schema without baking Ghidra/Dolphin-specific concepts into core types.

## Recommended Storage Layout

The project file should likely evolve from:
- `symbols: { struct_layout_descriptors... }`

Toward:
- `symbol_types`
- `symbol_roots`
- `symbol_bindings`
- `pointer_encodings`
- `metadata_schemas` or exporter/plugin metadata

One workable layout:

```json
{
  "symbols": {
    "types": [],
    "roots": [],
    "symbols": [],
    "pointer_encodings": [],
    "virtual_module_overrides": []
  }
}
```

Why split this way:
- Types are reusable and deduplicate nested layouts.
- Roots carry process-relative anchoring.
- Symbols carry naming, ownership, tagging, and parent/child relationships.
- Pointer encodings become reusable and extensible.

## Privileged vs Unprivileged Recommendation

### Source of truth
Keep the full authored symbol store unprivileged-side and project-owned.

Reasons:
- It is user/project data, not privileged execution state.
- It changes more often from authoring workflows than from privileged runtime requirements.
- It is much easier to version, migrate, diff, export, and edit on the unprivileged side.

### What the privileged side should get
Only send the smallest execution-oriented subset needed for current commands:
- Type/layout definitions that privileged reads/writes must materialize.
- Pointer encoding definitions needed to dereference typed pointers.
- Possibly a compact root-resolution table only for active symbols or views.

### What should stay unprivileged-only
- Display trees.
- Symbol explorer state.
- Export metadata.
- Authoring comments/tags.
- Large alias/path indexes.
- Reverse address indexes unless a specific command needs them.

### Main change needed
The current generation-based registry refresh path is too blunt for a large symbol store because it republishes the full registry catalog.

Recommended medium-term fix:
- Split privileged registry metadata from project symbol authoring state.
- Do not include project-authored symbols in the general privileged catalog used for UI cache refresh.
- Introduce a separate command family for on-demand symbol execution packs, or a diff-based project-symbol sync channel.

For desktop standalone mode this matters less, but for Android IPC it matters a lot.

## Virtual Modules And Static Labels

This repo already has the right conceptual seam for virtual modules through memory-view plugins. The symbol store should lean into that rather than inventing a second virtual-module abstraction.

Recommended rule:
- Module identity comes from the memory layer.
- The symbol store references module ids, not raw module names wherever possible.

Needed improvements:
- Introduce a stable `module_id` concept distinct from display name.
- Let memory-view plugins expose module capabilities and optional metadata.
- Allow project-side module aliasing so users can bind symbols to stable logical module ids even if a plugin’s display text changes.

For weird targets like Dolphin or emulator memory:
- The plugin owns address translation.
- The symbol store owns labels rooted in those translated module spaces.
- Exports can read plugin-provided metadata from the extension bag when needed.

## Fan-Out Through Nested Structs And Containers

The user’s "fan-out to the entire codebase through container types and nested symbolic structs" is the core design requirement.

The cleanest model is:
- Root symbols are sparse.
- Child symbols are mostly derived, not separately authored.

Example:
- Author `player_manager` at `game.exe + 0x123456`, type `PlayerManager`.
- `PlayerManager` field `active_player` is a pointer to `Player`.
- `Player` field `inventory` is `Inventory[4]`.

The symbol store should be able to materialize a tree like:
- `player_manager`
- `player_manager.active_player`
- `player_manager.active_player->gold`
- `player_manager.active_player->inventory[0]`

That does not mean every node must be stored permanently. Most of these should be derived on demand from:
- root symbol,
- type graph,
- pointer/container semantics,
- optional cached expansion state.

Recommendation:
- Persist authored roots and explicit overrides.
- Derive the full tree lazily in the explorer and in formatting/indexing passes.

## Symbol Resolution Runtime

You will likely want two resolvers:

### 1. Authoring resolver
Runs unprivileged-side.
Responsibilities:
- Type validation.
- Child-path expansion.
- Explorer tree building.
- Export graph generation.
- Address formatting and friendly naming.

### 2. Execution resolver
Runs privileged-side or via a compact execution pack.
Responsibilities:
- Materialize read/write layouts.
- Resolve concrete addresses for active project items.
- Walk pointer chains with architecture-correct pointer encodings.

This split keeps expensive authoring state out of IPC while still letting privileged commands do their job.

## Scan Augmentation With Known Symbols

This is possible, but there are two very different scopes:

### Cheap and viable first step
When a scan result address exactly matches:
- a rooted symbol address, or
- a currently materialized pointer project item target,

show the symbol name in place of the raw address or alongside it.

This only needs:
- exact address map,
- current modules,
- maybe a small cache for active symbols.

### Expensive version
Show nearest symbolic path for arbitrary addresses inside nested graphs, arrays, structs, and reachable pointer trees.

This effectively needs an address-to-symbol index over large portions of the process space. Doing that continuously for real-time scans is expensive because:
- pointer-derived children are dynamic,
- process memory changes invalidate cached expansions,
- array and nested struct explosion can become enormous,
- full-process reverse reachability through pointers is close to building a live pointer graph.

### Recommendation
Do not start with a full pointer-map-backed address-to-symbol engine.

Instead phase it:
1. Exact root-address aliasing.
2. Exact field-offset aliasing for non-pointer inline children of rooted symbols.
3. Optional cached pointer-expanded aliasing for explicitly opened explorer branches or pinned symbols.
4. Only then reconsider global reverse reachability.

This gives good UX without committing to a process-wide real-time pointer graph.

## Symbol Explorer Window

This is a very natural fit for a standalone GUI window.

Recommended MVP:
- Tree rooted by authored symbols.
- Expands inline fields immediately.
- Expands pointer children on demand.
- Shows current resolved address, type, container shape, and current value preview.
- Caches expanded branch results with explicit refresh.
- Supports jump-to memory viewer / code viewer / project item creation.

This should stay unprivileged-driven, with targeted memory reads for expanded branches rather than one giant process crawl.

## Export Pipeline

Exports are a strong reason to introduce structured metadata early.

Recommended architecture:
- Core symbol store remains exporter-agnostic.
- Exporters implement a trait over the symbol graph plus extension metadata.
- Plugins can register exporters and metadata interpreters.

Possible export targets:
- JSON interchange.
- Ghidra labels/types.
- Dolphin/GameCube cheat/symbol formats.
- Emulator-specific symbol maps.

Suggested export contract:
- Inputs: symbol graph, type graph, module map, plugin metadata, process/platform metadata.
- Outputs: file bundle(s), diagnostics, skipped-symbol report.

This is better than letting every exporter infer structure from free-form comments.

## Biggest Architectural Risks

### 1. IPC blow-up
The current registry catalog refresh is whole-state by generation. Large symbol stores will punish Android and any split process architecture.

### 2. Pointer semantics split brain
Pointer scans already understand more pointer formats than symbolic structs do. If not unified, symbol-store work will duplicate architecture logic in multiple places.

### 3. Naming/module identity instability
Raw module-name strings are easy to author but brittle for plugins and weird targets. Stable module ids are safer.

### 4. Eager graph expansion
Persisting or eagerly building every derived child symbol will explode memory and UI latency.

### 5. Overloading the privileged registry
If the privileged registry becomes the universal home of authoring state, it will become a transport bottleneck and muddle the current ownership boundary.

## Recommended Phased Build Order

### Phase 0: Refactor prerequisites
- Generalize pointer/container modeling so symbolic structs can use the same pointer encoding story as pointer scans.
- Introduce stable module ids for host and virtual modules.
- Split project-authored symbol sync from the generic privileged registry metadata path, or at minimum add a compact sync mode.

### Phase 1: Persist rooted symbols
- Keep existing type/layout support.
- Add rooted symbol records for module-offset and absolute-address labels.
- Bind project items and UI labels to rooted symbol ids instead of only a raw symbolic type string.
- Add a small exact-address symbol index on the unprivileged side.

### Phase 2: Add explorer and lazy derivation
- Build the symbol explorer window.
- Derive inline child nodes from type layouts.
- Resolve pointer children on demand using cached targeted reads.
- Support project item creation from any explorer node.

### Phase 3: Add friendly naming across the product
- Replace raw addresses with symbol names where there is an exact or safe inline-field match.
- Show symbol paths in project items, memory viewer, and scan results when resolution is cheap and unambiguous.

### Phase 4: Add export metadata and exporter plugins
- Introduce metadata extension bags.
- Add exporter trait and first-party JSON export.
- Add at least one real integration target to validate the schema.

### Phase 5: Revisit advanced reverse address mapping
- Only after the above is stable should we explore pointer-expanded reverse indexes or reachability-assisted symbolic lookup for arbitrary scan hits.

## Concrete First Implementation Slice

If the goal is to start building now with low regret, I would start here:

1. Replace `ContainerType::Pointer32/Pointer64` with an extensible pointer encoding reference.
2. Introduce `ProjectSymbolStore` beside the current `ProjectSymbolCatalog`.
3. Keep `StructLayoutDescriptor` compatibility for migration, but add new rooted symbol records.
4. Add a compact unprivileged-owned exact-address index for rooted symbols.
5. Let address and pointer project items optionally reference a rooted symbol id in addition to a type/layout id.
6. Add a small symbol explorer MVP rooted only at explicit static/virtual-module symbols.

This delivers visible value early without requiring the full "show a symbol for every reachable address" problem to be solved.

## Questions Worth Settling Early
- Do we want symbol ids to be user-stable strings, UUIDs, or path-like namespaces?
- Are module ids authored by plugins, by the OS layer, or normalized by Squalr?
- Should pointer encodings be fully generic now, or should we first unify around the existing pointer-scan enum and generalize later?
- Do we want symbol child nodes to be persisted when renamed/annotated, or should overrides be stored as patches over derived paths?
- Should project items point at symbol ids, type ids, or both?

## Bottom Line

This repo is already surprisingly close to the starting line:
- reusable type/layout definitions exist,
- project-owned symbol persistence exists,
- project-to-privileged sync exists,
- virtual modules already have a plugin seam,
- project items already carry symbolic type references,
- pointer scans already pushed the codebase toward weird-pointer support.

The missing leap is not "support symbols at all". The missing leap is introducing first-class rooted symbol instances, generalizing pointer/container semantics, and preventing the privileged registry sync path from becoming the transport bottleneck for authored symbol data.
