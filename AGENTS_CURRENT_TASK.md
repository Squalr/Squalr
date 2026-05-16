# Agentic Current Task
Our current task, from `README.md`, is:
`pr/todo`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- Replace caller-built `ValuedStruct` details with a shared reflection/schema-style Details projection and edit-planning model.
- Do this piecemeal. Keep the existing Struct Viewer UI working while adding a Details path beside the legacy `ValuedStruct` path.
- Do not start by slimming views or doing containment-only extraction. First create the shared vocabulary that lets inspected objects describe fields, editor semantics, and edit intent.
- Treat `ProjectItemDetails` as a temporary bridge from the previous refactor. Do not build more architecture on top of it.

## Important Information

- `StructViewerViewData` is the main choke point. It accepts completed `ValuedStruct` data plus an `Arc<dyn Fn(ValuedStructField)>` callback, so callers have to synthesize fields, remember string field names, and decode edits themselves.
- `StructViewerViewData` currently owns domain-specific virtual fields in `squalr/src/views/struct_viewer/view_data/struct_viewer_view_data.rs`: project item pointer rows, symbol resolver rows, symbol layout rows, live value detection, readable labels, editor choice, and virtual container row injection.
- `ValuedStruct`/`ValuedStructField` in `squalr-engine-domain/src/structures/structs/` carry data and read-only state, but not enough Details metadata: no stable field id separate from label, no source/origin, no semantic editor kind, no validation type override, no grouping, and no declared edit operation.
- `StructViewerFieldPresentation` in `squalr/src/views/struct_viewer/view_data/struct_viewer_field_presentation.rs` is GUI-only. New shared Details types must not depend on these widget/editor variants.
- `ProjectItemType` in `squalr-engine-api/src/structures/projects/project_items/project_item_type.rs` only exposes activation/tick behavior. It does not expose details projection, edit planning, preview planning, runtime target resolution, or value write semantics, so Project Hierarchy compensates with type-id switches.
- `ProjectItem` persists properties as a `ValuedStruct`. Keep that storage format during the migration; add a Details projection above it first.
- `ProjectItemDetails` in `squalr/src/views/project_explorer/project_hierarchy/project_item_details.rs` contains useful extracted facts, but it is still GUI-side glue. It currently owns runtime write planning, property projection, icon data type resolution, value-edit context, snapshot queries, runtime target resolution, and field-name filtering.
- Existing engine services already cover reusable project item behavior: `squalr-engine/src/services/projects/project_item_preview.rs`, `project_item_activation.rs`, and `project_item_symbol_resolution.rs`.
- `project_symbols write-value` already exists and is CLI-parsed. Project item runtime value edits need an equivalent `project_items write-value` or a shared write command path so GUI/CLI/TUI do not build `MemoryWriteRequest` directly.
- Symbol Tree already has shared tree data and command operations, but its Details projection is still GUI-owned in `squalr/src/views/symbol_tree/symbol_tree_view.rs`.
- Shared Details model location: `squalr-engine-api/src/structures/details/`. GUI adapters belong under `squalr/src/views/struct_viewer/`. Engine-only runtime planners belong under `squalr-engine/src/services/`.

## Concrete Hot Spots

- `StructViewerViewData` methods to drain or bypass:
  - `focus_valued_struct*`
  - `resolve_source_field_edit`
  - `create_field_presentations`
  - `is_live_value_field`
  - `create_presented_struct`
- `ProjectHierarchyView` methods to migrate after shared Details exists:
  - `focus_project_item_paths_in_struct_viewer`
  - `build_project_item_details_edit_callback`
  - `apply_project_item_edits`
  - `build_pointer_scanner_context_actions`
  - `resolve_tree_entry_icon`
