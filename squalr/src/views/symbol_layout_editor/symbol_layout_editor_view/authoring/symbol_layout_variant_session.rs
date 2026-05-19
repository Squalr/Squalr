use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{SymbolLayoutEditDraft, SymbolLayoutEditorViewData};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use squalr_engine_api::structures::{
    data_types::data_type_ref::DataTypeRef, projects::project_symbol_catalog::ProjectSymbolCatalog, structs::symbolic_struct_definition::SymbolicLayoutKind,
};
use std::collections::BTreeSet;

/// Owns the GUI edit session overlay for union variant layouts.
pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) struct SymbolLayoutVariantSession;

impl SymbolLayoutVariantSession {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn create_union_variant_layout_draft_with_pending(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        union_draft: &SymbolLayoutEditDraft,
        variant_index: usize,
        variant_field_draft: &crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::SymbolLayoutFieldEditDraft,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> SymbolLayoutEditDraft {
        let variant_layout_id = variant_field_draft
            .data_type_selection
            .visible_data_type()
            .get_data_type_id();
        if let Some(mut pending_variant_draft) = Self::read_pending_variant_layout_draft(symbol_layout_editor_view_data.clone(), variant_layout_id) {
            pending_variant_draft.size_text = union_draft.size_text.clone();
            pending_variant_draft.size_format = union_draft.size_format;

            return pending_variant_draft;
        }
        if let Some(variant_layout_descriptor) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == variant_layout_id)
        {
            return Self::create_union_variant_layout_draft_for_id_with_pending(
                project_symbol_catalog,
                symbol_layout_editor_view_data,
                union_draft,
                variant_layout_descriptor.get_struct_layout_id(),
                resolve_data_type_size_in_bytes,
            );
        }

        let variant_layout_id = Self::build_union_variant_layout_id(project_symbol_catalog, union_draft, variant_index);
        if let Some(mut pending_variant_draft) = Self::read_pending_variant_layout_draft(symbol_layout_editor_view_data, &variant_layout_id) {
            pending_variant_draft.size_text = union_draft.size_text.clone();
            pending_variant_draft.size_format = union_draft.size_format;

            return pending_variant_draft;
        }

        Self::create_virtual_union_variant_layout_draft(union_draft, variant_layout_id)
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn create_union_variant_layout_draft_for_id_with_pending(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        union_draft: &SymbolLayoutEditDraft,
        variant_layout_id: &str,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> SymbolLayoutEditDraft {
        if let Some(mut pending_variant_draft) = Self::read_pending_variant_layout_draft(symbol_layout_editor_view_data, variant_layout_id) {
            pending_variant_draft.size_text = union_draft.size_text.clone();
            pending_variant_draft.size_format = union_draft.size_format;

            return pending_variant_draft;
        }

        Self::create_union_variant_layout_draft_for_id(project_symbol_catalog, union_draft, variant_layout_id, resolve_data_type_size_in_bytes)
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn create_union_variant_layout_draft_for_id(
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &SymbolLayoutEditDraft,
        variant_layout_id: &str,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> SymbolLayoutEditDraft {
        if let Some(variant_layout_descriptor) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == variant_layout_id)
        {
            let mut variant_draft = SymbolLayoutEditorViewData::create_draft_from_descriptor(variant_layout_descriptor, resolve_data_type_size_in_bytes);

            variant_draft.size_text = union_draft.size_text.clone();
            variant_draft.size_format = union_draft.size_format;

            return variant_draft;
        }

        Self::create_virtual_union_variant_layout_draft(union_draft, variant_layout_id.to_string())
    }

    fn create_virtual_union_variant_layout_draft(
        union_draft: &SymbolLayoutEditDraft,
        variant_layout_id: String,
    ) -> SymbolLayoutEditDraft {
        SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: variant_layout_id,
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: union_draft.size_text.clone(),
            size_format: union_draft.size_format,
            field_drafts: Vec::new(),
        }
    }

    fn build_union_variant_layout_id(
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &SymbolLayoutEditDraft,
        variant_index: usize,
    ) -> String {
        let trimmed_union_layout_id = union_draft.layout_id.trim();
        let base_layout_id = if trimmed_union_layout_id.is_empty() {
            format!("union.variant_{}", variant_index + 1)
        } else {
            format!("{}.variant_{}", trimmed_union_layout_id, variant_index + 1)
        };
        if !project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == base_layout_id)
        {
            return base_layout_id;
        }

        let mut suffix_index = 2_u64;
        loop {
            let candidate_layout_id = format!("{}_{}", base_layout_id, suffix_index);
            if !project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == candidate_layout_id)
            {
                return candidate_layout_id;
            }

            suffix_index = suffix_index.saturating_add(1);
        }
    }

    fn read_pending_variant_layout_draft(
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        variant_layout_id: &str,
    ) -> Option<SymbolLayoutEditDraft> {
        symbol_layout_editor_view_data
            .read("SymbolLayoutEditor read pending variant draft")
            .and_then(|symbol_layout_editor_view_data| {
                symbol_layout_editor_view_data
                    .get_pending_variant_draft(variant_layout_id)
                    .cloned()
            })
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn cache_variant_layout_draft(
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        variant_draft: &SymbolLayoutEditDraft,
    ) -> bool {
        let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor cache pending variant draft") else {
            return false;
        };

        symbol_layout_editor_view_data.replace_pending_variant_draft(variant_draft.clone());
        true
    }

