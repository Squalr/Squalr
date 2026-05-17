use crate::app_context::AppContext;
use crate::views::struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData};
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{SymbolLayoutEditDraft, SymbolLayoutEditorViewData};
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog,
    symbol_layouts::{
        symbol_layout_details::SymbolLayoutDetails,
        symbol_layout_draft_ops::{SymbolLayoutDraftOps, SymbolLayoutFieldSpan},
    },
};
use squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicLayoutKind;
use std::{collections::BTreeSet, sync::Arc};

use super::super::SymbolLayoutEditorView;

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
        let mut variant_draft =
            symbol_layout_editor_view.create_union_variant_layout_draft_for_id_with_pending(project_symbol_catalog, union_draft, &variant_layout_id);
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
                let mut field_draft = symbol_layout_editor_view.create_field_draft_for_layout_kind(
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
            symbol_layout_editor_view.persist_variant_layout_draft(&variant_draft);
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
                let mut field_draft =
                    symbol_layout_editor_view.create_field_draft_for_layout_kind(project_symbol_catalog, draft.layout_kind, &draft.layout_id, insert_index);
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
    if !symbol_layout_editor_view.persist_variant_layout_draft(variant_draft) {
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
        symbol_layout_editor_view.focus_unassigned_span_in_struct_viewer(variant_draft, 0, layout_size_in_bytes);
    } else {
        SymbolLayoutEditorViewData::clear_field_selection(symbol_layout_editor_view.symbol_layout_editor_view_data.clone());
        symbol_layout_editor_view.clear_struct_viewer_if_symbol_layout_focused();
    }
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn focus_field_in_struct_viewer(
    symbol_layout_editor_view: &SymbolLayoutEditorView,
    project_symbol_catalog: &ProjectSymbolCatalog,
    draft: &SymbolLayoutEditDraft,
    field_index: usize,
) {
    let Some(field_draft) = draft.field_drafts.get(field_index) else {
        symbol_layout_editor_view.clear_struct_viewer_if_symbol_layout_focused();
        return;
    };

    let details_projection = SymbolLayoutDetails::build_field_projection(
        &draft.layout_id,
        field_index,
        draft.layout_kind,
        &SymbolLayoutEditorView::build_field_details(project_symbol_catalog, draft.layout_kind, field_draft),
    );
    let selection_key = format!("field|{}|{}", draft.layout_id, field_index);
    let edit_callback = SymbolLayoutEditorView::build_struct_viewer_field_edit_callback(
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
        symbol_layout_editor_view.clear_struct_viewer_if_symbol_layout_focused();
        return;
    };

    let details_projection = SymbolLayoutDetails::build_field_projection(
        &variant_draft.layout_id,
        field_index,
        SymbolicLayoutKind::Struct,
        &SymbolLayoutEditorView::build_field_details(project_symbol_catalog, SymbolicLayoutKind::Struct, field_draft),
    );
    let selection_key = format!("field|{}|{}", variant_draft.layout_id, field_index);
    let edit_callback = SymbolLayoutEditorView::build_variant_field_edit_callback(
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
