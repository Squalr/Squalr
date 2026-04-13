use crate::views::symbol_explorer::view_data::symbol_tree_entry::{SymbolTreeEntry, SymbolTreeEntryKind};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolExplorerSelection {
    RootedSymbol(String),
    DerivedNode(String),
    StructLayout(String),
    CreateRootedSymbol,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum RootedSymbolDraftLocatorMode {
    #[default]
    AbsoluteAddress,
    ModuleOffset,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RootedSymbolCreateDraft {
    pub display_name: String,
    pub struct_layout_id: String,
    pub locator_mode: RootedSymbolDraftLocatorMode,
    pub address_text: String,
    pub module_name: String,
    pub offset_text: String,
}

#[derive(Clone, Default)]
pub struct SymbolExplorerViewData {
    selected_entry: Option<SymbolExplorerSelection>,
    rooted_symbol_display_name_draft: String,
    rooted_symbol_create_draft: RootedSymbolCreateDraft,
    expanded_tree_node_keys: HashSet<String>,
}

impl SymbolExplorerViewData {
    pub fn new() -> Self {
        Self {
            selected_entry: None,
            rooted_symbol_display_name_draft: String::new(),
            rooted_symbol_create_draft: RootedSymbolCreateDraft::default(),
            expanded_tree_node_keys: HashSet::new(),
        }
    }

    pub fn get_selected_entry(&self) -> Option<&SymbolExplorerSelection> {
        self.selected_entry.as_ref()
    }

    pub fn get_rooted_symbol_display_name_draft(&self) -> &str {
        &self.rooted_symbol_display_name_draft
    }

    pub fn get_rooted_symbol_create_draft(&self) -> &RootedSymbolCreateDraft {
        &self.rooted_symbol_create_draft
    }

    pub fn get_expanded_tree_node_keys(&self) -> &HashSet<String> {
        &self.expanded_tree_node_keys
    }

    pub fn set_selected_entry(
        symbol_explorer_view_data: Dependency<Self>,
        selected_entry: Option<SymbolExplorerSelection>,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer set selected entry") {
            symbol_explorer_view_data.selected_entry = selected_entry;
            symbol_explorer_view_data
                .rooted_symbol_display_name_draft
                .clear();
            symbol_explorer_view_data.rooted_symbol_create_draft = RootedSymbolCreateDraft::default();
        }
    }

    pub fn begin_create_rooted_symbol(
        symbol_explorer_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer begin create rooted symbol") {
            symbol_explorer_view_data.selected_entry = Some(SymbolExplorerSelection::CreateRootedSymbol);
            symbol_explorer_view_data
                .rooted_symbol_display_name_draft
                .clear();
            symbol_explorer_view_data.rooted_symbol_create_draft = Self::create_default_rooted_symbol_create_draft(project_symbol_catalog);
        }
    }

    pub fn set_rooted_symbol_display_name_draft(
        symbol_explorer_view_data: Dependency<Self>,
        rooted_symbol_display_name_draft: String,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer set rooted symbol display name draft") {
            symbol_explorer_view_data.rooted_symbol_display_name_draft = rooted_symbol_display_name_draft;
        }
    }

    pub fn set_rooted_symbol_create_draft(
        symbol_explorer_view_data: Dependency<Self>,
        rooted_symbol_create_draft: RootedSymbolCreateDraft,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer set rooted symbol create draft") {
            symbol_explorer_view_data.rooted_symbol_create_draft = rooted_symbol_create_draft;
        }
    }

    pub fn toggle_tree_node_expansion(
        symbol_explorer_view_data: Dependency<Self>,
        tree_node_key: &str,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer toggle tree node expansion") {
            if symbol_explorer_view_data
                .expanded_tree_node_keys
                .contains(tree_node_key)
            {
                symbol_explorer_view_data
                    .expanded_tree_node_keys
                    .remove(tree_node_key);
            } else {
                symbol_explorer_view_data
                    .expanded_tree_node_keys
                    .insert(tree_node_key.to_string());
            }
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

            symbol_explorer_view_data
                .rooted_symbol_display_name_draft
                .clear();
            symbol_explorer_view_data.rooted_symbol_create_draft = RootedSymbolCreateDraft::default();
        }
    }

    pub fn synchronize_rooted_symbol_display_name_draft(
        symbol_explorer_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer synchronize rooted symbol display name draft") else {
            return;
        };
        let Some(SymbolExplorerSelection::RootedSymbol(selected_symbol_key)) = symbol_explorer_view_data.selected_entry.as_ref() else {
            symbol_explorer_view_data
                .rooted_symbol_display_name_draft
                .clear();
            return;
        };
        let Some(rooted_symbol) = project_symbol_catalog
            .get_rooted_symbols()
            .iter()
            .find(|rooted_symbol| rooted_symbol.get_symbol_key() == selected_symbol_key)
        else {
            symbol_explorer_view_data
                .rooted_symbol_display_name_draft
                .clear();
            return;
        };

        if symbol_explorer_view_data
            .rooted_symbol_display_name_draft
            .is_empty()
        {
            symbol_explorer_view_data.rooted_symbol_display_name_draft = rooted_symbol.get_display_name().to_string();
        }
    }

    pub fn synchronize_rooted_symbol_create_draft(
        symbol_explorer_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer synchronize rooted symbol create draft") else {
            return;
        };

        if !matches!(symbol_explorer_view_data.selected_entry, Some(SymbolExplorerSelection::CreateRootedSymbol)) {
            return;
        }

        if symbol_explorer_view_data
            .rooted_symbol_create_draft
            .struct_layout_id
            .is_empty()
        {
            symbol_explorer_view_data
                .rooted_symbol_create_draft
                .struct_layout_id = project_symbol_catalog
                .get_struct_layout_descriptors()
                .first()
                .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id().to_string())
                .unwrap_or_default();
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
            SymbolExplorerSelection::DerivedNode(_) => true,
            SymbolExplorerSelection::CreateRootedSymbol => true,
        }
    }

    pub fn synchronize_selection_to_tree_entries(
        symbol_explorer_view_data: Dependency<Self>,
        symbol_tree_entries: &[SymbolTreeEntry],
    ) {
        let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer synchronize selection to tree entries") else {
            return;
        };
        let Some(SymbolExplorerSelection::DerivedNode(selected_node_key)) = symbol_explorer_view_data.selected_entry.as_ref() else {
            return;
        };

        if symbol_tree_entries
            .iter()
            .any(|symbol_tree_entry| symbol_tree_entry.get_node_key() == selected_node_key)
        {
            return;
        }

        symbol_explorer_view_data.selected_entry = symbol_tree_entries
            .first()
            .map(|symbol_tree_entry| match symbol_tree_entry.get_kind() {
                SymbolTreeEntryKind::RootedSymbol { symbol_key } => SymbolExplorerSelection::RootedSymbol(symbol_key.to_string()),
                SymbolTreeEntryKind::StructField | SymbolTreeEntryKind::ArrayElement => {
                    SymbolExplorerSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string())
                }
            });
    }

    fn create_default_rooted_symbol_create_draft(project_symbol_catalog: &ProjectSymbolCatalog) -> RootedSymbolCreateDraft {
        RootedSymbolCreateDraft {
            struct_layout_id: project_symbol_catalog
                .get_struct_layout_descriptors()
                .first()
                .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id().to_string())
                .unwrap_or_default(),
            ..RootedSymbolCreateDraft::default()
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

    #[test]
    fn begin_create_rooted_symbol_prefills_first_struct_layout_id() {
        let symbol_explorer_view_data = create_dependency();
        let project_symbol_catalog = ProjectSymbolCatalog::new(vec![StructLayoutDescriptor::new(
            String::from("player.stats"),
            SymbolicStructDefinition::new(
                String::from("player.stats"),
                vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new("u32"),
                    ContainerType::None,
                )],
            ),
        )]);

        SymbolExplorerViewData::begin_create_rooted_symbol(symbol_explorer_view_data.clone(), &project_symbol_catalog);

        let symbol_explorer_view_data = symbol_explorer_view_data
            .read("Symbol explorer begin create rooted symbol test")
            .expect("Expected symbol explorer dependency read access in test.");

        assert_eq!(
            symbol_explorer_view_data.get_selected_entry(),
            Some(&SymbolExplorerSelection::CreateRootedSymbol)
        );
        assert_eq!(
            symbol_explorer_view_data
                .get_rooted_symbol_create_draft()
                .struct_layout_id,
            "player.stats"
        );
    }
}