    fn pending_variant_drafts_for_union_from_view_data(
        symbol_layout_editor_view_data: &SymbolLayoutEditorViewData,
        union_draft: Option<&SymbolLayoutEditDraft>,
    ) -> Vec<(SymbolLayoutEditDraft, BTreeSet<u64>)> {
        let Some(union_draft) = union_draft.filter(|union_draft| union_draft.layout_kind.is_union()) else {
            return Vec::new();
        };
        let referenced_variant_layout_ids = union_draft
            .field_drafts
            .iter()
            .map(|field_draft| {
                field_draft
                    .data_type_selection
                    .visible_data_type()
                    .get_data_type_id()
                    .to_string()
            })
            .collect::<BTreeSet<_>>();

        symbol_layout_editor_view_data
            .get_pending_variant_drafts_with_split_offsets()
            .into_iter()
            .filter(|(variant_draft, _unassigned_split_offsets)| referenced_variant_layout_ids.contains(&variant_draft.layout_id))
            .collect()
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn pending_variant_drafts_for_union(
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        union_draft: Option<&SymbolLayoutEditDraft>,
    ) -> Vec<(SymbolLayoutEditDraft, BTreeSet<u64>)> {
        symbol_layout_editor_view_data
            .read("SymbolLayoutEditor read pending variant drafts")
            .map(|symbol_layout_editor_view_data| Self::pending_variant_drafts_for_union_from_view_data(&symbol_layout_editor_view_data, union_draft))
            .unwrap_or_default()
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn build_effective_project_symbol_catalog_from_pending_drafts(
        project_symbol_catalog: &ProjectSymbolCatalog,
        pending_variant_drafts: &[(SymbolLayoutEditDraft, BTreeSet<u64>)],
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> ProjectSymbolCatalog {
        let mut effective_project_symbol_catalog = project_symbol_catalog.clone();
        let mut struct_layout_descriptors = effective_project_symbol_catalog
            .get_struct_layout_descriptors()
            .to_vec();

        for (variant_draft, unassigned_split_offsets) in pending_variant_drafts {
            let Ok(variant_descriptor) = SymbolLayoutEditorViewData::build_symbol_layout_descriptor_with_unassigned_split_offsets(
                project_symbol_catalog,
                variant_draft,
                unassigned_split_offsets,
                resolve_data_type_size_in_bytes,
            ) else {
                continue;
            };
            let variant_layout_id = variant_descriptor.get_struct_layout_id().to_string();

            struct_layout_descriptors.retain(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() != variant_layout_id);
            struct_layout_descriptors.push(variant_descriptor);
        }

        effective_project_symbol_catalog.set_struct_layout_descriptors(struct_layout_descriptors);
        effective_project_symbol_catalog
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn build_effective_project_symbol_catalog_from_view_data(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        focused_variant_layout_id: Option<&str>,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> ProjectSymbolCatalog {
        let pending_variant_drafts = symbol_layout_editor_view_data
            .read("SymbolLayoutEditor build effective catalog")
            .map(|symbol_layout_editor_view_data| {
                let union_draft = symbol_layout_editor_view_data.get_draft();
                let mut pending_variant_drafts = Self::pending_variant_drafts_for_union_from_view_data(&symbol_layout_editor_view_data, union_draft);

                if let Some(focused_variant_layout_id) = focused_variant_layout_id
                    && !pending_variant_drafts
                        .iter()
                        .any(|(variant_draft, _unassigned_split_offsets)| variant_draft.layout_id == focused_variant_layout_id)
                    && let Some(focused_variant_draft) = symbol_layout_editor_view_data.get_pending_variant_draft(focused_variant_layout_id)
                {
                    pending_variant_drafts.push((
                        focused_variant_draft.clone(),
                        symbol_layout_editor_view_data.get_unassigned_split_offsets_for_layout(Some(focused_variant_layout_id)),
                    ));
                }

                pending_variant_drafts
            })
            .unwrap_or_default();

        Self::build_effective_project_symbol_catalog_from_pending_drafts(project_symbol_catalog, &pending_variant_drafts, resolve_data_type_size_in_bytes)
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn build_pending_variant_layout_descriptors(
        project_symbol_catalog: &ProjectSymbolCatalog,
        pending_variant_drafts: &[(SymbolLayoutEditDraft, BTreeSet<u64>)],
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> Result<Vec<(Option<String>, StructLayoutDescriptor)>, String> {
        pending_variant_drafts
            .iter()
            .map(|(variant_draft, unassigned_split_offsets)| {
                SymbolLayoutEditorViewData::build_symbol_layout_descriptor_with_unassigned_split_offsets(
                    project_symbol_catalog,
                    variant_draft,
                    unassigned_split_offsets,
                    resolve_data_type_size_in_bytes,
                )
                .map(|struct_layout_descriptor| (variant_draft.original_layout_id.clone(), struct_layout_descriptor))
                .map_err(|error| format!("Variant layout `{}`: {}", variant_draft.layout_id, error))
            })
            .collect()
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn persist_variant_layout_draft(
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        variant_draft: &SymbolLayoutEditDraft,
    ) -> bool {
        Self::cache_variant_layout_draft(symbol_layout_editor_view_data, variant_draft)
    }
}
