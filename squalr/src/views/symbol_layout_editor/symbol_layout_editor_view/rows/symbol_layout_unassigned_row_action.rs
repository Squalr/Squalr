use super::super::SymbolLayoutEditorView;
use super::super::authoring::symbol_layout_field_draft_factory::SymbolLayoutFieldDraftFactory;
use super::super::authoring::symbol_layout_variant_session::SymbolLayoutVariantSession;
use super::super::details::symbol_layout_details_focus::focus_unassigned_span_in_struct_viewer;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{SymbolLayoutEditDraft, SymbolLayoutEditorViewData};
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog,
    symbol_layouts::symbol_layout_draft_ops::{SymbolLayoutDraftOps, SymbolLayoutUnassignedRowContext},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) enum SymbolLayoutUnassignedRowAction {
    SelectSpan,
    DefineField,
    MoveUp,
    MoveDown,
    SplitRange,
    MergeAbove,
    MergeBelow,
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn apply_unassigned_row_action(
    symbol_layout_editor_view: &SymbolLayoutEditorView,
    project_symbol_catalog: &ProjectSymbolCatalog,
    draft: &mut SymbolLayoutEditDraft,
    target_layout_id: Option<String>,
    unassigned_row_context: SymbolLayoutUnassignedRowContext,
    unassigned_row_action: SymbolLayoutUnassignedRowAction,
) {
    let mut target_variant_draft = target_layout_id.as_deref().map(|target_layout_id| {
        SymbolLayoutVariantSession::create_union_variant_layout_draft_for_id_with_pending(
            project_symbol_catalog,
            symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
            draft,
            target_layout_id,
            |data_type_ref| symbol_layout_editor_view.resolve_data_type_size_in_bytes(data_type_ref),
        )
    });
    let mut persist_target_variant_draft = false;
    let focus_unassigned_span = |focus_draft: &SymbolLayoutEditDraft, offset_in_bytes: u64, size_in_bytes: u64| {
        focus_unassigned_span_in_struct_viewer(
            symbol_layout_editor_view.app_context.clone(),
            symbol_layout_editor_view.struct_viewer_view_data.clone(),
            focus_draft,
            offset_in_bytes,
            size_in_bytes,
        );
    };

    match unassigned_row_action {
        SymbolLayoutUnassignedRowAction::SelectSpan => {
            SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                target_layout_id.clone(),
                unassigned_row_context.offset_in_bytes,
                unassigned_row_context.size_in_bytes,
            );
            let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
            focus_unassigned_span(focus_draft, unassigned_row_context.offset_in_bytes, unassigned_row_context.size_in_bytes);
        }
        SymbolLayoutUnassignedRowAction::DefineField => {
            if target_layout_id.is_some() {
                log::warn!("Ignoring Define Field action for nested union variant unassigned span.");
                return;
            }
            let mut field_draft = SymbolLayoutFieldDraftFactory::create_field_draft_for_unassigned_span(
                &symbol_layout_editor_view.app_context,
                project_symbol_catalog,
                draft.layout_kind,
                &draft.layout_id,
                0,
                unassigned_row_context.offset_in_bytes,
            );
            field_draft.field_name = format!("field_{:08X}", unassigned_row_context.offset_in_bytes);
            field_draft.field_name = SymbolLayoutDraftOps::build_unique_field_name(draft, &field_draft.field_name);
            SymbolLayoutEditorViewData::begin_define_field_from_unassigned_span(
                symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                draft.layout_id.clone(),
                unassigned_row_context.offset_in_bytes,
                unassigned_row_context.size_in_bytes,
                SymbolLayoutFieldDraftFactory::default_data_type_ref(&symbol_layout_editor_view.app_context),
            );
            SymbolLayoutEditorViewData::replace_define_field_draft(symbol_layout_editor_view.symbol_layout_editor_view_data.clone(), field_draft);
        }
        SymbolLayoutUnassignedRowAction::MoveUp => {
            let updated_unassigned_selection = if let Some(target_variant_draft) = target_variant_draft.as_mut() {
                SymbolLayoutDraftOps::move_unassigned_span_up(target_variant_draft, unassigned_row_context.clone())
            } else {
                SymbolLayoutDraftOps::move_unassigned_span_up(draft, unassigned_row_context.clone())
            };

            if let Some(updated_unassigned_selection) = updated_unassigned_selection {
                persist_target_variant_draft = target_layout_id.is_some();
                if let Some(split_offset_in_bytes) = SymbolLayoutDraftOps::split_offset_to_preserve_unassigned_move_up(&updated_unassigned_selection) {
                    SymbolLayoutEditorViewData::insert_unassigned_split_offset_for_layout(
                        symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                        target_layout_id.clone(),
                        split_offset_in_bytes,
                    );
                }
                SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                    symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                    target_layout_id.clone(),
                    updated_unassigned_selection.get_offset_in_bytes(),
                    updated_unassigned_selection.get_size_in_bytes(),
                );
                let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                focus_unassigned_span(
                    focus_draft,
                    updated_unassigned_selection.get_offset_in_bytes(),
                    updated_unassigned_selection.get_size_in_bytes(),
                );
            } else if let Some(move_up_unassigned_span) = unassigned_row_context.move_up_unassigned_span.as_ref() {
                let old_split_offset = unassigned_row_context.offset_in_bytes;
                let new_split_offset = move_up_unassigned_span
                    .get_offset_in_bytes()
                    .saturating_add(unassigned_row_context.size_in_bytes);
                SymbolLayoutEditorViewData::move_unassigned_split_offset_for_layout(
                    symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                    target_layout_id.clone(),
                    old_split_offset,
                    new_split_offset,
                );
                SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                    symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                    target_layout_id.clone(),
                    move_up_unassigned_span.get_offset_in_bytes(),
                    unassigned_row_context.size_in_bytes,
                );
                let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                focus_unassigned_span(focus_draft, move_up_unassigned_span.get_offset_in_bytes(), unassigned_row_context.size_in_bytes);
            }
        }
        SymbolLayoutUnassignedRowAction::MoveDown => {
            let updated_unassigned_selection = if let Some(target_variant_draft) = target_variant_draft.as_mut() {
                SymbolLayoutDraftOps::move_unassigned_span_down(target_variant_draft, unassigned_row_context.clone())
            } else {
                SymbolLayoutDraftOps::move_unassigned_span_down(draft, unassigned_row_context.clone())
            };

            if let Some(updated_unassigned_selection) = updated_unassigned_selection {
                persist_target_variant_draft = target_layout_id.is_some();
                if let Some(split_offset_in_bytes) = SymbolLayoutDraftOps::split_offset_to_preserve_unassigned_move_down(&updated_unassigned_selection) {
                    SymbolLayoutEditorViewData::insert_unassigned_split_offset_for_layout(
                        symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                        target_layout_id.clone(),
                        split_offset_in_bytes,
                    );
                }
                SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                    symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                    target_layout_id.clone(),
                    updated_unassigned_selection.get_offset_in_bytes(),
                    updated_unassigned_selection.get_size_in_bytes(),
                );
                let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                focus_unassigned_span(
                    focus_draft,
                    updated_unassigned_selection.get_offset_in_bytes(),
                    updated_unassigned_selection.get_size_in_bytes(),
                );
            } else if let Some(move_down_unassigned_span) = unassigned_row_context.move_down_unassigned_span.as_ref() {
                let old_split_offset = unassigned_row_context
                    .offset_in_bytes
                    .saturating_add(unassigned_row_context.size_in_bytes);
                let new_unassigned_offset = unassigned_row_context
                    .offset_in_bytes
                    .saturating_add(move_down_unassigned_span.get_size_in_bytes());
                SymbolLayoutEditorViewData::move_unassigned_split_offset_for_layout(
                    symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                    target_layout_id.clone(),
                    old_split_offset,
                    new_unassigned_offset,
                );
                SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                    symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                    target_layout_id.clone(),
                    new_unassigned_offset,
                    unassigned_row_context.size_in_bytes,
                );
                let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                focus_unassigned_span(focus_draft, new_unassigned_offset, unassigned_row_context.size_in_bytes);
            }
        }
        SymbolLayoutUnassignedRowAction::SplitRange => {
            if let Some(updated_unassigned_selection) = SymbolLayoutEditorViewData::split_unassigned_span_for_layout(
                symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                target_layout_id.clone(),
                unassigned_row_context.offset_in_bytes,
                unassigned_row_context.size_in_bytes,
            ) {
                let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                focus_unassigned_span(
                    focus_draft,
                    updated_unassigned_selection.get_offset_in_bytes(),
                    updated_unassigned_selection.get_size_in_bytes(),
                );
            }
        }
        SymbolLayoutUnassignedRowAction::MergeAbove => {
            if let Some(merge_above_span) = unassigned_row_context.merge_above_span.as_ref() {
                SymbolLayoutEditorViewData::remove_unassigned_split_offset_for_layout(
                    symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                    target_layout_id.clone(),
                    unassigned_row_context.offset_in_bytes,
                );
                SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                    symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                    target_layout_id.clone(),
                    merge_above_span.get_offset_in_bytes(),
                    merge_above_span.get_size_in_bytes(),
                );
                let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                focus_unassigned_span(focus_draft, merge_above_span.get_offset_in_bytes(), merge_above_span.get_size_in_bytes());
            }
        }
        SymbolLayoutUnassignedRowAction::MergeBelow => {
            if let Some(merge_below_span) = unassigned_row_context.merge_below_span.as_ref() {
                SymbolLayoutEditorViewData::remove_unassigned_split_offset_for_layout(
                    symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                    target_layout_id.clone(),
                    unassigned_row_context
                        .offset_in_bytes
                        .saturating_add(unassigned_row_context.size_in_bytes),
                );
                SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                    symbol_layout_editor_view.symbol_layout_editor_view_data.clone(),
                    target_layout_id.clone(),
                    merge_below_span.get_offset_in_bytes(),
                    merge_below_span.get_size_in_bytes(),
                );
                let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                focus_unassigned_span(focus_draft, merge_below_span.get_offset_in_bytes(), merge_below_span.get_size_in_bytes());
            }
        }
    }

    if persist_target_variant_draft && let Some(target_variant_draft) = target_variant_draft.as_ref() {
        SymbolLayoutVariantSession::persist_variant_layout_draft(symbol_layout_editor_view.symbol_layout_editor_view_data.clone(), target_variant_draft);
    }
}
