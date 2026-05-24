use crate::app_context::AppContext;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutEditDraft, SymbolLayoutFieldEditDraft, SymbolLayoutFieldOffsetMode,
};
use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::SymbolLayoutFieldContainerKind;
use squalr_engine_api::structures::{
    data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef},
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::symbolic_struct_definition::SymbolicLayoutKind,
};

/// Builds Symbol Layout field drafts from editor context and catalog defaults.
pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) struct SymbolLayoutFieldDraftFactory;

impl SymbolLayoutFieldDraftFactory {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn default_data_type_ref(app_context: &AppContext) -> DataTypeRef {
        let registered_data_types = app_context
            .engine_unprivileged_state
            .get_registered_data_type_refs();

        registered_data_types
            .iter()
            .find(|data_type_ref| data_type_ref.get_data_type_id() == DataTypeI32::DATA_TYPE_ID)
            .cloned()
            .or_else(|| registered_data_types.first().cloned())
            .unwrap_or_else(|| DataTypeRef::new(DataTypeI32::DATA_TYPE_ID))
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn create_field_draft_for_layout_kind(
        app_context: &AppContext,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_kind: SymbolicLayoutKind,
        owning_layout_id: &str,
        field_position: usize,
    ) -> SymbolLayoutFieldEditDraft {
        if layout_kind.is_union() {
            let struct_layout_id = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| {
                    struct_layout_descriptor.get_struct_layout_id() != owning_layout_id
                        && struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_layout_kind()
                            == SymbolicLayoutKind::Struct
                })
                .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id().to_string());

            let mut field_draft = SymbolLayoutFieldEditDraft::new(
                struct_layout_id
                    .as_deref()
                    .map(DataTypeRef::new)
                    .unwrap_or_else(|| Self::default_data_type_ref(app_context)),
            );
            field_draft.field_name = format!("Variant {}", field_position + 1);
            field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::Element;
            field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Sequential;

            return field_draft;
        }

        let mut field_draft = SymbolLayoutFieldEditDraft::new(Self::default_data_type_ref(app_context));
        field_draft.field_name = format!("field_{}", field_position + 1);
        field_draft
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn create_field_draft_for_unassigned_span(
        app_context: &AppContext,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_kind: SymbolicLayoutKind,
        owning_layout_id: &str,
        field_position: usize,
        offset_in_bytes: u64,
    ) -> SymbolLayoutFieldEditDraft {
        let mut field_draft = Self::create_field_draft_for_layout_kind(app_context, project_symbol_catalog, layout_kind, owning_layout_id, field_position);

        if !layout_kind.is_union() {
            field_draft.field_name = format!("field_{:X}", offset_in_bytes);
            field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
            field_draft.static_offset_in_bytes = String::from("0");
        }

        field_draft
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn normalize_union_field_drafts(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &mut SymbolLayoutEditDraft,
    ) {
        if !draft.layout_kind.is_union() {
            return;
        }

        for field_position in 0..draft.field_drafts.len() {
            let replacement_data_type_ref = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| {
                    struct_layout_descriptor.get_struct_layout_id() != draft.layout_id
                        && struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_layout_kind()
                            == SymbolicLayoutKind::Struct
                })
                .map(|struct_layout_descriptor| DataTypeRef::new(struct_layout_descriptor.get_struct_layout_id()));

            if let Some(field_draft) = draft.field_drafts.get_mut(field_position) {
                if let Some(replacement_data_type_ref) = replacement_data_type_ref.clone() {
                    field_draft
                        .data_type_selection
                        .replace_selected_data_types(vec![replacement_data_type_ref]);
                }
                field_draft.container_edit = Default::default();
                field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Sequential;
                field_draft.static_offset_in_bytes.clear();
                field_draft.offset_resolver_id.clear();
            }
        }
    }
}