- `SymbolTreeView` methods to migrate after shared Details exists:
  - `focus_symbol_tree_entry_in_struct_viewer`
  - `build_struct_viewer_edit_callback`
  - `build_symbol_layout_for_tree_entry`
  - `build_symbol_layout_metadata_fields`
  - `build_symbol_layout_location_fields`
  - `dispatch_memory_read_request`
  - `build_symbol_preview_snapshot_queries`

## Detailed Action Items

1. Done: add the shared Details model before moving any GUI behavior.
   - Create `squalr-engine-api/src/structures/details/mod.rs`.
   - Add `details_projection.rs`, `details_field.rs`, `details_edit.rs`, `details_edit_plan.rs`, and `details_target.rs`.
   - Minimum model:
     - `DetailsProjection { target, title, fields }`
     - `DetailsTarget { target_kind, target_id }`
     - `DetailsField { id, label, value, is_read_only, editor_hint, validation_data_type_ref, container_type, source }`
     - `DetailsEdit { target, field_id, value }`
     - `DetailsEditPlan { operations }`
   - Keep `DetailsEditorHint` semantic: value, address, code, data type, container type, pointer offsets, pointer size, resolver, layout field, etc. Do not use egui/widget enum names.
   - Keep `DetailsFieldSource` explicit enough to represent project item property, project item runtime value, project item address target metadata, project symbol runtime value, symbol layout metadata, and symbol resolver metadata.
   - Added small API tests for serialization/round trip, stable field id routing, and ordered edit-plan operations.

2. Done: add a Struct Viewer compatibility adapter.
   - Added GUI-only adapter at `squalr/src/views/struct_viewer/view_data/details_projection_adapter.rs`.
   - Added `StructViewerViewData::focus_details_projection_with_focus_target` beside the existing `focus_valued_struct*` APIs.
   - Converts `DetailsProjection` into the rendered legacy `ValuedStruct` so table rendering stays stable.
   - Preserves a local mapping from rendered field name to `DetailsFieldId` and uses `StructViewerFieldPresentation` for display labels.
   - Converts edited rows back into `DetailsEdit` before invoking caller logic. Callers no longer need to parse display labels on the Details path.
   - Added focused tests proving labels can change without breaking edit routing.

3. Stop adding domain rules to `StructViewerViewData`.
   - Leave old behavior in place while migration is incomplete.
   - New Details flows must carry editor hints/source metadata through `DetailsField`; they should not add more `VIRTUAL_FIELD_*` constants.
   - Existing project item pointer fields, symbol resolver fields, symbol layout fields, and live value field decisions should be removed only after their callers are migrated to Details.

4. In progress: add a project item detail projector.
   - Added pure API-shaped projection in `squalr-engine-api/src/structures/projects/project_items/details/`.
   - Put runtime-aware projection/planning in `squalr-engine/src/services/projects/project_item_details.rs` if it needs process memory, module resolution, pointer evaluation, or preview data.
   - API projector currently covers stored project item fields for Directory, Script, Address, and Pointer items.
   - API projector models address target pointer size/offsets, projected module metadata, symbolic data type, and runtime value as semantic `DetailsField`s.
   - Existing field-shape tests for address target fields, pointer preview hiding, projected module, and runtime editability were mirrored into the API projector.
   - Remaining target: migrate `ProjectItemDetails::build_struct_view_properties` and `resolve_project_item_icon_data_type_id` callers to shared projection.

5. Add project item edit planning.
   - A project item detail edit should produce explicit operations such as:
     - rename project item,
     - update persisted property,
     - update address target pointer size/offsets,
     - write runtime value,
     - save project,
     - refresh details,
     - no-op/error.
   - Use existing command boundaries where possible, including `ProjectItemsRenameRequest`, property mutation plus save, and the future runtime write command.
   - Replace `ProjectHierarchyView::apply_project_item_edits` only after the planner can describe every existing edit path.

