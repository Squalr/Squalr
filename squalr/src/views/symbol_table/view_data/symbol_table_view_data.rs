use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SymbolClaimDraftLocatorMode {
    #[default]
    AbsoluteAddress,
    ModuleOffset,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SymbolClaimCreateDraft {
    pub display_name: String,
    pub struct_layout_id: String,
    pub locator_mode: SymbolClaimDraftLocatorMode,
    pub address_text: String,
    pub module_name: String,
    pub offset_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolTableTakeOverState {
    CreateSymbolClaim,
    DeleteConfirmation { symbol_key: String, display_name: String },
}

#[derive(Clone, Default)]
pub struct SymbolTableViewData {
    selected_symbol_key: Option<String>,
    filter_text: String,
    take_over_state: Option<SymbolTableTakeOverState>,
    symbol_claim_create_draft: SymbolClaimCreateDraft,
}

impl SymbolTableViewData {
    pub fn new() -> Self {
        Self {
            selected_symbol_key: None,
            filter_text: String::new(),
            take_over_state: None,
            symbol_claim_create_draft: SymbolClaimCreateDraft::default(),
        }
    }

    pub fn get_selected_symbol_key(&self) -> Option<&str> {
        self.selected_symbol_key.as_deref()
    }

    pub fn get_filter_text(&self) -> &str {
        &self.filter_text
    }

    pub fn get_take_over_state(&self) -> Option<&SymbolTableTakeOverState> {
        self.take_over_state.as_ref()
    }

    pub fn get_symbol_claim_create_draft(&self) -> &SymbolClaimCreateDraft {
        &self.symbol_claim_create_draft
    }

    pub fn set_selected_symbol_key(
        symbol_table_view_data: Dependency<Self>,
        selected_symbol_key: Option<String>,
    ) {
        if let Some(mut symbol_table_view_data) = symbol_table_view_data.write("Symbol table set selected symbol key") {
            symbol_table_view_data.selected_symbol_key = selected_symbol_key;
        }
    }

    pub fn set_filter_text(
        symbol_table_view_data: Dependency<Self>,
        filter_text: String,
    ) {
        if let Some(mut symbol_table_view_data) = symbol_table_view_data.write("Symbol table set filter text") {
            symbol_table_view_data.filter_text = filter_text;
        }
    }

    pub fn begin_create_symbol_claim(
        symbol_table_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        if let Some(mut symbol_table_view_data) = symbol_table_view_data.write("Symbol table begin create symbol claim") {
            symbol_table_view_data.take_over_state = Some(SymbolTableTakeOverState::CreateSymbolClaim);
            symbol_table_view_data.symbol_claim_create_draft = Self::create_default_symbol_claim_create_draft(project_symbol_catalog);
        }
    }

    pub fn request_delete_confirmation(
        symbol_table_view_data: Dependency<Self>,
        symbol_key: String,
        display_name: String,
    ) {
        if let Some(mut symbol_table_view_data) = symbol_table_view_data.write("Symbol table request delete confirmation") {
            symbol_table_view_data.take_over_state = Some(SymbolTableTakeOverState::DeleteConfirmation { symbol_key, display_name });
        }
    }

    pub fn cancel_take_over_state(symbol_table_view_data: Dependency<Self>) {
        if let Some(mut symbol_table_view_data) = symbol_table_view_data.write("Symbol table cancel take over state") {
            symbol_table_view_data.take_over_state = None;
        }
    }

    pub fn set_symbol_claim_create_draft(
        symbol_table_view_data: Dependency<Self>,
        symbol_claim_create_draft: SymbolClaimCreateDraft,
    ) {
        if let Some(mut symbol_table_view_data) = symbol_table_view_data.write("Symbol table set symbol claim create draft") {
            symbol_table_view_data.symbol_claim_create_draft = symbol_claim_create_draft;
        }
    }

    pub fn synchronize_selection(
        symbol_table_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        if let Some(mut symbol_table_view_data) = symbol_table_view_data.write("Symbol table synchronize selection") {
            let has_valid_selection = symbol_table_view_data
                .selected_symbol_key
                .as_ref()
                .is_some_and(|selected_symbol_key| {
                    project_symbol_catalog
                        .find_symbol_claim(selected_symbol_key)
                        .is_some()
                });

            if has_valid_selection {
                return;
            }

            symbol_table_view_data.selected_symbol_key = project_symbol_catalog
                .get_symbol_claims()
                .first()
                .map(|symbol_claim| symbol_claim.get_symbol_key().to_string());
        }
    }

    pub fn synchronize_symbol_claim_create_draft(
        symbol_table_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let Some(mut symbol_table_view_data) = symbol_table_view_data.write("Symbol table synchronize symbol claim create draft") else {
            return;
        };

        if !matches!(symbol_table_view_data.take_over_state, Some(SymbolTableTakeOverState::CreateSymbolClaim)) {
            return;
        }

        if symbol_table_view_data
            .symbol_claim_create_draft
            .struct_layout_id
            .is_empty()
        {
            symbol_table_view_data.symbol_claim_create_draft = Self::create_default_symbol_claim_create_draft(project_symbol_catalog);
        }
    }

    pub fn synchronize_take_over_state(
        symbol_table_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let Some(mut symbol_table_view_data) = symbol_table_view_data.write("Symbol table synchronize take over state") else {
            return;
        };

        let should_clear_take_over_state = match symbol_table_view_data.take_over_state.as_ref() {
            Some(SymbolTableTakeOverState::DeleteConfirmation { symbol_key, .. }) => project_symbol_catalog.find_symbol_claim(symbol_key).is_none(),
            _ => false,
        };

        if should_clear_take_over_state {
            symbol_table_view_data.take_over_state = None;
        }
    }

    fn create_default_symbol_claim_create_draft(project_symbol_catalog: &ProjectSymbolCatalog) -> SymbolClaimCreateDraft {
        SymbolClaimCreateDraft {
            struct_layout_id: project_symbol_catalog
                .get_struct_layout_descriptors()
                .first()
                .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id().to_string())
                .unwrap_or_default(),
            ..SymbolClaimCreateDraft::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolTableTakeOverState, SymbolTableViewData};
    use squalr_engine_api::dependency_injection::dependency::Dependency;
    use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        projects::{project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim},
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    };

    fn create_dependency() -> Dependency<SymbolTableViewData> {
        let dependency_container = DependencyContainer::new();

        dependency_container.register(SymbolTableViewData::new())
    }

    fn create_project_symbol_catalog() -> ProjectSymbolCatalog {
        ProjectSymbolCatalog::new_with_symbol_claims(
            vec![StructLayoutDescriptor::new(
                String::from("player.stats"),
                SymbolicStructDefinition::new(
                    String::from("player.stats"),
                    vec![SymbolicFieldDefinition::new(
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
    fn synchronize_selection_prefers_first_symbol_claim() {
        let symbol_table_view_data = create_dependency();
        let project_symbol_catalog = create_project_symbol_catalog();

        SymbolTableViewData::synchronize_selection(symbol_table_view_data.clone(), &project_symbol_catalog);

        let selected_symbol_key = symbol_table_view_data
            .read("Symbol table synchronize selection test")
            .and_then(|symbol_table_view_data| {
                symbol_table_view_data
                    .get_selected_symbol_key()
                    .map(str::to_string)
            });

        assert_eq!(selected_symbol_key, Some(String::from("sym.player")));
    }

    #[test]
    fn begin_create_symbol_claim_prefills_first_struct_layout_id() {
        let symbol_table_view_data = create_dependency();
        let project_symbol_catalog = create_project_symbol_catalog();

        SymbolTableViewData::begin_create_symbol_claim(symbol_table_view_data.clone(), &project_symbol_catalog);

        let symbol_table_view_data = symbol_table_view_data
            .read("Symbol table begin create symbol claim test")
            .expect("Expected symbol table dependency read access in test.");

        assert_eq!(symbol_table_view_data.get_take_over_state(), Some(&SymbolTableTakeOverState::CreateSymbolClaim));
        assert_eq!(
            symbol_table_view_data
                .get_symbol_claim_create_draft()
                .struct_layout_id,
            "player.stats"
        );
    }
}
