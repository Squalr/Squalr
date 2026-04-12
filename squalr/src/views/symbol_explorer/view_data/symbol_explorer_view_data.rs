use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolExplorerSelection {
    RootedSymbol(String),
    StructLayout(String),
}

#[derive(Clone, Default)]
pub struct SymbolExplorerViewData {
    selected_entry: Option<SymbolExplorerSelection>,
}

impl SymbolExplorerViewData {
    pub fn new() -> Self {
        Self { selected_entry: None }
    }

    pub fn get_selected_entry(&self) -> Option<&SymbolExplorerSelection> {
        self.selected_entry.as_ref()
    }

    pub fn set_selected_entry(
        symbol_explorer_view_data: Dependency<Self>,
        selected_entry: Option<SymbolExplorerSelection>,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer set selected entry") {
            symbol_explorer_view_data.selected_entry = selected_entry;
        }
    }

    pub fn synchronize_selection(
        symbol_explorer_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer synchronize selection") {
            let has_valid_selection = symbol_explorer_view_data
                .selected_entry
                .as_ref()
                .is_some_and(|selected_entry| Self::selection_exists(project_symbol_catalog, selected_entry));

            if has_valid_selection {
                return;
            }

            symbol_explorer_view_data.selected_entry = project_symbol_catalog
                .get_rooted_symbols()
                .first()
                .map(|rooted_symbol| SymbolExplorerSelection::RootedSymbol(rooted_symbol.get_symbol_key().to_string()))
                .or_else(|| {
                    project_symbol_catalog
                        .get_struct_layout_descriptors()
                        .first()
                        .map(|struct_layout_descriptor| SymbolExplorerSelection::StructLayout(struct_layout_descriptor.get_struct_layout_id().to_string()))
                });
        }
    }

    fn selection_exists(
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_entry: &SymbolExplorerSelection,
    ) -> bool {
        match selected_entry {
            SymbolExplorerSelection::RootedSymbol(symbol_key) => project_symbol_catalog
                .get_rooted_symbols()
                .iter()
                .any(|rooted_symbol| rooted_symbol.get_symbol_key() == symbol_key),
            SymbolExplorerSelection::StructLayout(struct_layout_id) => project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == struct_layout_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolExplorerSelection, SymbolExplorerViewData};
    use squalr_engine_api::dependency_injection::dependency::Dependency;
    use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        projects::{project_root_symbol::ProjectRootSymbol, project_symbol_catalog::ProjectSymbolCatalog},
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    };

    fn create_dependency() -> Dependency<SymbolExplorerViewData> {
        let dependency_container = DependencyContainer::new();

        dependency_container.register(SymbolExplorerViewData::new())
    }

    #[test]
    fn synchronize_selection_prefers_first_rooted_symbol() {
        let symbol_explorer_view_data = create_dependency();
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_rooted_symbols(
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
            vec![ProjectRootSymbol::new_absolute_address(
                String::from("sym.player"),
                String::from("Player"),
                0x1234,
                String::from("player.stats"),
            )],
        );

        SymbolExplorerViewData::synchronize_selection(symbol_explorer_view_data.clone(), &project_symbol_catalog);

        let selected_entry = symbol_explorer_view_data
            .read("Symbol explorer synchronize selection test")
            .and_then(|symbol_explorer_view_data| symbol_explorer_view_data.get_selected_entry().cloned());

        assert_eq!(selected_entry, Some(SymbolExplorerSelection::RootedSymbol(String::from("sym.player"))));
    }
}
