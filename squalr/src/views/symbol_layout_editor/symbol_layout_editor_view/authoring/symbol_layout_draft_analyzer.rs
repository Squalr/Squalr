use super::symbol_layout_variant_session::SymbolLayoutVariantSession;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutEditDraft, SymbolLayoutEditorViewData, SymbolLayoutFieldEditDraft,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::{
    data_types::data_type_ref::DataTypeRef,
    projects::{
        project_symbol_catalog::ProjectSymbolCatalog,
        symbol_layouts::symbol_layout_draft_ops::{SymbolLayoutDraftOps, SymbolLayoutFieldSpan},
    },
    structs::symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
};

/// Resolves editable Symbol Layout drafts into spans and validation decisions.
pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) struct SymbolLayoutDraftAnalyzer;

impl SymbolLayoutDraftAnalyzer {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn build_symbolic_field_definition_from_draft(
        field_draft: &SymbolLayoutFieldEditDraft
    ) -> Result<SymbolicFieldDefinition, String> {
        let trimmed_data_type_id = field_draft
            .data_type_selection
            .visible_data_type()
            .get_data_type_id()
            .trim()
            .to_string();
        if trimmed_data_type_id.is_empty() {
            return Err(String::from("Field data type is required."));
        }

        Ok(SymbolicFieldDefinition::new_named_with_resolutions_and_display_count(
            field_draft.field_name.trim().to_string(),
            DataTypeRef::new(&trimmed_data_type_id),
            field_draft.container_edit.to_container_type()?,
            field_draft.container_edit.to_count_resolution()?,
            field_draft.container_edit.to_display_count_resolution()?,
            field_draft.to_offset_resolution()?,
        )
        .with_active_when_resolver(field_draft.to_active_when_resolver()))
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn validate_define_field_draft(
        project_symbol_catalog: &ProjectSymbolCatalog,
        field_draft: &SymbolLayoutFieldEditDraft,
        span_offset_in_bytes: u64,
        span_size_in_bytes: u64,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> Result<(u64, u64), String> {
        if field_draft.field_name.trim().is_empty() {
            return Err(String::from("Field name is required."));
        }

        if field_draft.offset_mode != crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::SymbolLayoutFieldOffsetMode::Static {
            return Err(String::from("Field offset must be static."));
        }

        let symbolic_field_definition = Self::build_symbolic_field_definition_from_draft(field_draft)?;
        let relative_offset_in_bytes = match symbolic_field_definition.get_offset_resolution() {
            SymbolicFieldOffsetResolution::Static(relative_offset_in_bytes) => *relative_offset_in_bytes,
            _ => return Err(String::from("Field offset must be static.")),
        };
        let field_size_in_bytes = SymbolLayoutEditorViewData::resolve_symbolic_field_size_in_bytes(
            project_symbol_catalog,
            &symbolic_field_definition,
            &mut std::collections::HashSet::new(),
            resolve_data_type_size_in_bytes,
        );

        if field_size_in_bytes == 0 {
            return Err(String::from("Field has no byte size."));
        }

        let relative_field_end_in_bytes = relative_offset_in_bytes
            .checked_add(field_size_in_bytes)
            .ok_or_else(|| String::from("Field range is too large."))?;

        if relative_field_end_in_bytes > span_size_in_bytes {
            if field_size_in_bytes > span_size_in_bytes {
                return Err(format!(
                    "`{}` uses {} byte(s); selected span has {}.",
                    symbolic_field_definition.get_data_type_ref().get_data_type_id(),
                    field_size_in_bytes,
                    span_size_in_bytes
                ));
            }

            return Err(format!(
                "`{}` uses {} byte(s); choose 0 to {}.",
                symbolic_field_definition.get_data_type_ref().get_data_type_id(),
                field_size_in_bytes,
                span_size_in_bytes.saturating_sub(field_size_in_bytes)
            ));
        }

        let field_offset_in_bytes = span_offset_in_bytes
            .checked_add(relative_offset_in_bytes)
            .ok_or_else(|| String::from("Field offset is too large."))?;

        Ok((field_offset_in_bytes, field_size_in_bytes))
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn resolve_variant_tail_unassigned_offset(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        union_draft: &SymbolLayoutEditDraft,
        variant_index: usize,
        variant_field_draft: &SymbolLayoutFieldEditDraft,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> Option<u64> {
        let variant_draft = SymbolLayoutVariantSession::create_union_variant_layout_draft_with_pending(
            project_symbol_catalog,
            symbol_layout_editor_view_data,
            union_draft,
            variant_index,
            variant_field_draft,
            resolve_data_type_size_in_bytes,
        );
        Self::resolve_draft_tail_unassigned_offset(project_symbol_catalog, &variant_draft, resolve_data_type_size_in_bytes)
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn resolve_draft_tail_unassigned_offset(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolLayoutEditDraft,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> Option<u64> {
        let (layout_size_in_bytes, field_spans) = Self::resolve_draft_field_spans(project_symbol_catalog, draft, resolve_data_type_size_in_bytes)?;

        SymbolLayoutDraftOps::resolve_tail_unassigned_offset(&field_spans, layout_size_in_bytes)
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn resolve_draft_field_spans(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolLayoutEditDraft,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> Option<(u64, Vec<SymbolLayoutFieldSpan>)> {
        let struct_layout_descriptor =
            SymbolLayoutEditorViewData::build_symbol_layout_descriptor(project_symbol_catalog, draft, resolve_data_type_size_in_bytes).ok()?;
        let symbolic_struct_definition = struct_layout_descriptor.get_struct_layout_definition();
        let layout_size_in_bytes = symbolic_struct_definition
            .get_declared_size_in_bytes()
            .unwrap_or_else(|| {
                SymbolLayoutEditorViewData::resolve_symbolic_struct_size_in_bytes(
                    project_symbol_catalog,
                    symbolic_struct_definition,
                    &mut std::collections::HashSet::new(),
                    resolve_data_type_size_in_bytes,
                )
            });
        let mut next_sequential_offset = 0_u64;
        let mut field_spans = Vec::with_capacity(symbolic_struct_definition.get_fields().len());
        let mut visible_field_position = 0_usize;

        for symbolic_field_definition in symbolic_struct_definition.get_fields() {
            if symbolic_field_definition.is_unassigned() {
                next_sequential_offset = next_sequential_offset.saturating_add(
                    symbolic_field_definition
                        .get_unassigned_size_in_bytes()
                        .unwrap_or(0),
                );
                continue;
            }

            let field_offset = match symbolic_field_definition.get_offset_resolution() {
                SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_)
                    if symbolic_struct_definition.get_layout_kind().is_union() =>
                {
                    0
                }
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
            };
            let field_size_in_bytes = SymbolLayoutEditorViewData::resolve_symbolic_field_size_in_bytes(
                project_symbol_catalog,
                symbolic_field_definition,
                &mut std::collections::HashSet::new(),
                resolve_data_type_size_in_bytes,
            );

            next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
            field_spans.push(SymbolLayoutFieldSpan {
                field_position: visible_field_position,
                offset_in_bytes: field_offset,
                size_in_bytes: field_size_in_bytes,
            });
            visible_field_position = visible_field_position.saturating_add(1);
        }

        Some((layout_size_in_bytes, field_spans))
    }
}
