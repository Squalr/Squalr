use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use crate::views::struct_editor::view_data::struct_field_container_edit::StructFieldContainerEdit;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use squalr_engine_api::structures::{
    data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef},
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructFieldEditDraft {
    pub field_name: String,
    pub data_type_selection: DataTypeSelection,
    pub container_edit: StructFieldContainerEdit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructLayoutEditDraft {
    pub original_layout_id: Option<String>,
    pub layout_id: String,
    pub field_drafts: Vec<StructFieldEditDraft>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructEditorTakeOverState {
    CreateStructLayout,
    EditStructLayout { layout_id: String },
    DeleteConfirmation { layout_id: String },
}

#[derive(Clone, Default)]
pub struct StructEditorViewData {
    selected_layout_id: Option<String>,
    filter_text: String,
    take_over_state: Option<StructEditorTakeOverState>,
    baseline_draft: Option<StructLayoutEditDraft>,
    draft: Option<StructLayoutEditDraft>,
}

impl StructEditorViewData {
    pub fn new() -> Self {
        Self {
            selected_layout_id: None,
            filter_text: String::new(),
            take_over_state: None,
            baseline_draft: None,
            draft: None,
        }
    }

    pub fn get_selected_layout_id(&self) -> Option<&str> {
        self.selected_layout_id.as_deref()
    }

    pub fn get_filter_text(&self) -> &str {
        &self.filter_text
    }

    pub fn get_take_over_state(&self) -> Option<&StructEditorTakeOverState> {
        self.take_over_state.as_ref()
    }

    pub fn get_draft(&self) -> Option<&StructLayoutEditDraft> {
        self.draft.as_ref()
    }

    pub fn get_baseline_draft(&self) -> Option<&StructLayoutEditDraft> {
        self.baseline_draft.as_ref()
    }

    pub fn set_filter_text(
        struct_editor_view_data: Dependency<Self>,
        filter_text: String,
    ) {
        if let Some(mut struct_editor_view_data) = struct_editor_view_data.write("Symbol structs set filter text") {
            struct_editor_view_data.filter_text = filter_text;
        }
    }

    pub fn update_draft(
        struct_editor_view_data: Dependency<Self>,
        draft: StructLayoutEditDraft,
    ) {
        if let Some(mut struct_editor_view_data) = struct_editor_view_data.write("Symbol structs update draft") {
            struct_editor_view_data.draft = Some(draft);
        }
    }

    pub fn select_struct_layout(
        struct_editor_view_data: Dependency<Self>,
        selected_layout_id: Option<String>,
    ) {
        if let Some(mut struct_editor_view_data) = struct_editor_view_data.write("Symbol structs select struct layout") {
            struct_editor_view_data.selected_layout_id = selected_layout_id;
        }
    }

    pub fn begin_create_struct_layout(
        struct_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        default_data_type_ref: DataTypeRef,
    ) {
        if let Some(mut struct_editor_view_data) = struct_editor_view_data.write("Symbol structs begin create struct layout") {
            struct_editor_view_data.selected_layout_id = None;
            struct_editor_view_data.take_over_state = Some(StructEditorTakeOverState::CreateStructLayout);
            let baseline_draft = Self::create_default_new_draft(project_symbol_catalog, default_data_type_ref);
            struct_editor_view_data.baseline_draft = Some(baseline_draft.clone());
            struct_editor_view_data.draft = Some(baseline_draft);
        }
    }

    pub fn begin_edit_struct_layout(
        struct_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
    ) {
        if let Some(mut struct_editor_view_data) = struct_editor_view_data.write("Symbol structs begin edit struct layout") {
            struct_editor_view_data.selected_layout_id = Some(layout_id.to_string());
            struct_editor_view_data.take_over_state = Some(StructEditorTakeOverState::EditStructLayout {
                layout_id: layout_id.to_string(),
            });
            struct_editor_view_data.baseline_draft = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id)
                .map(Self::create_draft_from_descriptor);
            struct_editor_view_data.draft = struct_editor_view_data.baseline_draft.clone();
        }
    }

    pub fn request_delete_confirmation(
        struct_editor_view_data: Dependency<Self>,
        layout_id: String,
    ) {
        if let Some(mut struct_editor_view_data) = struct_editor_view_data.write("Symbol structs request delete confirmation") {
            struct_editor_view_data.take_over_state = Some(StructEditorTakeOverState::DeleteConfirmation { layout_id });
        }
    }

    pub fn cancel_take_over_state(struct_editor_view_data: Dependency<Self>) {
        if let Some(mut struct_editor_view_data) = struct_editor_view_data.write("Symbol structs cancel take over state") {
            struct_editor_view_data.take_over_state = None;
            struct_editor_view_data.baseline_draft = None;
            struct_editor_view_data.draft = None;
        }
    }

    pub fn synchronize(
        struct_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let Some(mut struct_editor_view_data) = struct_editor_view_data.write("Symbol structs synchronize") else {
            return;
        };

        let next_selected_layout_id = struct_editor_view_data
            .selected_layout_id
            .as_ref()
            .filter(|selected_layout_id| {
                project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == selected_layout_id.as_str())
            })
            .cloned()
            .or_else(|| {
                project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .first()
                    .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id().to_string())
            });

        struct_editor_view_data.selected_layout_id = next_selected_layout_id.clone();

        let should_clear_take_over_state = match struct_editor_view_data.take_over_state.as_ref() {
            Some(StructEditorTakeOverState::CreateStructLayout) => false,
            Some(StructEditorTakeOverState::EditStructLayout { layout_id }) | Some(StructEditorTakeOverState::DeleteConfirmation { layout_id }) => {
                !project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id)
            }
            None => false,
        };

        if should_clear_take_over_state {
            struct_editor_view_data.take_over_state = None;
            struct_editor_view_data.baseline_draft = None;
            struct_editor_view_data.draft = None;
        }
    }

    pub fn layout_matches_filter(
        struct_layout_descriptor: &StructLayoutDescriptor,
        filter_text: &str,
    ) -> bool {
        let trimmed_filter_text = filter_text.trim();
        if trimmed_filter_text.is_empty() {
            return true;
        }

        let normalized_filter_text = trimmed_filter_text.to_ascii_lowercase();
        if struct_layout_descriptor
            .get_struct_layout_id()
            .to_ascii_lowercase()
            .contains(&normalized_filter_text)
        {
            return true;
        }

        struct_layout_descriptor
            .get_struct_layout_definition()
            .get_fields()
            .iter()
            .any(|symbolic_field_definition| {
                symbolic_field_definition
                    .to_string()
                    .to_ascii_lowercase()
                    .contains(&normalized_filter_text)
            })
    }

    pub fn count_symbol_claim_usages(
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_id: &str,
    ) -> usize {
        project_symbol_catalog
            .get_symbol_claims()
            .iter()
            .filter(|symbol_claim| symbol_claim.get_struct_layout_id() == struct_layout_id)
            .count()
    }

    pub fn create_draft_from_descriptor(struct_layout_descriptor: &StructLayoutDescriptor) -> StructLayoutEditDraft {
        StructLayoutEditDraft {
            original_layout_id: Some(struct_layout_descriptor.get_struct_layout_id().to_string()),
            layout_id: struct_layout_descriptor.get_struct_layout_id().to_string(),
            field_drafts: struct_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .iter()
                .map(|symbolic_field_definition| StructFieldEditDraft {
                    field_name: symbolic_field_definition.get_field_name().to_string(),
                    data_type_selection: DataTypeSelection::new(symbolic_field_definition.get_data_type_ref().clone()),
                    container_edit: StructFieldContainerEdit::from_container_type(symbolic_field_definition.get_container_type()),
                })
                .collect(),
        }
    }

    pub fn create_default_new_draft(
        project_symbol_catalog: &ProjectSymbolCatalog,
        default_data_type_ref: DataTypeRef,
    ) -> StructLayoutEditDraft {
        let mut suffix_index = 1_u64;
        let mut proposed_layout_id = String::from("new.struct");
        while project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == proposed_layout_id)
        {
            suffix_index = suffix_index.saturating_add(1);
            proposed_layout_id = format!("new.struct{}", suffix_index);
        }

        StructLayoutEditDraft {
            original_layout_id: None,
            layout_id: proposed_layout_id,
            field_drafts: vec![StructFieldEditDraft {
                field_name: String::new(),
                data_type_selection: DataTypeSelection::new(default_data_type_ref),
                container_edit: StructFieldContainerEdit::default(),
            }],
        }
    }

    pub fn build_struct_layout_descriptor(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &StructLayoutEditDraft,
    ) -> Result<StructLayoutDescriptor, String> {
        let trimmed_layout_id = draft.layout_id.trim();
        if trimmed_layout_id.is_empty() {
            return Err(String::from("Struct layout id is required."));
        }

        let conflicts_with_existing_layout = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .any(|struct_layout_descriptor| {
                struct_layout_descriptor.get_struct_layout_id() == trimmed_layout_id && draft.original_layout_id.as_deref() != Some(trimmed_layout_id)
            });
        if conflicts_with_existing_layout {
            return Err(String::from("Struct layout id must be unique."));
        }

        let mut symbolic_field_definitions = Vec::with_capacity(draft.field_drafts.len());
        for field_draft in &draft.field_drafts {
            let trimmed_data_type_id = field_draft
                .data_type_selection
                .visible_data_type()
                .get_data_type_id()
                .trim()
                .to_string();
            if trimmed_data_type_id.is_empty() {
                return Err(String::from("Each field needs a data type."));
            }

            let container_type = field_draft.container_edit.to_container_type()?;
            let trimmed_field_name = field_draft.field_name.trim().to_string();
            let data_type_ref = DataTypeRef::new(&trimmed_data_type_id);
            let symbolic_field_definition = if trimmed_field_name.is_empty() {
                SymbolicFieldDefinition::new(data_type_ref, container_type)
            } else {
                SymbolicFieldDefinition::new_named(trimmed_field_name, data_type_ref, container_type)
            };

            symbolic_field_definitions.push(symbolic_field_definition);
        }

        Ok(StructLayoutDescriptor::new(
            trimmed_layout_id.to_string(),
            SymbolicStructDefinition::new(trimmed_layout_id.to_string(), symbolic_field_definitions),
        ))
    }

    pub fn apply_draft_to_catalog(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &StructLayoutEditDraft,
    ) -> Result<ProjectSymbolCatalog, String> {
        let resolved_struct_layout_descriptor = Self::build_struct_layout_descriptor(project_symbol_catalog, draft)?;
        let mut updated_project_symbol_catalog = project_symbol_catalog.clone();
        let mut updated_struct_layout_descriptors = updated_project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .filter(|struct_layout_descriptor| draft.original_layout_id.as_deref() != Some(struct_layout_descriptor.get_struct_layout_id()))
            .cloned()
            .collect::<Vec<_>>();

        updated_struct_layout_descriptors.push(resolved_struct_layout_descriptor.clone());
        updated_struct_layout_descriptors.sort_by(|left_layout, right_layout| {
            left_layout
                .get_struct_layout_id()
                .to_ascii_lowercase()
                .cmp(&right_layout.get_struct_layout_id().to_ascii_lowercase())
        });
        updated_project_symbol_catalog.set_struct_layout_descriptors(updated_struct_layout_descriptors);

        if let Some(original_layout_id) = draft.original_layout_id.as_deref() {
            if original_layout_id != resolved_struct_layout_descriptor.get_struct_layout_id() {
                for symbol_claim in updated_project_symbol_catalog.get_symbol_claims_mut() {
                    if symbol_claim.get_struct_layout_id() == original_layout_id {
                        symbol_claim.set_struct_layout_id(
                            resolved_struct_layout_descriptor
                                .get_struct_layout_id()
                                .to_string(),
                        );
                    }
                }
            }
        }

        Ok(updated_project_symbol_catalog)
    }

    pub fn remove_struct_layout_from_catalog(
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_id: &str,
    ) -> Result<ProjectSymbolCatalog, String> {
        if Self::count_symbol_claim_usages(project_symbol_catalog, struct_layout_id) > 0 {
            return Err(String::from("Struct layouts that are still used by symbol claims cannot be deleted."));
        }

        let mut updated_project_symbol_catalog = project_symbol_catalog.clone();
        updated_project_symbol_catalog.set_struct_layout_descriptors(
            updated_project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .filter(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() != struct_layout_id)
                .cloned()
                .collect(),
        );

        Ok(updated_project_symbol_catalog)
    }
}

