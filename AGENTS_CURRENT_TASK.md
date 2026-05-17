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

## Vestigial Code Audit

- Removed `StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_OFFSET_MODE` and `VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_STATIC_OFFSET`; Symbol Layout Details no longer exposes offset/static authoring rows.
- Removed `SymbolLayoutFieldEditDraft::is_hidden`, `SymbolicFieldDefinition::is_hidden`, `with_hidden`, and `" hidden"` parsing/storage. Non-rendered layout space is now represented by explicit `unassigned[...]` entries, while union visibility remains handled by resolver activation.
- Removed unproduced Details vocabulary: `DetailsEditorHint::{Code, ContainerType, SymbolResolver, SymbolLayout}` and `DetailsFieldSource::SymbolResolverMetadata`.
- `ProjectItemDetails` remains reachable for preview/value display helpers, memory/code context actions, runtime target resolution, and memory-read helpers. It is no longer the Project Hierarchy persisted-property Details bridge.
- Symbol Tree normal selections and external array/value-viewer selections now focus `SymbolTreeDetailsProjection`; the old view-owned `build_symbol_layout_*` / `ValuedStruct` external projection path was deleted.
- `SymbolTreeView::dispatch_memory_read_request` and `build_symbol_preview_snapshot_queries` remain live for preview refreshes; they are not part of the deleted Details bridge.
- `plugins/squalr-plugin-symbols-pe/src/populate_pe_symbols_action.rs::resolve_primitive_data_type_size` is a local size table used only by PE layout placement. It is not dead, but it duplicates data-type sizing knowledge and should eventually use a shared symbolic layout sizing service if one is introduced.

## Vestigial Cleanup Action Items

1. Done: removed the two dead Symbol Layout offset/static Struct Viewer virtual field constants and updated tests to assert absence without obsolete ids.
2. Done: removed hidden symbolic fields as a domain concept and kept reserved bytes as explicit `unassigned[...]`.
3. Done: tightened the Details model by removing unproduced editor/source variants.
4. Done: migrated Project Hierarchy multi-selection off `ProjectItemDetails::build_struct_view_properties` and removed the legacy persisted-property edit parser from `ProjectHierarchyView`.
5. Done: moved Symbol Tree external array/value-viewer projection to Details and deleted the remaining legacy `build_symbol_layout_*` path.
6. Consider extracting shared symbolic layout size estimation once PE placement and runtime/tree sizing need the same behavior.

## Symbol Layout Editor Cleanup Audit

`squalr/src/views/symbol_layout_editor/symbol_layout_editor_view.rs` should keep only top-level window composition, high-level shortcut routing, and handoff to named subviews/controllers. Do not create vague helper buckets; every extraction below needs an intent-revealing owner.

1. Done: moved Symbol Layout Struct Viewer Details focus out of the parent.
   - New home: `symbol_layout_editor_view/details/symbol_layout_details_focus.rs`.
   - Moved `clear_struct_viewer_if_symbol_layout_focused`, `focus_selected_layout_in_struct_viewer`, `build_struct_viewer_layout_edit_callback`, `focus_unassigned_span_in_struct_viewer`, `build_field_details`, and the test-only `build_field_details_struct`.
   - Notes: unassigned span Details now uses a named read-only edit callback instead of scattering no-op closures.

2. Done: moved unassigned span action handling beside the unassigned row.
   - New homes: `rows/symbol_layout_unassigned_row_action.rs` and `rows/symbol_layout_unassigned_context_menu.rs`.
   - Moved `SymbolLayoutUnassignedRowAction`, `render_unassigned_context_menu`, and the large unassigned action applier out of the parent.
   - Notes: the parent still collects the selected context target while `render_field_rows` owns traversal; the next tree-view extraction should remove that remaining coordination.

3. Done: moved field context menu handling out of the parent.
   - New home: `rows/symbol_layout_field_context_menu.rs`.
   - The menu emits `SymbolLayoutFieldRowAction`; the parent no longer owns menu labels, menu ids, delete eligibility, or move eligibility.

4. Done: extracted the field tree/list renderer.
   - New home: `rows/symbol_layout_draft_field_tree_view.rs`.
   - Moved `render_field_rows`, `render_union_variant_layout_rows`, `render_union_variant_child_row`, and `SymbolLayoutVariantLayoutRowAction`.
   - Notes: the draft field tree now walks fields/unassigned/variant child rows, collects row actions, and dispatches row action appliers. It still calls existing parent draft/session operations until those get their own owner.

