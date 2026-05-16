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
   - `DetailsEdit` now carries `DetailsFieldSource` from the projected field, so callers can route edits without re-inferring source from field ids.
   - Added small API tests for serialization/round trip, stable field id routing, and ordered edit-plan operations.

2. Done: add a Struct Viewer compatibility adapter.
   - Added GUI-only adapter at `squalr/src/views/struct_viewer/view_data/details_projection_adapter.rs`.
   - Added `StructViewerViewData::focus_details_projection_with_focus_target` beside the existing `focus_valued_struct*` APIs.
   - Converts `DetailsProjection` into the rendered legacy `ValuedStruct` so table rendering stays stable.
   - Preserves a local mapping from rendered field name to `DetailsFieldId` and uses `StructViewerFieldPresentation` for display labels.
   - Converts edited rows back into `DetailsEdit` with stable field id and source metadata before invoking caller logic. Callers no longer need to parse display labels on the Details path.
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
   - Single-selection Project Hierarchy details focus now uses `ProjectItemDetailsProjection` plus `StructViewerViewData::focus_details_projection_with_focus_target`.
   - Project Hierarchy icon data type resolution now uses `ProjectItemDetailsProjection::resolve_project_item_icon_data_type_id`.
   - Multi-selection Project Hierarchy details focus still uses legacy `ProjectItemDetails::build_struct_view_properties` until a multi-projection strategy exists.
   - Remaining target: remove legacy single-selection `ProjectItemDetails::build_struct_view_properties` dependency after stored-field edit application moves off legacy fields.

5. In progress: add project item edit planning.
   - Added `ProjectItemDetailsEditPlanner` in `squalr-engine-api/src/structures/projects/project_items/details/`.
   - Added `ProjectItemDetailsEditApplier` in the same API details module for stored project item field mutations.
   - A project item detail edit now produces explicit operations such as:
     - rename project item,
     - update persisted property,
     - update address target pointer size/offsets,
     - write runtime value,
     - refresh details,
     - no-op/error.
   - Single-selection Project Hierarchy Details edits now run through `ProjectItemDetailsEditPlanner`.
   - `WriteRuntimeValue` operations now carry `DetailsFieldSource::ProjectItemRuntimeValue` so callers do not need to infer runtime field paths from rendered ids.
   - Project item runtime edit planning preserves the source path carried by `DetailsEdit`, with a scalar `value` fallback for older/default edits.
   - Stored-field application now has shared logic for regular properties, address target pointer size/offsets, address target module updates, and pointer item pointer size/offset serialization.
   - Single-selection runtime value edits now dispatch `ProjectItemsWriteValueRequest` instead of building memory writes in GUI code.
   - Single-selection stored-field edits now apply through `ProjectItemDetailsEditApplier`; rename operations dispatch `ProjectItemsRenameRequest`.
   - Multi-selection legacy fallback now also dispatches `ProjectItemsWriteValueRequest` for runtime value edits instead of building `MemoryWriteRequest` in GUI code.
   - The GUI still keeps legacy `apply_project_item_edits` for multi-selection persisted property edits.
   - Remaining target: remove or shrink `ProjectHierarchyView::apply_project_item_edits` after multi-selection moves off legacy fields.

6. Done: add `project_items write-value`.
   - Added request/response/command enum variants under `squalr-engine-api/src/commands/project_items/write_value/`.
   - Added executor under `squalr-engine/src/command_executors/project_items/write_value/`.
   - Added `squalr-engine/src/services/projects/project_item_runtime_value_write.rs` so project items resolve to the existing `project_symbols write-value` runtime write planner instead of building memory writes in GUI code.
   - Reused `project_item_symbol_resolution::resolve_project_item_locator` and `resolve_project_item_struct_layout_id`, then dispatches `MemoryWriteRequest` from the engine.
   - Added CLI parsing and response handling to mirror `project_symbols write-value`.
   - Project Hierarchy runtime value edits now route through this command.

