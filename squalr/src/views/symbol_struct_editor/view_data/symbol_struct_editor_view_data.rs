use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use crate::views::symbol_struct_editor::view_data::symbol_struct_field_container_edit::SymbolStructFieldContainerEdit;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use squalr_engine_api::structures::{
    data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef},
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolStructFieldEditDraft {
    pub field_name: String,
    pub data_type_selection: DataTypeSelection,
    pub container_edit: SymbolStructFieldContainerEdit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolStructLayoutEditDraft {
    pub original_layout_id: Option<String>,
    pub layout_id: String,
    pub field_drafts: Vec<SymbolStructFieldEditDraft>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolStructEditorTakeOverState {
    CreateStructLayout,
    EditStructLayout { layout_id: String },
    DeleteConfirmation { layout_id: String },
}

#[derive(Clone, Default)]
pub struct SymbolStructEditorViewData {
    selected_layout_id: Option<String>,
    filter_text: String,
    take_over_state: Option<SymbolStructEditorTakeOverState>,
    baseline_draft: Option<SymbolStructLayoutEditDraft>,
    draft: Option<SymbolStructLayoutEditDraft>,
}

impl SymbolStructEditorViewData {
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

    pub fn get_take_over_state(&self) -> Option<&SymbolStructEditorTakeOverState> {
        self.take_over_state.as_ref()
    }

    pub fn get_draft(&self) -> Option<&SymbolStructLayoutEditDraft> {
        self.draft.as_ref()
    }

    pub fn get_baseline_draft(&self) -> Option<&SymbolStructLayoutEditDraft> {
        self.baseline_draft.as_ref()
    }

    pub fn set_filter_text(
        symbol_struct_editor_view_data: Dependency<Self>,
        filter_text: String,
    ) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor set filter text") {
            symbol_struct_editor_view_data.filter_text = filter_text;
        }
    }

    pub fn update_draft(
        symbol_struct_editor_view_data: Dependency<Self>,
        draft: SymbolStructLayoutEditDraft,
    ) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor update draft") {
            symbol_struct_editor_view_data.draft = Some(draft);
        }
    }

    pub fn select_struct_layout(
        symbol_struct_editor_view_data: Dependency<Self>,
        selected_layout_id: Option<String>,
    ) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor select struct layout") {
            symbol_struct_editor_view_data.selected_layout_id = selected_layout_id;
        }
    }

    pub fn begin_create_struct_layout(
        symbol_struct_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        default_data_type_ref: DataTypeRef,
    ) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor begin create struct layout") {
            symbol_struct_editor_view_data.selected_layout_id = None;
            symbol_struct_editor_view_data.take_over_state = Some(SymbolStructEditorTakeOverState::CreateStructLayout);
            let baseline_draft = Self::create_default_new_draft(project_symbol_catalog, default_data_type_ref);
            symbol_struct_editor_view_data.baseline_draft = Some(baseline_draft.clone());
            symbol_struct_editor_view_data.draft = Some(baseline_draft);
        }
    }

    pub fn begin_edit_struct_layout(
        symbol_struct_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
    ) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor begin edit struct layout") {
            symbol_struct_editor_view_data.selected_layout_id = Some(layout_id.to_string());
            symbol_struct_editor_view_data.take_over_state = Some(SymbolStructEditorTakeOverState::EditStructLayout {
                layout_id: layout_id.to_string(),
            });
            symbol_struct_editor_view_data.baseline_draft = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id)
                .map(Self::create_draft_from_descriptor);
            symbol_struct_editor_view_data.draft = symbol_struct_editor_view_data.baseline_draft.clone();
        }
    }

    pub fn request_delete_confirmation(
        symbol_struct_editor_view_data: Dependency<Self>,
        layout_id: String,
    ) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor request delete confirmation") {
            symbol_struct_editor_view_data.take_over_state = Some(SymbolStructEditorTakeOverState::DeleteConfirmation { layout_id });
        }
    }

    pub fn cancel_take_over_state(symbol_struct_editor_view_data: Dependency<Self>) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor cancel take over state") {
            symbol_struct_editor_view_data.take_over_state = None;
            symbol_struct_editor_view_data.baseline_draft = None;
            symbol_struct_editor_view_data.draft = None;
        }
    }

    pub fn synchronize(
        symbol_struct_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor synchronize") else {
            return;
        };

        let next_selected_layout_id = symbol_struct_editor_view_data
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

        symbol_struct_editor_view_data.selected_layout_id = next_selected_layout_id.clone();

        let should_clear_take_over_state = match symbol_struct_editor_view_data.take_over_state.as_ref() {
            Some(SymbolStructEditorTakeOverState::CreateStructLayout) => false,
            Some(SymbolStructEditorTakeOverState::EditStructLayout { layout_id }) | Some(SymbolStructEditorTakeOverState::DeleteConfirmation { layout_id }) => {
                !project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id)
            }
            None => false,
        };

        if should_clear_take_over_state {
            symbol_struct_editor_view_data.take_over_state = None;
            symbol_struct_editor_view_data.baseline_draft = None;
            symbol_struct_editor_view_data.draft = None;
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

    pub fn create_draft_from_descriptor(struct_layout_descriptor: &StructLayoutDescriptor) -> SymbolStructLayoutEditDraft {
        SymbolStructLayoutEditDraft {
            original_layout_id: Some(struct_layout_descriptor.get_struct_layout_id().to_string()),
            layout_id: struct_layout_descriptor.get_struct_layout_id().to_string(),
            field_drafts: struct_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .iter()
                .map(|symbolic_field_definition| SymbolStructFieldEditDraft {
                    field_name: symbolic_field_definition.get_field_name().to_string(),
                    data_type_selection: DataTypeSelection::new(symbolic_field_definition.get_data_type_ref().clone()),
                    container_edit: SymbolStructFieldContainerEdit::from_container_type(symbolic_field_definition.get_container_type()),
                })
                .collect(),
        }
    }

    pub fn create_default_new_draft(
        project_symbol_catalog: &ProjectSymbolCatalog,
        default_data_type_ref: DataTypeRef,
    ) -> SymbolStructLayoutEditDraft {
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

        SymbolStructLayoutEditDraft {
            original_layout_id: None,
            layout_id: proposed_layout_id,
            field_drafts: vec![SymbolStructFieldEditDraft {
                field_name: String::new(),
                data_type_selection: DataTypeSelection::new(default_data_type_ref),
                container_edit: SymbolStructFieldContainerEdit::default(),
            }],
        }
    }

    pub fn build_struct_layout_descriptor(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolStructLayoutEditDraft,
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
        let mut field_names = HashSet::new();
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
            if !trimmed_field_name.is_empty() && !field_names.insert(trimmed_field_name.clone()) {
                return Err(format!("Field name `{}` is already used in this struct.", trimmed_field_name));
            }

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
        draft: &SymbolStructLayoutEditDraft,
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

impl Default for SymbolStructLayoutEditDraft {
    fn default() -> Self {
        Self {
            original_layout_id: None,
            layout_id: String::new(),
            field_drafts: vec![SymbolStructFieldEditDraft {
                field_name: String::new(),
                data_type_selection: DataTypeSelection::new(DataTypeRef::new(DataTypeI32::DATA_TYPE_ID)),
                container_edit: SymbolStructFieldContainerEdit::default(),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolStructEditorViewData, SymbolStructFieldEditDraft, SymbolStructLayoutEditDraft};
    use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
    use crate::views::symbol_struct_editor::view_data::symbol_struct_field_container_edit::{SymbolStructFieldContainerEdit, SymbolStructFieldContainerKind};
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
                String::from("Player"),
                0x1234,
                String::from("player.stats"),
            )],
        )
    }

    #[test]
    fn create_default_new_draft_picks_unique_layout_id() {
        let project_symbol_catalog = create_project_symbol_catalog();

        let draft = SymbolStructEditorViewData::create_default_new_draft(&project_symbol_catalog, DataTypeRef::new(DataTypeI32::DATA_TYPE_ID));

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
        let draft = SymbolStructLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            field_drafts: vec![SymbolStructFieldEditDraft {
                field_name: String::from("items"),
                data_type_selection: DataTypeSelection::new(DataTypeRef::new("u16")),
                container_edit: SymbolStructFieldContainerEdit {
                    kind: SymbolStructFieldContainerKind::FixedArray,
                    fixed_array_length: String::from("4"),
                    ..SymbolStructFieldContainerEdit::default()
                },
            }],
        };

        let struct_layout_descriptor =
            SymbolStructEditorViewData::build_struct_layout_descriptor(&project_symbol_catalog, &draft).expect("Expected draft to build.");

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
    fn build_struct_layout_descriptor_rejects_duplicate_field_names() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolStructLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("timer.state"),
            field_drafts: vec![
                SymbolStructFieldEditDraft {
                    field_name: String::from("Timer"),
                    data_type_selection: DataTypeSelection::new(DataTypeRef::new("u32")),
                    container_edit: SymbolStructFieldContainerEdit::default(),
                },
                SymbolStructFieldEditDraft {
                    field_name: String::from("Timer"),
                    data_type_selection: DataTypeSelection::new(DataTypeRef::new("u32")),
                    container_edit: SymbolStructFieldContainerEdit::default(),
                },
            ],
        };

        let result = SymbolStructEditorViewData::build_struct_layout_descriptor(&project_symbol_catalog, &draft);

        assert!(result.is_err());
    }

    #[test]
    fn apply_draft_to_catalog_renames_symbol_claim_type_usage() {
        let project_symbol_catalog = create_project_symbol_catalog();
        let draft = SymbolStructLayoutEditDraft {
            original_layout_id: Some(String::from("player.stats")),
            layout_id: String::from("player.profile"),
            field_drafts: vec![SymbolStructFieldEditDraft {
                field_name: String::from("health"),
                data_type_selection: DataTypeSelection::new(DataTypeRef::new("u32")),
                container_edit: SymbolStructFieldContainerEdit::default(),
            }],
        };

        let updated_project_symbol_catalog =
            SymbolStructEditorViewData::apply_draft_to_catalog(&project_symbol_catalog, &draft).expect("Expected draft to apply.");

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

        let result = SymbolStructEditorViewData::remove_struct_layout_from_catalog(&project_symbol_catalog, "player.stats");

        assert!(result.is_err());
    }
}
