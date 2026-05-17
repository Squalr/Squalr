use crate::app_context::AppContext;
use crate::views::struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData};
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutEditDraft, SymbolLayoutEditorViewData, SymbolLayoutFieldEditDraft, SymbolLayoutFieldElementType,
};
use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::SymbolLayoutFieldContainerKind;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::{
    data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef},
    data_values::anonymous_value_string_format::AnonymousValueStringFormat,
    details::DetailsEdit,
    pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
    projects::{
        project_symbol_catalog::ProjectSymbolCatalog,
        symbol_layouts::{
            symbol_layout_details::{SymbolLayoutDetails, SymbolLayoutDetailsEditOperation},
            symbol_layout_draft_ops::{SymbolLayoutDraftOps, SymbolLayoutFieldSpan},
        },
    },
    structs::{symbolic_field_definition::SymbolicFieldOffsetResolution, symbolic_struct_definition::SymbolicLayoutKind},
};
use std::{collections::BTreeSet, str::FromStr, sync::Arc};

use super::super::SymbolLayoutEditorView;
use super::super::authoring::symbol_layout_field_draft_factory::SymbolLayoutFieldDraftFactory;
use super::super::authoring::symbol_layout_variant_session::SymbolLayoutVariantSession;
use super::super::details::symbol_layout_details_focus::{
    build_field_details, clear_struct_viewer_if_symbol_layout_focused, focus_unassigned_span_in_struct_viewer,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) enum SymbolLayoutFieldRowAction {
    InsertAfter,
    InsertFieldIntoVariant,
    RequestRemoveFieldConfirmation,
    MoveUp,
    MoveDown,
    SelectField,
}