5. Done: moved Symbol Layout list ownership out of the parent.
   - New home: `symbol_layout_editor_view/list/symbol_layout_list_panel_view.rs`.
   - Moved `render_list_panel`, `render_filter_text_box`, and parent-owned `SymbolLayoutRowAction`.
   - Notes: the list panel owns filtering/list-row composition and open/rename/delete handoff; layout row selection calls the layout Details focus handler.

6. Done: moved define-field and layout-edit controls into named controls/views.
   - New home: `symbol_layout_editor_view/controls/`.
   - Moved `render_string_value_box`, `render_u64_data_value_box`, `render_define_field_container_selector`, `render_define_field_type_combo`, `render_layout_kind_combo`, `render_layout_size_editor`, `render_add_entry_button`, and `render_centered_add_entry_button`.
   - Notes: `render_flat_icon_button` was removed after adding shared `IconButtonView`; type-picker search/grouping helpers now live with the define-field type combo.

7. Done: moved draft/session authoring operations out of the parent.
   - Former offenders: `append_field_to_variant_layout`, `resolve_variant_tail_unassigned_offset`, `resolve_draft_tail_unassigned_offset`, `build_symbolic_field_definition_from_draft`, `validate_define_field_draft`, and `resolve_draft_field_spans`.
   - Target home: `SymbolLayoutEditorViewData` for session state operations, engine-api draft ops for pure reusable operations, or named GUI-side controllers for draft factories/variant session overlays that still need `AppContext`.
   - Done this pass: moved the union variant session overlay into `authoring/symbol_layout_variant_session.rs`, including `create_union_variant_layout_draft_*`, `build_union_variant_layout_id`, pending draft cache/read/filter operations, effective catalog construction, pending descriptor materialization, and pending variant draft persistence.
   - Done this pass: moved default field draft construction into `authoring/symbol_layout_field_draft_factory.rs`, including default data type choice, layout-kind field draft creation, unassigned-span field draft creation, and union field normalization.
   - Done this pass: moved draft span resolution, tail-gap resolution, define-field validation, and field-definition materialization into `authoring/symbol_layout_draft_analyzer.rs`.
   - Done this pass: moved union variant field append/session mutation into `authoring/symbol_layout_variant_field_appender.rs`.
   - Notes: anything pure and useful to CLI/TUI belongs outside the GUI; anything GUI-only still needs a clear owner. Parent `symbol_layout_editor_view.rs` is now about 1,239 lines.

8. Move constants to their owners as extraction proceeds.
   - Current offenders: parent-level constants for takeovers, rows, list, context menus, define-field selector dimensions, and layout-kind combo widths.
   - Target home: each extracted view/control keeps its own dimensions unless the constant is genuinely shared UI design language.

9. Shrink the root `Widget::ui` method after the above.
   - Current offenders: top-level state snapshot, escape/enter/up/down/delete shortcut handling, and takeover/list routing.
   - Target home: keep the final parent as the window orchestrator, but move shortcut handling and takeover host composition only after their target views/controllers exist.
   - Notes: this is lower priority than removing the current Details, unassigned, context menu, and row-tree responsibilities.

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
   - Multi-selection Project Hierarchy details focus now adapts multiple `DetailsProjection`s and combines the rendered rows through the Struct Viewer compatibility layer.
   - Done: removed legacy `ProjectItemDetails::build_struct_view_properties`.

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
   - Done: removed the legacy `ProjectHierarchyView::apply_project_item_edits` fallback after multi-selection moved to Details.

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
   - Done: removed direct parsing of edited `ValuedStructField` names from the Project Hierarchy persisted-property fallback.
   - Keep tree rendering, selection, drag/drop, and context menus in the view until a separate reason exists to move them.

