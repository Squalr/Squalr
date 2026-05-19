use crate::app_context::AppContext;
use crate::views::struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData};
use crate::views::symbol_layout_editor::symbol_layout_command_dispatcher::SymbolLayoutCommandDispatcher;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutEditDraft, SymbolLayoutEditorViewData, SymbolLayoutFieldEditDraft, SymbolLayoutFieldElementType, SymbolLayoutFieldOffsetMode,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use squalr_engine_api::structures::{
    details::DetailsEdit,
    projects::{
        project_symbol_catalog::ProjectSymbolCatalog,
        symbol_layouts::symbol_layout_details::{
            SymbolLayoutDetails, SymbolLayoutDetailsEditOperation, SymbolLayoutDetailsFieldContainerKind, SymbolLayoutDetailsFieldElementKind,
            SymbolLayoutFieldDetails,
        },
    },
    structs::symbolic_struct_definition::{SymbolicLayoutKind, SymbolicStructDefinition},
};
use std::sync::Arc;

use super::super::SymbolLayoutEditorView;
use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::SymbolLayoutFieldContainerKind;

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn clear_struct_viewer_if_symbol_layout_focused(
    struct_viewer_view_data: Dependency<StructViewerViewData>
) {
    let is_symbol_layout_focused = struct_viewer_view_data
        .read("SymbolLayoutEditor check details focus")
        .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned())
        .is_some_and(|focus_target| matches!(focus_target, StructViewerFocusTarget::SymbolLayoutEditor { .. }));

    if is_symbol_layout_focused {
        StructViewerViewData::clear_focus(struct_viewer_view_data);
    }
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn focus_selected_layout_in_struct_viewer(
    app_context: Arc<AppContext>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    selected_layout_id: Option<&str>,
) {
    let Some(selected_layout_id) = selected_layout_id else {
        clear_struct_viewer_if_symbol_layout_focused(struct_viewer_view_data);
        return;
    };
    let Some(struct_layout_descriptor) = project_symbol_catalog
        .get_struct_layout_descriptors()
        .iter()
        .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == selected_layout_id)
    else {
        clear_struct_viewer_if_symbol_layout_focused(struct_viewer_view_data);
        return;
    };

    let details_projection = SymbolLayoutDetails::build_layout_projection(
        struct_layout_descriptor.get_struct_layout_id(),
        struct_layout_descriptor
            .get_struct_layout_definition()
            .get_layout_kind(),
    );
    let selection_key = format!("layout|{}", struct_layout_descriptor.get_struct_layout_id());
    let edit_callback = build_struct_viewer_layout_edit_callback(
        app_context.clone(),
        struct_viewer_view_data.clone(),
        struct_layout_descriptor.get_struct_layout_id().to_string(),
    );

    focus_details_projection(&app_context, &struct_viewer_view_data, details_projection, edit_callback, selection_key);
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn focus_unassigned_span_in_struct_viewer(
    app_context: Arc<AppContext>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
    draft: &SymbolLayoutEditDraft,
    offset_in_bytes: u64,
    size_in_bytes: u64,
) {
    let details_projection = SymbolLayoutDetails::build_unassigned_projection(&draft.layout_id, offset_in_bytes, size_in_bytes);
    let selection_key = format!("unassigned|{}|{}|{}", draft.layout_id, offset_in_bytes, size_in_bytes);

    focus_details_projection(
        &app_context,
        &struct_viewer_view_data,
        details_projection,
        read_only_details_edit_callback(),
        selection_key,
    );
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn build_field_details(
    project_symbol_catalog: &ProjectSymbolCatalog,
    layout_kind: SymbolicLayoutKind,
    field_draft: &SymbolLayoutFieldEditDraft,
) -> SymbolLayoutFieldDetails {
    let element_type = SymbolLayoutEditorViewData::resolve_field_element_type(project_symbol_catalog, field_draft);
    let fixed_array_length = matches!(
        field_draft.container_edit.kind,
        SymbolLayoutFieldContainerKind::FixedArray | SymbolLayoutFieldContainerKind::FixedPointerArray
    )
    .then(|| {
        field_draft
            .container_edit
            .fixed_array_length
            .trim()
            .parse::<u64>()
            .unwrap_or(1)
            .max(1)
    });
    let uses_count_resolver = matches!(
        field_draft.container_edit.kind,
        SymbolLayoutFieldContainerKind::DynamicArray | SymbolLayoutFieldContainerKind::DynamicPointerArray
    );
    let uses_display_count_resolver = matches!(
        field_draft.container_edit.kind,
        SymbolLayoutFieldContainerKind::FixedArray
            | SymbolLayoutFieldContainerKind::FixedPointerArray
            | SymbolLayoutFieldContainerKind::DynamicArray
            | SymbolLayoutFieldContainerKind::DynamicPointerArray
    );
    let uses_pointer_size = matches!(
        field_draft.container_edit.kind,
        SymbolLayoutFieldContainerKind::Pointer | SymbolLayoutFieldContainerKind::FixedPointerArray | SymbolLayoutFieldContainerKind::DynamicPointerArray
    );

    SymbolLayoutFieldDetails {
        field_name: field_draft.field_name.clone(),
        data_type_id: field_draft
            .data_type_selection
            .visible_data_type()
            .get_data_type_id()
            .to_string(),
        element_kind: match element_type {
            SymbolLayoutFieldElementType::BuiltInDataType => SymbolLayoutDetailsFieldElementKind::BuiltInDataType,
            SymbolLayoutFieldElementType::SymbolLayout => SymbolLayoutDetailsFieldElementKind::SymbolLayout,
        },
        container_kind: symbol_layout_details_container_kind_from_edit_kind(field_draft.container_edit.kind),
        fixed_array_length,
        count_resolver_id: uses_count_resolver.then(|| {
            field_draft
                .container_edit
                .dynamic_array_count_resolver_id
                .clone()
        }),
        display_count_resolver_id: uses_display_count_resolver.then(|| field_draft.container_edit.display_count_resolver_id.clone()),
        active_when_resolver_id: (layout_kind.is_union() || !field_draft.active_when_resolver_id.is_empty())
            .then(|| field_draft.active_when_resolver_id.clone()),
        pointer_size_label: uses_pointer_size.then(|| field_draft.container_edit.pointer_size.to_string()),
        offset_resolver_id: (field_draft.offset_mode == SymbolLayoutFieldOffsetMode::Resolver).then(|| field_draft.offset_resolver_id.clone()),
    }
}

fn symbol_layout_details_container_kind_from_edit_kind(container_kind: SymbolLayoutFieldContainerKind) -> SymbolLayoutDetailsFieldContainerKind {
    match container_kind {
        SymbolLayoutFieldContainerKind::Element => SymbolLayoutDetailsFieldContainerKind::Element,
        SymbolLayoutFieldContainerKind::Array => SymbolLayoutDetailsFieldContainerKind::Array,
        SymbolLayoutFieldContainerKind::FixedArray => SymbolLayoutDetailsFieldContainerKind::FixedArray,
        SymbolLayoutFieldContainerKind::DynamicArray => SymbolLayoutDetailsFieldContainerKind::DynamicArray,
        SymbolLayoutFieldContainerKind::Pointer => SymbolLayoutDetailsFieldContainerKind::Pointer,
        SymbolLayoutFieldContainerKind::FixedPointerArray => SymbolLayoutDetailsFieldContainerKind::FixedPointerArray,
        SymbolLayoutFieldContainerKind::DynamicPointerArray => SymbolLayoutDetailsFieldContainerKind::DynamicPointerArray,
    }
}

fn build_struct_viewer_layout_edit_callback(
    app_context: Arc<AppContext>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
    layout_id: String,
) -> Arc<dyn Fn(DetailsEdit) + Send + Sync> {
    Arc::new(move |details_edit: DetailsEdit| {
        let SymbolLayoutDetailsEditOperation::UpdateLayoutKind(edited_layout_kind) = SymbolLayoutDetails::plan_edit(&details_edit) else {
            return;
        };
        let Some(project_symbol_catalog) = SymbolLayoutEditorView::get_opened_project_symbol_catalog_from_context(&app_context) else {
            return;
        };

        let mut did_update_layout = false;
        let updated_struct_layout_descriptors = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .map(|struct_layout_descriptor| {
                if struct_layout_descriptor.get_struct_layout_id() != layout_id {
                    return struct_layout_descriptor.clone();
                }

                let struct_layout_definition = struct_layout_descriptor.get_struct_layout_definition();
                if struct_layout_definition.get_layout_kind() == edited_layout_kind {
                    return struct_layout_descriptor.clone();
                }

                did_update_layout = true;
                StructLayoutDescriptor::new(
                    struct_layout_descriptor.get_struct_layout_id().to_string(),
                    SymbolicStructDefinition::new_with_layout_kind(
                        struct_layout_definition.get_symbol_namespace().to_string(),
                        edited_layout_kind,
                        struct_layout_definition.get_fields().to_vec(),
                    )
                    .with_declared_size_in_bytes(struct_layout_definition.get_declared_size_in_bytes()),
                )
            })
            .collect::<Vec<_>>();

        if !did_update_layout {
            return;
        }

        let mut updated_project_symbol_catalog = project_symbol_catalog;
        updated_project_symbol_catalog.set_struct_layout_descriptors(updated_struct_layout_descriptors);

        let Some(updated_struct_layout_descriptor) = updated_project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id)
        else {
            return;
        };
        SymbolLayoutCommandDispatcher::new(app_context.clone()).persist_symbol_layout_descriptor(Some(layout_id.clone()), updated_struct_layout_descriptor);
        let details_projection = SymbolLayoutDetails::build_layout_projection(
            updated_struct_layout_descriptor.get_struct_layout_id(),
            updated_struct_layout_descriptor
                .get_struct_layout_definition()
                .get_layout_kind(),
        );
        let selection_key = format!("layout|{}", updated_struct_layout_descriptor.get_struct_layout_id());
        let edit_callback = build_struct_viewer_layout_edit_callback(app_context.clone(), struct_viewer_view_data.clone(), layout_id.clone());

        focus_details_projection(&app_context, &struct_viewer_view_data, details_projection, edit_callback, selection_key);
    })
}

fn read_only_details_edit_callback() -> Arc<dyn Fn(DetailsEdit) + Send + Sync> {
    Arc::new(|_details_edit| {})
}

fn focus_details_projection(
    app_context: &Arc<AppContext>,
    struct_viewer_view_data: &Dependency<StructViewerViewData>,
    details_projection: squalr_engine_api::structures::details::details_projection::DetailsProjection,
    edit_callback: Arc<dyn Fn(DetailsEdit) + Send + Sync>,
    selection_key: String,
) {
    StructViewerViewData::focus_details_projection_with_focus_target(
        struct_viewer_view_data.clone(),
        app_context.engine_unprivileged_state.clone(),
        details_projection,
        edit_callback,
        Some(StructViewerFocusTarget::SymbolLayoutEditor { selection_key }),
    );
}
