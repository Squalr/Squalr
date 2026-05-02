use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolTableTakeOverState {
    DeleteConfirmation { symbol_locator_key: String, display_name: String },
}

#[derive(Clone, Default)]
pub struct SymbolTableViewData {
    selected_symbol_locator_key: Option<String>,
    filter_text: String,
    take_over_state: Option<SymbolTableTakeOverState>,
}

impl SymbolTableViewData {
    pub fn new() -> Self {
        Self {
            selected_symbol_locator_key: None,
            filter_text: String::new(),
            take_over_state: None,
        }
    }

    pub fn get_selected_symbol_locator_key(&self) -> Option<&str> {
        self.selected_symbol_locator_key.as_deref()
    }

    pub fn get_filter_text(&self) -> &str {
        &self.filter_text
    }

    pub fn get_take_over_state(&self) -> Option<&SymbolTableTakeOverState> {
        self.take_over_state.as_ref()
    }

    pub fn set_selected_symbol_locator_key(
        symbol_table_view_data: Dependency<Self>,
        selected_symbol_locator_key: Option<String>,
    ) {
        if let Some(mut symbol_table_view_data) = symbol_table_view_data.write("Symbol table set selected symbol locator key") {
            symbol_table_view_data.selected_symbol_locator_key = selected_symbol_locator_key;
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

    pub fn request_delete_confirmation(
        symbol_table_view_data: Dependency<Self>,
        symbol_locator_key: String,
        display_name: String,
    ) {
        if let Some(mut symbol_table_view_data) = symbol_table_view_data.write("Symbol table request delete confirmation") {
            symbol_table_view_data.take_over_state = Some(SymbolTableTakeOverState::DeleteConfirmation {
                symbol_locator_key,
                display_name,
            });
        }
    }

    pub fn cancel_take_over_state(symbol_table_view_data: Dependency<Self>) {
        if let Some(mut symbol_table_view_data) = symbol_table_view_data.write("Symbol table cancel take over state") {
            symbol_table_view_data.take_over_state = None;
        }
    }

    pub fn synchronize_selection(
        symbol_table_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        if let Some(mut symbol_table_view_data) = symbol_table_view_data.write("Symbol table synchronize selection") {
            let has_valid_selection = symbol_table_view_data
                .selected_symbol_locator_key
                .as_ref()
                .is_some_and(|selected_symbol_locator_key| {
                    project_symbol_catalog
                        .find_symbol_claim(selected_symbol_locator_key)
                        .is_some()
                });

            if has_valid_selection {
                return;
            }

            symbol_table_view_data.selected_symbol_locator_key = project_symbol_catalog
                .get_symbol_claims()
                .first()
                .map(|symbol_claim| symbol_claim.get_symbol_locator_key().to_string());
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
            Some(SymbolTableTakeOverState::DeleteConfirmation { symbol_locator_key, .. }) => project_symbol_catalog
                .find_symbol_claim(symbol_locator_key)
                .is_none(),
            _ => false,
        };

        if should_clear_take_over_state {
            symbol_table_view_data.take_over_state = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolTableViewData;
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

        let selected_symbol_locator_key = symbol_table_view_data
            .read("Symbol table synchronize selection test")
            .and_then(|symbol_table_view_data| {
                symbol_table_view_data
                    .get_selected_symbol_locator_key()
                    .map(str::to_string)
            });

        assert_eq!(selected_symbol_locator_key, Some(String::from("absolute:1234")));
    }
}