8. Migrate Symbol Tree to Details.
   - Added shared projector under `squalr-engine-api/src/structures/projects/symbol_tree/details/`.
   - `SymbolTreeDetailsProjection` covers symbol claim display-name metadata, type metadata, address/module/size/path fields, fallback locator/status fields, and normalized runtime value fields.
   - Normal readable Symbol Tree selections now focus `DetailsProjection` through `StructViewerViewData::focus_details_projection_with_focus_target`.
   - Symbol Tree Details metadata is read-only; symbol layout/name edits stay in the struct layout editor, symbol resolver tools, and inline rename paths.
   - Normal Details runtime value edits dispatch existing `ProjectSymbolsWriteValueRequest`.
   - External array/value-viewer rows now use `SymbolTreeDetailsProjection::build_external_value`, with the GUI adapter mapping array runtime values to the existing live-value row presentation.
   - Done: removed legacy metadata/location/value field construction from the old `build_symbol_layout_*` path.
   - Keep symbol tree rendering, expansion state, and selection in the GUI view.

9. Drain old virtual fields from Struct Viewer.
   - Remove project item pointer virtual fields after Project Hierarchy uses Details.
   - Remove symbol layout/resolver virtual fields after Symbol Tree and symbol authoring editors use Details.
   - Keep generic container/array display only if it is truly Struct Viewer behavior; otherwise model it as detail metadata too.

10. Only then slim the large views.
   - `ProjectHierarchyView` should select project items, render rows/context actions, and route user actions.
   - `SymbolTreeView` should select symbols, render rows/context actions, and route user actions.
   - Neither view should construct Details rows, parse edited field names, or build memory read/write requests directly.