impl Default for StructLayoutEditDraft {
    fn default() -> Self {
        Self {
            original_layout_id: None,
            layout_id: String::new(),
            field_drafts: vec![StructFieldEditDraft {
                field_name: String::new(),
                data_type_selection: DataTypeSelection::new(DataTypeRef::new(DataTypeI32::DATA_TYPE_ID)),
                container_edit: StructFieldContainerEdit::default(),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{StructEditorViewData, StructFieldEditDraft, StructLayoutEditDraft};
    use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
    use crate::views::struct_editor::view_data::struct_field_container_edit::{StructFieldContainerEdit, StructFieldContainerKind};
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef},
        data_values::container_type::ContainerType,
        projects::{project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim},
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    };

    fn create_project_symbol_catalog() -> ProjectSymbolCatalog {
        ProjectSymbolCatalog::new_with_symbol_claims(
            vec![StructLayoutDescriptor::new(
                String::from("player.stats"),
                SymbolicStructDefinition::new(
                    String::from("player.stats"),
                    vec![SymbolicFieldDefinition::new_named(
                        String::from("health"),
                        DataTypeRef::new("u32"),
                        ContainerType::None,
                    )],
                ),
            )],
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("sym.player"),
                String::from("Player"),
                0x1234,
                String::from("player.stats"),
            )],
        )
    }