7. Migrate Project Hierarchy to Details.
   - Change `focus_project_item_paths_in_struct_viewer` to request/focus `DetailsProjection`s.
   - Change the edit callback to receive `DetailsEdit`, ask the planner for operations, and dispatch commands.
   - Single-selection Details focus/edit now uses `DetailsEdit` and dispatches `project_items write-value` for runtime edits.
   - Single-selection stored-field edits now use `ProjectItemDetailsEditApplier` instead of converting operations back to `ValuedStructField`.
   - Runtime value edits no longer build raw `MemoryWriteRequest`s in Project Hierarchy.
   - Remaining target: remove direct parsing of edited `ValuedStructField` names from multi-selection persisted property fallback.
   - Keep tree rendering, selection, drag/drop, and context menus in the view until a separate reason exists to move them.

8. Migrate Symbol Tree to Details.
   - Added shared projector under `squalr-engine-api/src/structures/projects/symbol_tree/details/`.
   - `SymbolTreeDetailsProjection` covers symbol claim display-name metadata, type metadata, address/module/size/path fields, fallback locator/status fields, and normalized runtime value fields.
   - Normal readable Symbol Tree selections now focus `DetailsProjection` through `StructViewerViewData::focus_details_projection_with_focus_target`.
   - Symbol Tree Details metadata is read-only; symbol layout/name edits stay in the struct layout editor, symbol resolver tools, and inline rename paths.
   - Normal Details runtime value edits dispatch existing `ProjectSymbolsWriteValueRequest`.
   - Remaining target: move external array/value-viewer rows off the old `ValuedStruct` path.
   - Remaining target: remove legacy metadata/location/value field construction from `build_symbol_layout_*` after the external path is migrated.
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
- Current project item pass added a pure API-side `ProjectItemDetailsProjection`.
- Current Project Hierarchy pass migrated single-selection details focus to `DetailsProjection`; multi-selection and legacy edit application still use `ProjectItemDetails`.
- Current project item planning pass added `ProjectItemDetailsEditPlanner` and wired single-selection Project Hierarchy Details edits through it.
- Current command pass added `project_items write-value` plus an engine service that resolves project items into symbol runtime writes. Validated with `cargo test -p squalr-engine project_items --lib --locked`, `cargo test -p squalr-engine project_item_runtime_value_write --lib --locked`, and `cargo test -p squalr-cli parse_input --locked`.
- Current Project Hierarchy command-routing pass changed Details runtime edits to dispatch `ProjectItemsWriteValueRequest` and added runtime source metadata to `DetailsEditOperation::WriteRuntimeValue`. Validated with `cargo test -p squalr-engine-api project_item_details --lib --locked` and `cargo test -p squalr project_hierarchy --lib --locked`.
- Current icon-routing pass moved Project Hierarchy icon data type resolution to `ProjectItemDetailsProjection` and removed the GUI bridge method.
- Current shared applier pass added `ProjectItemDetailsEditApplier` for stored project item Details operations. Validated with `cargo test -p squalr-engine-api project_item_details --lib --locked`.
- Current Project Hierarchy stored-edit pass wired single-selection `UpdateStoredField` operations to `ProjectItemDetailsEditApplier` and `RenameTarget` operations to `ProjectItemsRenameRequest`. Validated with `cargo test -p squalr project_hierarchy --lib --locked`.
- Current Project Hierarchy runtime cleanup pass routed legacy fallback runtime edits through `ProjectItemsWriteValueRequest` and removed `ProjectItemDetails::build_memory_write_request_for_runtime_value_edit`. Validated with `cargo test -p squalr project_hierarchy --lib --locked`.
- Current Symbol Tree projector pass added `SymbolTreeDetailsProjection` for metadata, fallback status, and runtime value fields. Validated with `cargo test -p squalr-engine-api symbol_tree_details --lib --locked`.
- Current Symbol Tree focus pass routed normal readable Symbol Tree selections through `SymbolTreeDetailsProjection` and Details edit callbacks, leaving the external array/value-viewer path legacy. Validated with `cargo test -p squalr symbol_tree --lib --locked`.
- Current Details edit-source pass preserved `DetailsFieldSource` on `DetailsEdit` and routed Symbol Tree runtime edits from source metadata instead of field-id inference. Validated with `cargo test -p squalr-engine-api details --lib --locked`, `cargo test -p squalr struct_viewer --lib --locked`, and `cargo test -p squalr symbol_tree --lib --locked`.
- Current project item source-preservation pass changed `ProjectItemDetailsEditPlanner` to preserve runtime source paths from `DetailsEdit`. Validated with `cargo test -p squalr-engine-api project_item_details --lib --locked`.
- Current project metadata load pass defaulted additive `ProjectSymbolCatalog`, `SymbolicStructDefinition::layout_kind`, and `ValuedStruct::layout_kind` fields so older alpha project metadata/items can deserialize without bespoke migration code. Validated with focused legacy deserialization tests plus `cargo test -p squalr-engine-projects project_info --lib --locked`. Local smoke opened BFBB, Old Project, Shrek, Torchlight II, and winmine; Minesweeper still fails on an older `Expression` layout resolver shape, and FFCC still has the intentionally ignored legacy plugin array shape.
- Current ProjectItemAddress Details UI repair pass changed the Struct Viewer Details adapter to render project item property fields under their original property keys, while still routing edits by stable `DetailsFieldId`. This restores the existing data-type selector, virtual container rows, and live value preview/editor selection for Project Hierarchy Details projections. Validated with `cargo test -p squalr struct_viewer --lib --locked`, `cargo test -p squalr project_hierarchy --lib --locked`, and `cargo test -p squalr-engine-api project_item_details --lib --locked`.
- Current data viewer tooltip pass added a shared `ThemedTooltip` control helper and routed `ComboBoxView` label tooltips through it instead of `Response::on_hover_text`, so Details/data-type combo tooltips use app theme colors/fonts. Validated with `cargo test -p squalr data_type_selector --lib --locked` and `cargo test -p squalr struct_viewer --lib --locked`.
- Current Symbol Tree Details repair pass lets module roots focus metadata-only Details, keeps module roots out of the external array-value viewer, renders Symbol Tree type metadata through the legacy symbolic struct reference field so the data-type selector is populated, appends project symbol layouts to generic data-type selector options after the built-in grid, and omits nested runtime structs from value fields so PE aggregate children no longer collapse into comma previews. Validated with `cargo test -p squalr-engine-api symbol_tree_details --lib --locked`, `cargo test -p squalr data_type_selector --lib --locked`, `cargo test -p squalr struct_viewer --lib --locked`, and `cargo test -p squalr symbol_tree --lib --locked`.
- Current Symbol Tree readonly Details pass makes Symbol Tree metadata rows read-only, renders read-only data-type metadata as a value field instead of an editor, routes Details edits only for runtime value sources, and overrides module-root metadata type with the module-root layout id when present so source tree roots show layouts such as `winmine.exe` instead of `u8[size]`. Validated with `cargo test -p squalr-engine-api symbol_tree_details --lib --locked`, `cargo test -p squalr struct_viewer --lib --locked`, and `cargo test -p squalr symbol_tree --lib --locked`. Needs human verification in the GUI.
- Current data-type selector grouping pass treats registered data types as the two-column selector group, including `bool8`, `bool32`, `string_utf8`, and plugin-authored data types; only unregistered project struct/layout refs stay in the full-width custom section. Validated with `cargo test -p squalr data_type_selector --lib --locked` and `cargo test -p squalr struct_viewer --lib --locked`. Needs human verification in the GUI.
- Current Symbol Layout field details cleanup pass removes the editable `Hidden` field from Struct Viewer details, renames ambiguous `Offset`/`Static Offset` labels to `Offset Mode`/`Byte Offset`, and auto-grows a layout draft when fixed byte offset edits push fields past the declared layout size. Validated with `cargo test -p squalr symbol_layout_editor --lib --locked` and `cargo test -p squalr struct_viewer --lib --locked`. Needs human verification in the GUI.
- Current Symbol Layout offset display pass renders static layout field byte offsets as hex text by default and accepts hex text edits. Negative static layout field offsets remain rejected because `SymbolicFieldOffsetResolution::Static` and layout spans model unsigned positions within the layout; signed pointer offsets are handled by the pointer-chain model instead. Validated with `cargo test -p squalr symbol_layout_editor --lib --locked` and `cargo test -p squalr struct_viewer --lib --locked`. Needs human verification in the GUI.
- Current explicit unassigned layout pass changes `SymbolicFieldDefinition` into a field/unassigned enum so sparse layout space can be stored as `unassigned[...]` entries instead of only inferred from fixed offsets. Struct size/resolver/symbol-tree/runtime-write paths advance through unassigned entries without rendering them as editable value fields. Symbol Layout editor descriptor building now materializes forward static gaps as explicit unassigned entries and then stores the following field as sequential. Overlapping/static and resolver offsets remain available for cases that are not pure sparse sequential space. Validated with `cargo test -p squalr-engine-domain symbolic_struct --lib --locked`, `cargo test -p squalr-engine-api symbol_tree --lib --locked`, `cargo test -p squalr-engine project_symbol --lib --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, `cargo test -p squalr data_type_selector --lib --locked`, `cargo test -p squalr struct_viewer --lib --locked`, and `cargo check -p squalr --locked`. Needs human verification in the GUI.
- Current tooltip/placement cleanup pass routes shared Button/Checkbox/Slider tooltips plus Project Hierarchy row previews and Element Scanner cell previews through `ThemedTooltip` instead of egui `on_hover_text`; `rg "on_hover_text" squalr/src` now has no matches. Symbol Layout field details no longer expose `Offset Mode` / static byte-offset rows, and the old Struct Viewer offset-mode combo/presentation was removed so `Sequential` / `Static` do not leak as Details UI. Static placement remains an internal draft/descriptor concept for moves, overlaps, imports, and resolver materialization; sparse authoring is represented by explicit `unassigned[...]` rows. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, `cargo test -p squalr struct_viewer --lib --locked`, `cargo test -p squalr data_type_selector --lib --locked`, `cargo test -p squalr project_hierarchy --lib --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current module-root unassigned/save repair pass makes Symbol Layout editor split offsets part of dirty detection and descriptor persistence, restores persisted adjacent `unassigned[...]` split boundaries when reopening a layout, and passes active split offsets into save. Symbol Tree module-root expansion now uses the module-root layout descriptor as the rendered child source when present, preserving explicit split `UNASSIGNED` ranges and mapping mirrored promoted fields back to existing symbol claims at the same module offset to avoid duplicates. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr-engine-api symbol_tree --lib --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, `cargo test -p squalr symbol_tree --lib --locked`, `cargo test -p squalr struct_viewer --lib --locked`, and `git diff --check`. Needs human verification in the GUI with the `winmine.exe+579C -> promote to symbol -> edit winmine.exe layout` flow.
- Current PE symbol population repair pass keeps the plugin's legacy module-field write but also updates the module-root layout descriptor, so populating PE headers after a promoted/split module root inserts `PE Headers` at offset `0`, rebuilds the following explicit `unassigned[...]` gap, and preserves existing later fields such as `winmine.exe+579C`. Validated with `cargo fmt --all`, `cargo test -p squalr-plugin-symbols-pe --locked`, `cargo test -p squalr-engine execute_plugin_action_populates_pe_symbols --lib --locked`, `cargo test -p squalr-engine-api symbol_tree --lib --locked`, `cargo check -p squalr --locked`, and `git diff --check`. Needs human verification in the GUI with `winmine.exe+579C -> promote to symbol -> populate PE headers`.
