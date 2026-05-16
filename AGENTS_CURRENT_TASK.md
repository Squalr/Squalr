# Agentic Current Task
Our current task, from `README.md`, is:
`pr/todo`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- Audit and plan a piecemeal migration from caller-built `ValuedStruct` details toward a reflection/schema-style Details system.
- Do not do another containment-only extraction as the architectural answer. The goal is to make inspected objects describe their own details and edit semantics, not just move bulky view code into nearby helpers.
- Add a shared Details document/edit model first, then migrate Project Hierarchy and Symbol Tree to it incrementally while keeping the current Struct Viewer UI working during the transition.

## Important Information

- Current choke point: `StructViewerViewData` accepts a completed `ValuedStruct` plus an `Arc<dyn Fn(ValuedStructField)>` edit callback. This forces each caller to synthesize display fields, remember string field names, and decode edits itself.
- `ValuedStruct`/`ValuedStructField` carry data and read-only state, but not enough Details metadata: no stable field id separate from label, no source/origin, no semantic editor kind, no validation type override, no grouping, and no declared edit operation.
- `StructViewerViewData` currently owns many domain-specific virtual field ids and editor decisions, including project-item pointer fields, symbol resolver fields, and symbol layout fields. That is a sign the Details layer is missing a semantic field schema.
- `ProjectItemType` currently only exposes activation/tick behavior. It does not expose details projection, edit planning, preview planning, runtime target resolution, or value write semantics, so Project Hierarchy compensates with type-id switches.
- `ProjectItem` persists properties as a `ValuedStruct`, which can remain the storage format for now. The migration should add a Details projection above it rather than immediately replacing persistence.
- `ProjectItemDetails` is only a temporary containment point from the previous refactor. It should not be treated as the desired architecture; much of its logic belongs in shared project-item detail/command services or type-specific capabilities.
- Existing engine services already cover some reusable pieces: `project_item_preview`, `project_item_activation`, and `project_item_symbol_resolution`. Project-item Details should reuse those instead of duplicating runtime pointer/address resolution in the GUI.
- `project_symbols write-value` already exists as the correct command-boundary pattern for runtime writes. Project item runtime value edits need an equivalent `project_items write-value` or shared write command path so GUI/CLI/TUI do not build `MemoryWriteRequest` directly.
- Symbol Tree already has a shared tree model and shared semantic operations, but its Details projection is still mostly GUI-owned in `symbol_tree_view.rs`.
- Recommended shared model location: start in `squalr-engine-api/src/structures/details/` or similar, because GUI, CLI, TUI, and future plugins all need the same description/edit vocabulary. Keep GUI-only rendering widgets in `squalr/src/views/struct_viewer`.

## Details System Migration Plan

1. Introduce a shared Details model with no behavior migration yet.
   - Add types like `DetailDocument`, `DetailField`, `DetailFieldId`, `DetailFieldValue`, `DetailFieldEditorHint`, `DetailEdit`, and `DetailEditPlan`.
   - Keep it semantic, not egui-specific. The GUI maps semantic editor hints to controls.
   - Include enough metadata to retire virtual-name switches over time: stable id, display label, read-only/editable, validation `DataTypeRef`, container type, source/origin, and optional action/edit kind.

2. Add a compatibility adapter in the Struct Viewer.
   - Let `StructViewerViewData` focus either a legacy `ValuedStruct` or a new `DetailDocument`.
   - Initially adapt `DetailDocument` into the existing rendered `ValuedStruct` so the UI does not need a large rewrite.
   - Convert field edits back to `DetailEdit` using stable field ids instead of relying on display/property names.

3. Move project-item detail projection behind a shared provider/planner.
   - Address, Pointer, Directory, and Script item types should expose their detail fields through shared detail projection code.
   - Project Hierarchy should ask for a `DetailDocument` for selected project items instead of building struct fields itself.
   - Pointer size/offsets, module, symbolic type, runtime value, and preview display should become semantic fields, not `StructViewerViewData` virtual fields.

4. Add project-item edit planning before changing UI behavior.
   - A project item detail edit should return an explicit plan: rename item, update persisted properties, write runtime value, save project, refresh details, or no-op/error.
   - Keep command dispatch outside the Struct Viewer. The view invokes a planner/command adapter rather than switching on field names.

5. Add a `project_items write-value` command path.
   - Move runtime value write planning out of the GUI and reuse `project_item_symbol_resolution` for address/pointer/symbol target resolution.
   - Mirror the `project_symbols write-value` boundary so GUI/CLI/TUI can all write project item runtime values consistently.

6. Migrate Symbol Tree Details projection.
   - Add a shared Symbol Tree detail projector that accepts `ProjectSymbolCatalog`, `SymbolTreeNode`, and runtime/default-value context.
   - Preserve existing `project_symbols write-value` for edits; the GUI should route a `DetailEdit` into that command instead of building symbol-specific callbacks in `symbol_tree_view.rs`.

7. Drain domain-specific virtual fields out of `StructViewerViewData`.
   - Once Project Hierarchy and Symbol Tree are on `DetailDocument`, move symbol resolver/layout editor fields to the same model or consciously keep them as editor-specific until their turn.
   - The end state is that `StructViewerViewData` handles selection, edit state, display formats, and generic editor dispatch, while domains describe fields and plan edits.

8. Only then slim the large views.
   - After shared projection/edit planning exists, remove view-owned Details construction from `ProjectHierarchyView` and `SymbolTreeView`.
   - The views should select objects, request detail focus, render tree/list rows, and route user actions. They should not parse edited field names or build memory write requests.

## Validation Notes

- Details architecture audit used source inspection only; no runtime behavior was changed in this audit pass.