    #[test]
    fn create_default_new_draft_picks_unique_layout_id() {
        let project_symbol_catalog = create_project_symbol_catalog();

        let draft = StructEditorViewData::create_default_new_draft(&project_symbol_catalog, DataTypeRef::new(DataTypeI32::DATA_TYPE_ID));

        assert_eq!(draft.layout_id, "new.struct");
        assert_eq!(
            draft.field_drafts.first().map(|field_draft| field_draft
                .data_type_selection
                .visible_data_type()
                .get_data_type_id()),
            Some(DataTypeI32::DATA_TYPE_ID)
        );
    }

    #[test]
    fn build_struct_layout_descriptor_parses_container_suffixes() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = StructLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            field_drafts: vec![StructFieldEditDraft {
                field_name: String::from("items"),
                data_type_selection: DataTypeSelection::new(DataTypeRef::new("u16")),
                container_edit: StructFieldContainerEdit {
                    kind: StructFieldContainerKind::FixedArray,
                    fixed_array_length: String::from("4"),
                    ..StructFieldContainerEdit::default()
                },
            }],
        };

        let struct_layout_descriptor = StructEditorViewData::build_struct_layout_descriptor(&project_symbol_catalog, &draft).expect("Expected draft to build.");

        assert_eq!(struct_layout_descriptor.get_struct_layout_id(), "inventory.slot");
        assert_eq!(
            struct_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .first()
                .map(SymbolicFieldDefinition::to_string),
            Some(String::from("items:u16[4]"))
        );
    }

    #[test]
    fn apply_draft_to_catalog_renames_symbol_claim_type_usage() {
        let project_symbol_catalog = create_project_symbol_catalog();
        let draft = StructLayoutEditDraft {
            original_layout_id: Some(String::from("player.stats")),
            layout_id: String::from("player.profile"),
            field_drafts: vec![StructFieldEditDraft {
                field_name: String::from("health"),
                data_type_selection: DataTypeSelection::new(DataTypeRef::new("u32")),
                container_edit: StructFieldContainerEdit::default(),
            }],
        };

        let updated_project_symbol_catalog = StructEditorViewData::apply_draft_to_catalog(&project_symbol_catalog, &draft).expect("Expected draft to apply.");

        assert!(
            updated_project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "player.profile")
        );
        assert_eq!(
            updated_project_symbol_catalog
                .get_symbol_claims()
                .first()
                .map(|symbol_claim| symbol_claim.get_struct_layout_id()),
            Some("player.profile")
        );
    }

    #[test]
    fn remove_struct_layout_from_catalog_rejects_in_use_layouts() {
        let project_symbol_catalog = create_project_symbol_catalog();

        let result = StructEditorViewData::remove_struct_layout_from_catalog(&project_symbol_catalog, "player.stats");

        assert!(result.is_err());
    }
}