6. Add `project_items write-value`.
   - Add request/response/command enum variants under `squalr-engine-api/src/commands/project_items/`.
   - Add executor under `squalr-engine/src/command_executors/project_items/write_value/`.
   - Reuse `project_item_symbol_resolution::{resolve_project_item_locator, resolve_address_target_runtime_target, resolve_pointer_runtime_target}` and dispatch `MemoryWriteRequest` from the engine, not the GUI.
   - Add CLI parsing and response handling to mirror `project_symbols write-value`.
   - After this exists, runtime value edits in GUI/TUI/CLI can share the same command path.

7. Migrate Project Hierarchy to Details.
   - Change `focus_project_item_paths_in_struct_viewer` to request/focus `DetailsProjection`s.
   - Change the edit callback to receive `DetailsEdit`, ask the planner for operations, and dispatch commands.
   - Remove direct parsing of edited `ValuedStructField` names from `ProjectHierarchyView`.
   - Keep tree rendering, selection, drag/drop, and context menus in the view until a separate reason exists to move them.

8. Migrate Symbol Tree to Details.
   - Add a shared projector, likely under `squalr-engine-api/src/structures/projects/symbol_tree/details/`.
   - Move metadata/location/value field construction out of `build_symbol_layout_*`.
   - Use existing `ProjectSymbolsWriteValueRequest` for runtime edits.
   - Keep symbol tree rendering, expansion state, and selection in the GUI view.

9. Drain old virtual fields from Struct Viewer.
   - Remove project item pointer virtual fields after Project Hierarchy uses Details.
   - Remove symbol layout/resolver virtual fields after Symbol Tree and symbol authoring editors use Details.
   - Keep generic container/array display only if it is truly Struct Viewer behavior; otherwise model it as detail metadata too.

10. Only then slim the large views.
   - `ProjectHierarchyView` should select project items, render rows/context actions, and route user actions.
   - `SymbolTreeView` should select symbols, render rows/context actions, and route user actions.
   - Neither view should construct Details rows, parse edited field names, or build memory read/write requests directly.

## Ordering Constraints

- Do not slim views first; that repeats the containment-only mistake.
- Do not extend the `ProjectItemType` trait until the Details model is stable enough. Start with sidecar projectors for built-ins, then consider trait capabilities for plugins.
- Do not put command dispatch in Struct Viewer. Struct Viewer should emit `DetailsEdit` or legacy field edits only.
- Do not make `DetailsEditorHint` a GUI widget enum.
- Do not replace persisted `ProjectItem.properties` yet. Projection above storage is the safer first step.
- Do not delete `ProjectItemDetails` until Project Hierarchy has a Details projector, edit planner, and project item write-value command.

## Validation Per Stage

- Shared Details API model: added under `squalr-engine-api/src/structures/details/`; validate with `cargo test -p squalr-engine-api details --lib --locked`.
- Struct Viewer adapter: added under `squalr/src/views/struct_viewer/view_data/`; validate with `cargo test -p squalr struct_viewer --lib --locked`.
- Project item detail projector/planner: API projector validates with `cargo test -p squalr-engine-api project_item_details --lib --locked`; GUI/runtime migration should also run `cargo test -p squalr project_hierarchy --lib --locked` and `cargo test -p squalr-engine project_items --lib --locked`.
- `project_items write-value`: `cargo test -p squalr-engine project_items --lib --locked` and `cargo test -p squalr-cli parse_input --locked`.
- Symbol Tree projector: `cargo test -p squalr symbol_tree --lib --locked` and `cargo test -p squalr-engine-api symbol_tree --lib --locked`.
- Final cleanup pass: run the relevant package tests plus `cargo fmt` for touched Rust files.

## Validation Notes

- Details architecture audit used source inspection only.
- Previous implementation pass added the shared `DetailsProjection`/field/edit/plan model only.
- Current Struct Viewer pass added a compatibility adapter and focus API for `DetailsProjection`; no Project Hierarchy or Symbol Tree callers have migrated yet.
- Current project item pass added a pure API-side `ProjectItemDetailsProjection`; no Project Hierarchy callers have migrated yet.