11. In progress: move Symbol Layout authoring behavior out of the GUI view.
   - Extracted pure symbol-layout draft/span operations into `squalr-engine-api/src/structures/projects/symbol_layouts/symbol_layout_draft_ops.rs`.
   - `SymbolLayoutEditorView` now keeps rendering, selection, context menus, and command dispatch orchestration, while draft field/unassigned span movement/planning lives outside the view.
   - Added command-backed Symbol Layout persistence through `project_symbols upsert-layout` and `project_symbols delete-layout`.
   - Normal GUI Symbol Layout save/delete paths now dispatch those commands instead of replacing the entire symbol catalog from the view.
   - `SymbolLayoutEditorViewData` adapts GUI drafts into shared descriptor builder traits, so descriptor materialization, size parsing, and size walking are shared. It still owns editor session state, draft creation from descriptors, unassigned split bookkeeping, and GUI takeover transitions.
   - Done: union variant edits now stay in pending editor draft state, build an effective in-memory catalog for sizing/rendering, and persist referenced variant layouts only on Accept via `project_symbols upsert-layout`.
   - Done: removed the Symbol Layout editor's rollback-only `project_symbols set-catalog` usage and the take-over catalog side-effect flag.
   - Done: added command-backed Symbol Resolver upsert/delete persistence through `project_symbols upsert-resolver` and `project_symbols delete-resolver`.
   - Done: removed the whole-catalog `project_symbols set-catalog` command and executor after migrating GUI authoring callers off it.
   - Current investigation: `SymbolLayoutEditorView` is still about 5k lines and still owns legacy Struct Viewer projection/edit parsing for layout metadata, field metadata, variant fields, and unassigned spans through `ValuedStruct` plus `StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_*` names.
   - Done: added `SymbolLayoutDetails` in `squalr-engine-api/src/structures/projects/symbol_layouts/symbol_layout_details.rs` for layout/field/unassigned Details projection plus stable-id edit planning.
   - Done: selected Symbol Layout editor layout/field/variant/unassigned details now route through `StructViewerViewData::focus_details_projection_with_focus_target`; the old selected-details edit callbacks no longer parse `ValuedStructField` names.
   - Done: the Struct Viewer Details adapter now maps `DetailsFieldSource::SymbolLayoutMetadata` to existing Symbol Layout editor controls and preserves data-type selector state for Details-backed data-type fields.
   - Done: moved the Symbol Layout editor takeover screens into `squalr/src/views/symbol_layout_editor/symbol_layout_editor_view/takeovers/` with intent-revealing files: `layout_editor_takeover.rs`, `define_field_from_unassigned_takeover.rs`, `delete_symbol_layout_takeover.rs`, `delete_symbol_layout_field_takeover.rs`, and shared shell/actions in `takeover_panel.rs`.
   - Done: moved duplicated GUI text measurement and ellipsis fitting out of Symbol Layout editor, Symbol Resolver editor, Symbol Tree, Project Hierarchy, toolbar menus, and combo-box items into `squalr/src/ui/text/text_fitting.rs`.
   - Done: extracted Symbol Layout editor row rendering into row views under `squalr/src/views/symbol_layout_editor/symbol_layout_editor_view/rows/`: layout rows, field rows, unassigned rows, and union variant preview rows now render themselves, with field row actions applied by `symbol_layout_field_row_action.rs` instead of owner-side action matches.
   - Done: moved the Symbol Layout list header to shared `squalr/src/ui/widgets/controls/list_header.rs`; the editor now instantiates `ListHeaderView` instead of owning `render_list_header`.
   - Next clear cleanup: remove remaining Symbol Layout virtual field constants and legacy Symbol Layout presentation tests from `StructViewerViewData` after any remaining callers are confirmed off `focus_valued_struct_with_focus_target`.
   - Next clear extraction: move union variant draft transaction helpers, effective-catalog overlay construction, and field/default creation planners out of `SymbolLayoutEditorView` so the view mostly renders rows/takeover panels and dispatches command requests.

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
- Current Symbol Layout field move repair pass treats tail gaps after fields as explicit `unassigned[...]` rows and preserves the split boundary when a field moves across one unassigned row into another, preventing adjacent explicit unassigned rows from silently merging/disappearing. Variant layout persistence now also uses scoped unassigned split offsets. Validated with `cargo fmt --all`, `cargo test -p squalr symbol_layout_editor --lib --locked`, `cargo test -p squalr symbol_tree --lib --locked`, `cargo test -p squalr struct_viewer --lib --locked`, `cargo test -p squalr-engine-api symbol_tree --lib --locked`, `cargo check -p squalr --locked`, and `git diff --check`. Needs human verification in the GUI with `unassigned -> field -> unassigned`, then moving the field up/down.
- Current vestigial cleanup pass removed dead Symbol Layout offset/static virtual field constants, removed hidden symbolic-field parsing/storage/draft state, removed unproduced Details editor/source variants, migrated Project Hierarchy multi-selection Details off the legacy `ProjectItemDetails::build_struct_view_properties` bridge, removed `ProjectHierarchyView::apply_project_item_edits`, and moved Symbol Tree external array/value-viewer focus to `SymbolTreeDetailsProjection::build_external_value`. Added regression tests for external symbol array Details projection and adapter rendering. Validated with `cargo fmt --all`, `cargo test -p squalr-engine-domain symbolic_field --lib --locked`, `cargo test -p squalr-engine-api symbol_tree --lib --locked`, `cargo test -p squalr-engine-api details --lib --locked`, `cargo test -p squalr-engine-api project_item_details --lib --locked`, `cargo test -p squalr struct_viewer --lib --locked`, `cargo test -p squalr project_hierarchy --lib --locked`, `cargo test -p squalr symbol_tree --lib --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, `cargo check -p squalr --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current struct icon pass added `squalr/images/app/data_types/struct.png` to `IconLibrary` and routes known user-defined symbol layouts, including unions, through that icon instead of the unknown data-type icon. Symbol Tree nodes now carry layout-icon metadata for module-root layouts, symbol claims, nested fields, pointer targets, and array elements backed by project struct layouts. Project Explorer item icons, Struct Viewer symbol-layout selectors, generic DataTypeSelector custom-layout rows, Symbol Layout editor field rows/menus, and Symbol Tree define-field menus now use the struct icon for catalog-backed layouts. Validated with `cargo fmt --all`, `cargo test -p squalr-engine-api symbol_tree --lib --locked`, `cargo test -p squalr symbol_tree --lib --locked`, `cargo test -p squalr data_type_selector --lib --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, `cargo test -p squalr project_hierarchy --lib --locked`, `cargo test -p squalr struct_viewer --lib --locked`, `cargo check -p squalr --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout details icon pass routes the selected field details row and selected combo label for `SymbolLayoutFieldSymbolLayoutSelector` through the shared struct icon instead of resolving the custom layout id as a primitive data type. Validated with `cargo fmt --all`, `cargo test -p squalr struct_viewer --lib --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, `cargo check -p squalr --locked`, and `git diff --check`. Needs human verification in the GUI when double-clicking a symbol layout field whose element type is a struct/layout.
- Current Symbol Tree unassigned Details pass makes explicit `UNASSIGNED` segments metadata-only in Details: the shared projector refuses runtime value fields for unassigned nodes, and the GUI skips memory reads for unassigned selections while still showing size metadata. Validated with `cargo fmt --all`, `cargo test -p squalr-engine-api symbol_tree_details --lib --locked`, `cargo test -p squalr symbol_tree --lib --locked`, `cargo check -p squalr --locked`, and `git diff --check`. Needs human verification in the GUI when clicking an `UNASSIGNED` segment.
- Current Symbol Tree context-menu ordering pass moves `Edit Symbol Layout...` to the first row when available and adjusts separators around the remaining actions. Validated with `cargo fmt --all`, `cargo test -p squalr symbol_tree --lib --locked`, `cargo check -p squalr --locked`, and `git diff --check`. Needs human verification in the GUI context menu.
- Current Symbol Layout draft-ops extraction pass moved pure field insertion, unique naming, field move, unassigned-span move, and unassigned-row context planning out of `SymbolLayoutEditorView` into `symbol_layout_draft_ops.rs`. The view now calls `SymbolLayoutDraftOps` for those mutations while keeping GUI rendering/orchestration local. Validated with `cargo fmt --all`, `cargo test -p squalr symbol_layout_editor --lib --locked`, `cargo check -p squalr --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout command-authoring pass added `project_symbols upsert-layout` and `project_symbols delete-layout` request/response/executor/CLI paths, moved reusable layout catalog upsert/delete/retarget mutation into `ProjectSymbolLayoutMutation`, routed normal GUI layout save/delete through those commands, and removed the dead GUI-side catalog upsert/delete helpers. Validated with `cargo fmt --all`, `cargo test -p squalr-engine-api project_symbols_upsert_layout --lib --locked`, `cargo test -p squalr-engine project_symbol_layout_mutation --lib --locked`, `cargo test -p squalr-engine upsert_layout --lib --locked`, `cargo test -p squalr-engine delete_layout --lib --locked`, `cargo test -p squalr-cli parse_input --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, `cargo check -p squalr --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout variant-transaction pass removed the editor's rollback-only whole-catalog `set-catalog` path. Union variant field/detail edits now cache pending variant drafts in `SymbolLayoutEditorViewData`, render and size against an effective in-memory catalog, clear pending variant drafts on cancel, and upsert referenced variant layout descriptors only when the main Symbol Layout edit is accepted. Validated with `cargo fmt --all`, `cargo test -p squalr symbol_layout_editor --lib --locked`, and `cargo check -p squalr --locked`. Needs human verification in the GUI with union variant edit/cancel/save flows.
- Current Symbol Resolver command-authoring pass added `project_symbols upsert-resolver` and `project_symbols delete-resolver` request/response/executor/CLI paths, moved reusable resolver catalog mutation into `ProjectSymbolResolverMutation`, routed Symbol Resolver editor save/delete/name-edit through those commands, removed GUI-side resolver catalog replacement helpers, and deleted the old whole-catalog `project_symbols set-catalog` API/engine command. Validated with `cargo fmt --all`, `cargo test -p squalr-engine-api project_symbols --lib --locked`, `cargo test -p squalr-engine project_symbol_resolver_mutation --lib --locked`, `cargo test -p squalr-engine upsert_resolver --lib --locked`, `cargo test -p squalr-cli parse_input --locked`, `cargo test -p squalr symbol_resolver_editor --lib --locked`, `cargo check -p squalr --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout unassigned-move boundary pass inserts scoped unassigned split offsets when moving an explicit unassigned row across an adjacent field, preserving the moved row as a separate `unassigned[...]` entry if it lands next to another unassigned row. Validated with `cargo fmt --all`, `cargo test -p squalr symbol_layout_editor --lib --locked`, `cargo check -p squalr --locked`, and `git diff --check`. Needs human verification in the GUI with unassigned-row move-up/down flows.
- Current Symbol Layout shared-authoring pass moved pure draft/span operations into `squalr-engine-api/src/structures/projects/symbol_layouts/symbol_layout_draft_ops.rs`, removed the GUI-local ops module, and moved descriptor materialization/size validation into `symbol_layout_descriptor_builder.rs` behind API traits. `SymbolLayoutEditorViewData` now adapts its GUI draft to shared planner/builder traits instead of owning those rules. Validated with `cargo fmt --all`, `cargo test -p squalr-engine-api symbol_layout --lib --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, `cargo check -p squalr --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout size-resolver pass added `squalr-engine-api/src/structures/projects/symbol_layouts/symbol_layout_size_resolver.rs` and routed Symbol Tree preview sizing, project symbol runtime value write sizing, project symbol layout mutation sizing, and descriptor builder sizing through it. This removes duplicated symbolic struct/field size walkers while preserving each caller's own data-type and nested-layout lookup callbacks. Validated with `cargo fmt --all`, `cargo test -p squalr-engine-api symbol_layout --lib --locked`, `cargo test -p squalr symbol_tree --lib --locked`, `cargo test -p squalr-engine project_symbol --lib --locked`, `cargo check -p squalr --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout editor investigation pass audited recent refactor state. Shared pieces now include draft/span ops, descriptor building, size resolution, layout upsert/delete commands, resolver upsert/delete commands, and engine-side catalog mutation services. Remaining monolith work is concentrated in `squalr/src/views/symbol_layout_editor/symbol_layout_editor_view.rs`: legacy `ValuedStruct`/virtual-field Details projection, field edit parsing, union variant transaction helpers, effective catalog overlays, and GUI session/takeover orchestration. Validated with `cargo test -p squalr-engine-api symbol_layout --lib --locked` and `cargo test -p squalr symbol_layout_editor --lib --locked`. Needs human verification in the GUI.
- Current Symbol Layout Details routing pass added the shared `SymbolLayoutDetails` projection/edit-planning model, routed selected Symbol Layout editor details through the Struct Viewer Details adapter, and taught the adapter to map Symbol Layout metadata sources to existing specialized controls without relying on legacy virtual field names. The legacy field Details struct builder is now test-only. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr-engine-api symbol_layout --lib --locked`, `cargo test -p squalr struct_viewer --lib --locked`, and `cargo test -p squalr symbol_layout_editor --lib --locked`. Needs human verification in the GUI.
- Current Symbol Layout takeover naming pass replaced the vague `take_over_views.rs` extraction with intent-revealing sub-view modules under `symbol_layout_editor_view/takeovers/`: `layout_editor_takeover.rs`, `define_field_from_unassigned_takeover.rs`, `delete_symbol_layout_takeover.rs`, `delete_symbol_layout_field_takeover.rs`, and shared shell/actions in `takeover_panel.rs`. Parent `symbol_layout_editor_view.rs` still remains large at about 4.8k lines, so the next meaningful de-monolith work is still extracting real authoring responsibilities rather than helper buckets. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, and `cargo test -p squalr struct_viewer --lib --locked`. Needs human verification in the GUI.
- Current shared text-fitting pass moved generic egui text width measurement and ellipsis truncation into `squalr/src/ui/text/text_fitting.rs`, removing duplicated local helpers from Symbol Layout editor, Symbol Resolver editor, Symbol Tree rows, Project Hierarchy rows, toolbar menu items, and combo-box items. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, `cargo test -p squalr symbol_resolver_editor --lib --locked`, `cargo test -p squalr symbol_tree --lib --locked`, `cargo test -p squalr project_hierarchy --lib --locked`, `cargo test -p squalr toolbar_menu --lib --locked`, `cargo test -p squalr combo_box --lib --locked`, `cargo test -p squalr data_type_selector --lib --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout field-row action pass moved owner-side field action handling, variant field deletion, and field Struct Viewer focusing into `symbol_layout_editor_view/rows/symbol_layout_field_row_action.rs`. The parent now forwards row actions to the field action object instead of matching on field behavior itself; takeovers call the same field-focus helper rather than owning a `SymbolLayoutEditorView::focus_field_in_struct_viewer` method. Parent `symbol_layout_editor_view.rs` is now about 4.0k lines. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current shared list-header pass added `ListHeaderView` under `squalr/src/ui/widgets/controls/list_header.rs` and removed `SymbolLayoutEditorView::render_list_header`. The Symbol Layout editor now uses the same standalone-widget direction as other reusable UI chrome. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout list-toolbar pass moved `render_list_toolbar` into `symbol_layout_editor_view/toolbars/symbol_layout_list_toolbar_view.rs`. The toolbar now owns its draw chrome and create-layout click handling while the parent list panel composes it as a widget. Parent `symbol_layout_editor_view.rs` is now 3,954 lines. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout field-details edit pass moved `build_struct_viewer_field_edit_callback`, `build_variant_field_edit_callback`, field Details edit application, and draft auto-grow after field edits out of `SymbolLayoutEditorView` and into `symbol_layout_editor_view/rows/symbol_layout_field_row_action.rs`, beside the field-entry focus/action handling that feeds Struct Viewer. Parent `symbol_layout_editor_view.rs` is now 3,707 lines. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout editor cleanup audit pass added the "Symbol Layout Editor Cleanup Audit" section so remaining parent-file cleanup targets are tracked in one place: Details focus handlers, unassigned row actions, context menus, field-tree rendering, list panel ownership, define-field controls, draft/session authoring operations, constants, and final root widget shrinkage. Validated with source inspection and `git diff --check`; no code behavior changed.
- Current Symbol Layout details/unassigned/menu extraction pass moved layout/field/unassigned Struct Viewer Details focus into `symbol_layout_editor_view/details/symbol_layout_details_focus.rs`, moved unassigned row action application into `rows/symbol_layout_unassigned_row_action.rs`, moved unassigned context menu rendering into `rows/symbol_layout_unassigned_context_menu.rs`, and moved field context menu rendering into `rows/symbol_layout_field_context_menu.rs`. Parent `symbol_layout_editor_view.rs` is now 2,991 lines. The next clear bite is extracting `render_field_rows` / union child row traversal into a named draft field tree view. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout icon/tree extraction pass added shared `IconButtonView` under `squalr/src/ui/widgets/controls/icon_button.rs`, removed `SymbolLayoutEditorView::render_flat_icon_button`, routed Symbol Layout row/list-toolbar/add-entry icon buttons through the shared widget, and moved draft field/unassigned/variant row traversal into `rows/symbol_layout_draft_field_tree_view.rs`. Parent `symbol_layout_editor_view.rs` is now 2,439 lines. The next clear bite is moving `render_list_panel` / `render_filter_text_box` and `SymbolLayoutRowAction` into `symbol_layout_editor_view/list/`. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout list-panel extraction pass moved filtering, list toolbar/header/row composition, row action handling, and layout Details focusing into `symbol_layout_editor_view/list/symbol_layout_list_panel_view.rs`; `SymbolLayoutRowAction` now lives with `SymbolLayoutRowView`. Parent `symbol_layout_editor_view.rs` is now 2,323 lines. The next clear bite is moving the define-field/layout-edit controls and their type-picker helpers out of the parent. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout controls extraction pass moved value boxes, layout kind combo, add-entry buttons, define-field container selector, and define-field type picker/search/grid into `symbol_layout_editor_view/controls/`. Parent `symbol_layout_editor_view.rs` is now 1,842 lines. Remaining large cleanup is draft/session authoring operations and then root widget shortcut/routing shrinkage. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout variant-session extraction pass moved pending union variant draft/session overlay logic into `symbol_layout_editor_view/authoring/symbol_layout_variant_session.rs`, including pending draft cache/read/filter operations, union variant draft materialization, effective catalog construction, and pending descriptor validation/materialization. Parent `symbol_layout_editor_view.rs` is now 1,565 lines. Remaining authoring cleanup is field draft factories, append/tail-offset helpers, draft span resolution, define-field validation, and union field normalization. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout field-draft factory pass moved default data type selection, field draft creation for structs/unions, unassigned-span draft creation, and union field normalization into `symbol_layout_editor_view/authoring/symbol_layout_field_draft_factory.rs`. Parent `symbol_layout_editor_view.rs` is now 1,464 lines. Remaining authoring cleanup is append/tail-offset helpers, draft span resolution, and define-field validation. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, and `git diff --check`. Needs human verification in the GUI.
- Current Symbol Layout draft-analysis extraction pass moved draft field-definition materialization, define-field validation, field-span/tail-gap resolution, and variant field append mutation into `symbol_layout_editor_view/authoring/` (`symbol_layout_draft_analyzer.rs` and `symbol_layout_variant_field_appender.rs`). Parent `symbol_layout_editor_view.rs` is now 1,239 lines; remaining cleanup is mostly constants ownership and root `Widget::ui` shortcut/routing shrinkage. Validated with `cargo fmt --all`, `cargo check -p squalr --locked`, `cargo test -p squalr symbol_layout_editor --lib --locked`, and `git diff --check`. Needs human verification in the GUI.