impl SymbolLayoutFieldRowAction {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn apply_to_variant_layout(
        self,
        symbol_layout_editor_view: &SymbolLayoutEditorView,
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &SymbolLayoutEditDraft,
        variant_layout_id: String,
        field_index: usize,
    ) {
        let mut variant_draft = SymbolLayoutVariantSession::create_union_variant_layout_draft_for_id_with_pending(
            project_symbol_catalog,
            symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
            union_draft,
            &variant_layout_id,
        );
        let unassigned_split_offsets = symbol_layout_editor_view
            .symbol_layout_editor_view_data
            .read("SymbolLayoutEditor variant field split offsets")
            .map(|symbol_layout_editor_view_data| symbol_layout_editor_view_data.get_unassigned_split_offsets_for_layout(Some(&variant_layout_id)))
            .unwrap_or_default();
        let mut field_index_to_focus = None;
        let mut should_persist_variant_draft = false;

        match self {
            SymbolLayoutFieldRowAction::SelectField => {
                field_index_to_focus = Some(field_index);
            }
            SymbolLayoutFieldRowAction::MoveUp => {
                if let Some((layout_size_in_bytes, field_spans)) = symbol_layout_editor_view.resolve_draft_field_spans(project_symbol_catalog, &variant_draft)
                    && SymbolLayoutDraftOps::move_struct_field_up(&mut variant_draft, &field_spans, &unassigned_split_offsets, field_index)
                {
                    if let Some(split_offset_in_bytes) =
                        SymbolLayoutDraftOps::split_offset_to_preserve_field_move_up(&field_spans, layout_size_in_bytes, &unassigned_split_offsets, field_index)
                    {
                        SymbolLayoutEditorViewData::insert_unassigned_split_offset_for_layout(
                            symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                            Some(variant_layout_id.clone()),
                            split_offset_in_bytes,
                        );
                    }
                    field_index_to_focus = Some(field_index);
                    should_persist_variant_draft = true;
                }
            }
            SymbolLayoutFieldRowAction::MoveDown => {
                if let Some((layout_size_in_bytes, field_spans)) = symbol_layout_editor_view.resolve_draft_field_spans(project_symbol_catalog, &variant_draft)
                    && SymbolLayoutDraftOps::move_struct_field_down(
                        &mut variant_draft,
                        &field_spans,
                        layout_size_in_bytes,
                        &unassigned_split_offsets,
                        field_index,
                    )
                {
                    if let Some(split_offset_in_bytes) = SymbolLayoutDraftOps::split_offset_to_preserve_field_move_down(
                        &field_spans,
                        layout_size_in_bytes,
                        &unassigned_split_offsets,
                        field_index,
                    ) {
                        SymbolLayoutEditorViewData::insert_unassigned_split_offset_for_layout(
                            symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                            Some(variant_layout_id.clone()),
                            split_offset_in_bytes,
                        );
                    }
                    field_index_to_focus = Some(field_index);
                    should_persist_variant_draft = true;
                }
            }
            SymbolLayoutFieldRowAction::InsertAfter => {
                let insert_index = field_index
                    .saturating_add(1)
                    .min(variant_draft.field_drafts.len());
                let mut field_draft = SymbolLayoutFieldDraftFactory::create_field_draft_for_layout_kind(
                    &symbol_layout_editor_view.app_context,
                    project_symbol_catalog,
                    SymbolicLayoutKind::Struct,
                    &variant_draft.layout_id,
                    insert_index,
                );
                field_draft.field_name = SymbolLayoutDraftOps::build_unique_field_name(&variant_draft, &field_draft.field_name);
                variant_draft.field_drafts.insert(insert_index, field_draft);
                field_index_to_focus = Some(insert_index);
                should_persist_variant_draft = true;
            }
            SymbolLayoutFieldRowAction::RequestRemoveFieldConfirmation => {
                delete_variant_field(symbol_layout_editor_view, project_symbol_catalog, &mut variant_draft, field_index);
                return;
            }
            SymbolLayoutFieldRowAction::InsertFieldIntoVariant => {}
        }

        if should_persist_variant_draft {
            SymbolLayoutVariantSession::persist_variant_layout_draft(symbol_layout_editor_view.symbol_layout_editor_view_data.clone(), &variant_draft);
        }

        if let Some(field_index_to_focus) = field_index_to_focus {
            SymbolLayoutEditorViewData::select_field_for_layout(
                symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                Some(variant_layout_id),
                field_index_to_focus,
            );
            focus_variant_field_in_struct_viewer(symbol_layout_editor_view, project_symbol_catalog, &variant_draft, field_index_to_focus);
        }
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn apply_to_layout_draft(
        self,
        symbol_layout_editor_view: &SymbolLayoutEditorView,
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &mut SymbolLayoutEditDraft,
        field_index: usize,
        field_spans: Option<&(u64, Vec<SymbolLayoutFieldSpan>)>,
        unassigned_split_offsets: &BTreeSet<u64>,
    ) {
        let mut field_index_to_focus = None;
        match self {
            SymbolLayoutFieldRowAction::InsertAfter => {
                let insert_index = field_index.saturating_add(1).min(draft.field_drafts.len());
                let mut field_draft = SymbolLayoutFieldDraftFactory::create_field_draft_for_layout_kind(
                    &symbol_layout_editor_view.app_context,
                    project_symbol_catalog,
                    draft.layout_kind,
                    &draft.layout_id,
                    insert_index,
                );
                field_draft.field_name = SymbolLayoutDraftOps::build_unique_field_name(draft, &field_draft.field_name);
                draft.field_drafts.insert(insert_index, field_draft);
                field_index_to_focus = Some(insert_index);
            }
            SymbolLayoutFieldRowAction::InsertFieldIntoVariant => {
                if let Some((variant_draft, variant_field_index)) =
                    symbol_layout_editor_view.append_field_to_variant_layout(project_symbol_catalog, draft, field_index)
                {
                    SymbolLayoutEditorViewData::select_field_for_layout(
                        symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                        Some(variant_draft.layout_id.clone()),
                        variant_field_index,
                    );
                    focus_variant_field_in_struct_viewer(symbol_layout_editor_view, project_symbol_catalog, &variant_draft, variant_field_index);
                }
            }
            SymbolLayoutFieldRowAction::RequestRemoveFieldConfirmation => {
                SymbolLayoutEditorViewData::request_field_delete_confirmation(
                    symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                    draft.layout_id.clone(),
                    field_index,
                );
                field_index_to_focus = Some(field_index);
            }
            SymbolLayoutFieldRowAction::MoveUp => {
                if !draft.layout_kind.is_union() {
                    if let Some((layout_size_in_bytes, field_spans)) = field_spans
                        && SymbolLayoutDraftOps::move_struct_field_up(draft, field_spans, unassigned_split_offsets, field_index)
                    {
                        if let Some(split_offset_in_bytes) = SymbolLayoutDraftOps::split_offset_to_preserve_field_move_up(
                            field_spans,
                            *layout_size_in_bytes,
                            unassigned_split_offsets,
                            field_index,
                        ) {
                            SymbolLayoutEditorViewData::insert_unassigned_split_offset_for_layout(
                                symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                                None,
                                split_offset_in_bytes,
                            );
                        }
                        field_index_to_focus = Some(field_index);
                    }
                } else if field_index > 0 {
                    draft.field_drafts.swap(field_index, field_index - 1);
                    field_index_to_focus = Some(field_index - 1);
                }
            }
            SymbolLayoutFieldRowAction::MoveDown => {
                if !draft.layout_kind.is_union() {
                    if let Some((layout_size_in_bytes, field_spans)) = field_spans
                        && SymbolLayoutDraftOps::move_struct_field_down(draft, field_spans, *layout_size_in_bytes, unassigned_split_offsets, field_index)
                    {
                        if let Some(split_offset_in_bytes) = SymbolLayoutDraftOps::split_offset_to_preserve_field_move_down(
                            field_spans,
                            *layout_size_in_bytes,
                            unassigned_split_offsets,
                            field_index,
                        ) {
                            SymbolLayoutEditorViewData::insert_unassigned_split_offset_for_layout(
                                symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                                None,
                                split_offset_in_bytes,
                            );
                        }
                        field_index_to_focus = Some(field_index);
                    }
                } else if field_index + 1 < draft.field_drafts.len() {
                    draft.field_drafts.swap(field_index, field_index + 1);
                    field_index_to_focus = Some(field_index + 1);
                }
            }
            SymbolLayoutFieldRowAction::SelectField => {
                field_index_to_focus = Some(field_index);
            }
        }

        if let Some(field_index_to_focus) = field_index_to_focus {
            SymbolLayoutEditorViewData::select_field(symbol_layout_editor_view.symbol_layout_editor_view_data.clone(), field_index_to_focus);
            focus_field_in_struct_viewer(symbol_layout_editor_view, project_symbol_catalog, draft, field_index_to_focus);
        }
    }
}

fn delete_variant_field(
    symbol_layout_editor_view: &SymbolLayoutEditorView,
    project_symbol_catalog: &ProjectSymbolCatalog,
    variant_draft: &mut SymbolLayoutEditDraft,
    field_index: usize,
) {
    if field_index >= variant_draft.field_drafts.len() {
        return;
    }

    variant_draft.field_drafts.remove(field_index);
    if !SymbolLayoutVariantSession::persist_variant_layout_draft(symbol_layout_editor_view.symbol_layout_editor_view_data.clone(), variant_draft) {
        return;
    }

    if !variant_draft.field_drafts.is_empty() {
        let field_index_to_focus = field_index.min(variant_draft.field_drafts.len().saturating_sub(1));
        SymbolLayoutEditorViewData::select_field_for_layout(
            symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
            Some(variant_draft.layout_id.clone()),
            field_index_to_focus,
        );
        focus_variant_field_in_struct_viewer(symbol_layout_editor_view, project_symbol_catalog, variant_draft, field_index_to_focus);
        return;
    }

    if let Ok(layout_size_in_bytes) = SymbolLayoutEditorViewData::parse_layout_size_text(&variant_draft.size_text, variant_draft.size_format) {
        SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
            symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
            Some(variant_draft.layout_id.clone()),
            0,
            layout_size_in_bytes,
        );
        focus_unassigned_span_in_struct_viewer(
            symbol_layout_editor_view.app_context.clone(),
            symbol_layout_editor_view.struct_viewer_view_data.clone(),
            variant_draft,
            0,
            layout_size_in_bytes,
        );
    } else {
        SymbolLayoutEditorViewData::clear_field_selection(symbol_layout_editor_view.symbol_layout_editor_view_data.clone());
        clear_struct_viewer_if_symbol_layout_focused(symbol_layout_editor_view.struct_viewer_view_data.clone());
    }
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn focus_field_in_struct_viewer(
    symbol_layout_editor_view: &SymbolLayoutEditorView,
    project_symbol_catalog: &ProjectSymbolCatalog,
    draft: &SymbolLayoutEditDraft,
    field_index: usize,
) {
    let Some(field_draft) = draft.field_drafts.get(field_index) else {
        clear_struct_viewer_if_symbol_layout_focused(symbol_layout_editor_view.struct_viewer_view_data.clone());
        return;
    };

    let details_projection = SymbolLayoutDetails::build_field_projection(
        &draft.layout_id,
        field_index,
        draft.layout_kind,
        &build_field_details(project_symbol_catalog, draft.layout_kind, field_draft),
    );
    let selection_key = format!("field|{}|{}", draft.layout_id, field_index);
    let edit_callback = build_struct_viewer_field_edit_callback(
        symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
        symbol_layout_editor_view.struct_viewer_view_data.clone(),
        symbol_layout_editor_view.app_context.clone(),
        field_index,
    );

    focus_details_projection(
        &symbol_layout_editor_view.app_context,
        &symbol_layout_editor_view.struct_viewer_view_data,
        details_projection,
        edit_callback,
        selection_key,
    );
}

fn focus_variant_field_in_struct_viewer(
    symbol_layout_editor_view: &SymbolLayoutEditorView,
    project_symbol_catalog: &ProjectSymbolCatalog,
    variant_draft: &SymbolLayoutEditDraft,
    field_index: usize,
) {
    let Some(field_draft) = variant_draft.field_drafts.get(field_index) else {
        clear_struct_viewer_if_symbol_layout_focused(symbol_layout_editor_view.struct_viewer_view_data.clone());
        return;
    };

    let details_projection = SymbolLayoutDetails::build_field_projection(
        &variant_draft.layout_id,
        field_index,
        SymbolicLayoutKind::Struct,
        &build_field_details(project_symbol_catalog, SymbolicLayoutKind::Struct, field_draft),
    );
    let selection_key = format!("field|{}|{}", variant_draft.layout_id, field_index);
    let edit_callback = build_variant_field_edit_callback(
        symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
        symbol_layout_editor_view.struct_viewer_view_data.clone(),
        symbol_layout_editor_view.app_context.clone(),
        variant_draft.layout_id.clone(),
        field_index,
    );

    focus_details_projection(
        &symbol_layout_editor_view.app_context,
        &symbol_layout_editor_view.struct_viewer_view_data,
        details_projection,
        edit_callback,
        selection_key,
    );
}

fn build_struct_viewer_field_edit_callback(
    symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
    app_context: Arc<AppContext>,
    field_index: usize,
) -> Arc<dyn Fn(DetailsEdit) + Send + Sync> {
    Arc::new(move |details_edit: DetailsEdit| {
        let updated_draft = {
            let Some(mut view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor apply field details edit") else {
                return;
            };
            let Some(mut draft) = view_data.get_draft().cloned() else {
                return;
            };
            let Some(field_draft) = draft.field_drafts.get_mut(field_index) else {
                return;
            };

            let project_symbol_catalog = SymbolLayoutEditorView::get_opened_project_symbol_catalog_from_context(&app_context).unwrap_or_default();
            apply_field_details_operation(&project_symbol_catalog, field_draft, SymbolLayoutDetails::plan_edit(&details_edit));
            grow_draft_size_to_fit_fields(&project_symbol_catalog, &mut draft);
            view_data.replace_draft(draft.clone());
            draft
        };

        let Some(updated_field_draft) = updated_draft.field_drafts.get(field_index) else {
            return;
        };
        let project_symbol_catalog = SymbolLayoutEditorView::get_opened_project_symbol_catalog_from_context(&app_context).unwrap_or_default();
        let details_projection = SymbolLayoutDetails::build_field_projection(
            &updated_draft.layout_id,
            field_index,
            updated_draft.layout_kind,
            &build_field_details(&project_symbol_catalog, updated_draft.layout_kind, updated_field_draft),
        );
        let selection_key = format!("field|{}|{}", updated_draft.layout_id, field_index);
        let edit_callback = build_struct_viewer_field_edit_callback(
            symbol_layout_editor_view_data.clone(),
            struct_viewer_view_data.clone(),
            app_context.clone(),
            field_index,
        );

        focus_details_projection(&app_context, &struct_viewer_view_data, details_projection, edit_callback, selection_key);
    })
}

fn build_variant_field_edit_callback(
    symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
    app_context: Arc<AppContext>,
    variant_layout_id: String,
    field_index: usize,
) -> Arc<dyn Fn(DetailsEdit) + Send + Sync> {
    Arc::new(move |details_edit: DetailsEdit| {
        let project_symbol_catalog = SymbolLayoutEditorView::get_opened_project_symbol_catalog_from_context(&app_context).unwrap_or_default();
        let updated_variant_draft = {
            let Some(mut view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor apply variant field details edit") else {
                return;
            };
            let Some(union_draft) = view_data.get_draft().cloned() else {
                return;
            };
            let mut variant_draft = view_data
                .get_pending_variant_draft(&variant_layout_id)
                .cloned()
                .unwrap_or_else(|| {
                    SymbolLayoutVariantSession::create_union_variant_layout_draft_for_id(&project_symbol_catalog, &union_draft, &variant_layout_id)
                });
            let Some(field_draft) = variant_draft.field_drafts.get_mut(field_index) else {
                return;
            };

            apply_field_details_operation(&project_symbol_catalog, field_draft, SymbolLayoutDetails::plan_edit(&details_edit));
            grow_draft_size_to_fit_fields(&project_symbol_catalog, &mut variant_draft);
            view_data.replace_pending_variant_draft(variant_draft.clone());
            variant_draft
        };
        SymbolLayoutEditorViewData::select_field_for_layout(symbol_layout_editor_view_data.clone(), Some(variant_layout_id.clone()), field_index);

        let updated_project_symbol_catalog = SymbolLayoutVariantSession::build_effective_project_symbol_catalog_from_view_data(
            &project_symbol_catalog,
            symbol_layout_editor_view_data.clone(),
            Some(&variant_layout_id),
        );
        let details_projection = updated_variant_draft
            .field_drafts
            .get(field_index)
            .map(|field_draft| {
                SymbolLayoutDetails::build_field_projection(
                    &updated_variant_draft.layout_id,
                    field_index,
                    SymbolicLayoutKind::Struct,
                    &build_field_details(&updated_project_symbol_catalog, SymbolicLayoutKind::Struct, field_draft),
                )
            });
        let Some(details_projection) = details_projection else {
            return;
        };
        let selection_key = format!("field|{}|{}", variant_layout_id, field_index);
        let edit_callback = build_variant_field_edit_callback(
            symbol_layout_editor_view_data.clone(),
            struct_viewer_view_data.clone(),
            app_context.clone(),
            variant_layout_id.clone(),
            field_index,
        );

        focus_details_projection(&app_context, &struct_viewer_view_data, details_projection, edit_callback, selection_key);
    })
}

fn apply_field_details_operation(
    project_symbol_catalog: &ProjectSymbolCatalog,
    field_draft: &mut SymbolLayoutFieldEditDraft,
    edit_operation: SymbolLayoutDetailsEditOperation,
) {
    match edit_operation {
        SymbolLayoutDetailsEditOperation::UpdateFieldName(field_name) => {
            field_draft.field_name = field_name;
        }
        SymbolLayoutDetailsEditOperation::UpdateFieldElementKind(element_kind) => {
            apply_field_element_type_edit(project_symbol_catalog, field_draft, element_kind.label());
        }
        SymbolLayoutDetailsEditOperation::UpdateFieldDataType(data_type_id) | SymbolLayoutDetailsEditOperation::UpdateFieldSymbolLayout(data_type_id) => {
            field_draft
                .data_type_selection
                .replace_selected_data_types(vec![DataTypeRef::new(data_type_id.trim())]);
        }
        SymbolLayoutDetailsEditOperation::UpdateFieldContainerKind(container_kind_label) => {
            if let Some(container_kind) = container_kind_from_label(&container_kind_label) {
                field_draft.container_edit.kind = container_kind;
            }
        }
        SymbolLayoutDetailsEditOperation::UpdateFieldFixedArrayLength(length) => {
            field_draft.container_edit.fixed_array_length = length.max(1).to_string();
        }
        SymbolLayoutDetailsEditOperation::UpdateFieldCountResolver(count_resolver_id) => {
            field_draft.container_edit.dynamic_array_count_resolver_id = count_resolver_id;
        }
        SymbolLayoutDetailsEditOperation::UpdateFieldDisplayCountResolver(display_count_resolver_id) => {
            field_draft.container_edit.display_count_resolver_id = display_count_resolver_id;
        }
        SymbolLayoutDetailsEditOperation::UpdateFieldActiveWhenResolver(active_when_resolver_id) => {
            field_draft.active_when_resolver_id = active_when_resolver_id;
        }
        SymbolLayoutDetailsEditOperation::UpdateFieldPointerSize(pointer_size_label) => {
            if let Ok(pointer_size) = PointerScanPointerSize::from_str(pointer_size_label.trim()) {
                field_draft.container_edit.pointer_size = pointer_size;
            }
        }
        SymbolLayoutDetailsEditOperation::UpdateFieldOffsetResolver(offset_resolver_id) => {
            field_draft.offset_resolver_id = offset_resolver_id;
        }
        SymbolLayoutDetailsEditOperation::UpdateLayoutKind(_) | SymbolLayoutDetailsEditOperation::NoOp | SymbolLayoutDetailsEditOperation::Reject(_) => {}
    }
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn grow_draft_size_to_fit_fields(
    project_symbol_catalog: &ProjectSymbolCatalog,
    draft: &mut SymbolLayoutEditDraft,
) {
    let Ok(declared_size_in_bytes) = SymbolLayoutEditorViewData::parse_layout_size_text(&draft.size_text, draft.size_format) else {
        return;
    };
    let mut next_sequential_offset = 0_u64;

    for field_draft in &draft.field_drafts {
        let Ok(symbolic_field_definition) = SymbolLayoutEditorView::build_symbolic_field_definition_from_draft(field_draft) else {
            continue;
        };
        let field_offset = match symbolic_field_definition.get_offset_resolution() {
            SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) if draft.layout_kind.is_union() => 0,
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
        };
        let field_size_in_bytes = SymbolLayoutEditorViewData::resolve_symbolic_field_size_in_bytes(
            project_symbol_catalog,
            &symbolic_field_definition,
            &mut std::collections::HashSet::new(),
        );

        next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
    }

    if next_sequential_offset > declared_size_in_bytes {
        draft.size_text = format_layout_size(next_sequential_offset, draft.size_format);
    }
}

fn format_layout_size(
    size_in_bytes: u64,
    size_format: AnonymousValueStringFormat,
) -> String {
    match size_format {
        AnonymousValueStringFormat::Binary => format!("{:b}", size_in_bytes),
        AnonymousValueStringFormat::Hexadecimal | AnonymousValueStringFormat::Address => format!("{:X}", size_in_bytes),
        _ => size_in_bytes.to_string(),
    }
}

fn apply_field_element_type_edit(
    project_symbol_catalog: &ProjectSymbolCatalog,
    field_draft: &mut SymbolLayoutFieldEditDraft,
    edited_text: &str,
) {
    let current_element_type = SymbolLayoutEditorViewData::resolve_field_element_type(project_symbol_catalog, field_draft);
    let selected_element_type = SymbolLayoutFieldElementType::ALL
        .iter()
        .copied()
        .find(|element_type| element_type.label() == edited_text.trim())
        .unwrap_or(current_element_type);

    if selected_element_type == current_element_type {
        return;
    }

    let next_data_type_ref = match selected_element_type {
        SymbolLayoutFieldElementType::BuiltInDataType => Some(DataTypeRef::new(DataTypeI32::DATA_TYPE_ID)),
        SymbolLayoutFieldElementType::SymbolLayout => {
            SymbolLayoutEditorViewData::first_symbol_layout_id(project_symbol_catalog).map(|struct_layout_id| DataTypeRef::new(&struct_layout_id))
        }
    };

    if let Some(next_data_type_ref) = next_data_type_ref {
        field_draft
            .data_type_selection
            .replace_selected_data_types(vec![next_data_type_ref]);
    }
}

fn container_kind_from_label(label: &str) -> Option<SymbolLayoutFieldContainerKind> {
    SymbolLayoutFieldContainerKind::ALL
        .iter()
        .copied()
        .find(|container_kind| container_kind.label() == label)
}

fn focus_details_projection(
    app_context: &Arc<AppContext>,
    struct_viewer_view_data: &squalr_engine_api::dependency_injection::dependency::Dependency<StructViewerViewData>,
    details_projection: squalr_engine_api::structures::details::details_projection::DetailsProjection,
    edit_callback: Arc<dyn Fn(squalr_engine_api::structures::details::details_edit::DetailsEdit) + Send + Sync>,
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
