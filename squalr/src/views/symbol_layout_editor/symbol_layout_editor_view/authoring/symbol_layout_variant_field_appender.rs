use super::symbol_layout_draft_analyzer::SymbolLayoutDraftAnalyzer;
use super::symbol_layout_field_draft_factory::SymbolLayoutFieldDraftFactory;
use super::symbol_layout_variant_session::SymbolLayoutVariantSession;
use crate::app_context::AppContext;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutEditDraft, SymbolLayoutEditorViewData, SymbolLayoutFieldOffsetMode,
};
use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::SymbolLayoutFieldContainerKind;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::{
    data_types::data_type_ref::DataTypeRef,
    projects::{project_symbol_catalog::ProjectSymbolCatalog, symbol_layouts::symbol_layout_draft_ops::SymbolLayoutDraftOps},
    structs::symbolic_struct_definition::SymbolicLayoutKind,
};

/// Appends fields to the pending struct layout backing a union variant row.
pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) struct SymbolLayoutVariantFieldAppender;

impl SymbolLayoutVariantFieldAppender {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn append_field_to_variant_layout(
        app_context: &AppContext,
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &mut SymbolLayoutEditDraft,
        variant_index: usize,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> Option<(SymbolLayoutEditDraft, usize)> {
        let Some(variant_field_draft) = union_draft.field_drafts.get(variant_index) else {
            return None;
        };
        let mut variant_draft = SymbolLayoutVariantSession::create_union_variant_layout_draft_with_pending(
            project_symbol_catalog,
            symbol_layout_editor_view_data.clone(),
            union_draft,
            variant_index,
            variant_field_draft,
            resolve_data_type_size_in_bytes,
        );

        let Some((layout_size_in_bytes, field_spans)) =
            SymbolLayoutDraftAnalyzer::resolve_draft_field_spans(project_symbol_catalog, &variant_draft, resolve_data_type_size_in_bytes)
        else {
            return None;
        };
        let Some(field_offset_in_bytes) = SymbolLayoutDraftOps::resolve_tail_unassigned_offset(&field_spans, layout_size_in_bytes) else {
            return None;
        };

        let field_position = variant_draft.field_drafts.len();
        let mut field_draft = SymbolLayoutFieldDraftFactory::create_field_draft_for_layout_kind(
            app_context,
            project_symbol_catalog,
            SymbolicLayoutKind::Struct,
            &variant_draft.layout_id,
            field_position,
        );
        field_draft.field_name = format!("field_{:08X}", field_offset_in_bytes);
        field_draft.field_name = SymbolLayoutDraftOps::build_unique_field_name(&variant_draft, &field_draft.field_name);
        field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
        field_draft.static_offset_in_bytes = field_offset_in_bytes.to_string();
        variant_draft.field_drafts.push(field_draft);
        if let Some(variant_field_draft) = union_draft.field_drafts.get_mut(variant_index) {
            variant_field_draft
                .data_type_selection
                .select_single_data_type(DataTypeRef::new(&variant_draft.layout_id));
            variant_field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::Element;
            variant_field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Sequential;
        }

        SymbolLayoutVariantSession::cache_variant_layout_draft(symbol_layout_editor_view_data, &variant_draft).then_some((variant_draft, field_position))
    }
}
