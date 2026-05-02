use crate::views::symbol_explorer::view_data::symbol_tree_entry::{SymbolTreeEntry, SymbolTreeEntryKind};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolExplorerSelection {
    ModuleRoot(String),
    SymbolClaim(String),
    DerivedNode(String),
    CreateModuleRoot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolExplorerTakeOverState {
    DeleteConfirmation { symbol_locator_key: String, display_name: String },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModuleRootCreateDraft {
    pub module_name: String,
    pub size_text: String,
}

#[derive(Clone, Default)]
pub struct SymbolExplorerViewData {
    selected_entry: Option<SymbolExplorerSelection>,
    take_over_state: Option<SymbolExplorerTakeOverState>,
    module_root_create_draft: ModuleRootCreateDraft,
    inline_rename_tree_node_key: Option<String>,
    expanded_tree_node_keys: HashSet<String>,
}

impl SymbolExplorerViewData {
    pub fn new() -> Self {
        Self {
            selected_entry: None,
            take_over_state: None,
            module_root_create_draft: ModuleRootCreateDraft::default(),
            inline_rename_tree_node_key: None,
            expanded_tree_node_keys: HashSet::new(),
        }
    }

    pub fn get_selected_entry(&self) -> Option<&SymbolExplorerSelection> {
        self.selected_entry.as_ref()
    }

    pub fn get_take_over_state(&self) -> Option<&SymbolExplorerTakeOverState> {
        self.take_over_state.as_ref()
    }

    pub fn get_module_root_create_draft(&self) -> &ModuleRootCreateDraft {
        &self.module_root_create_draft
    }

    pub fn get_inline_rename_tree_node_key(&self) -> Option<&str> {
        self.inline_rename_tree_node_key.as_deref()
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
            symbol_explorer_view_data.take_over_state = None;
            symbol_explorer_view_data.module_root_create_draft = ModuleRootCreateDraft::default();
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
        }
    }

    pub fn begin_create_module_root(symbol_explorer_view_data: Dependency<Self>) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer begin create module root") {
            symbol_explorer_view_data.selected_entry = Some(SymbolExplorerSelection::CreateModuleRoot);
            symbol_explorer_view_data.take_over_state = None;
            symbol_explorer_view_data.module_root_create_draft = ModuleRootCreateDraft::default();
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
        }
    }

    pub fn begin_inline_rename(
        symbol_explorer_view_data: Dependency<Self>,
        tree_node_key: String,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer begin inline rename") {
            symbol_explorer_view_data.inline_rename_tree_node_key = Some(tree_node_key);
            symbol_explorer_view_data.take_over_state = None;
        }
    }

    pub fn cancel_inline_rename(symbol_explorer_view_data: Dependency<Self>) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer cancel inline rename") {
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
        }
    }

    pub fn set_module_root_create_draft(
        symbol_explorer_view_data: Dependency<Self>,
        module_root_create_draft: ModuleRootCreateDraft,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer set module root create draft") {
            symbol_explorer_view_data.module_root_create_draft = module_root_create_draft;
        }
    }

    pub fn request_delete_confirmation(
        symbol_explorer_view_data: Dependency<Self>,
        symbol_locator_key: String,
        display_name: String,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer request delete confirmation") {
            symbol_explorer_view_data.take_over_state = Some(SymbolExplorerTakeOverState::DeleteConfirmation {
                symbol_locator_key,
                display_name,
            });
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
        }
    }

    pub fn cancel_take_over_state(symbol_explorer_view_data: Dependency<Self>) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer cancel take over state") {
            symbol_explorer_view_data.take_over_state = None;
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

    pub fn expand_tree_node(
        symbol_explorer_view_data: Dependency<Self>,
        tree_node_key: &str,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer expand tree node") {
            symbol_explorer_view_data
                .expanded_tree_node_keys
                .insert(tree_node_key.to_string());
        }
    }

    pub fn synchronize_selection(
        symbol_explorer_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        suppress_default_selection: bool,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer synchronize selection") {
            let has_valid_selection = symbol_explorer_view_data
                .selected_entry
                .as_ref()
                .is_some_and(|selected_entry| Self::selection_exists(project_symbol_catalog, selected_entry));

            if has_valid_selection {
                return;
            }

            if suppress_default_selection {
                symbol_explorer_view_data.selected_entry = None;
                symbol_explorer_view_data.take_over_state = None;
                symbol_explorer_view_data.module_root_create_draft = ModuleRootCreateDraft::default();
                symbol_explorer_view_data.inline_rename_tree_node_key = None;
                return;
            }

            symbol_explorer_view_data.selected_entry = project_symbol_catalog
                .get_symbol_modules()
                .first()
                .map(|symbol_module| SymbolExplorerSelection::ModuleRoot(symbol_module.get_module_name().to_string()))
                .or_else(|| {
                    project_symbol_catalog
                        .get_symbol_claims()
                        .first()
                        .map(|symbol_claim| SymbolExplorerSelection::SymbolClaim(symbol_claim.get_symbol_locator_key().to_string()))
                });

            symbol_explorer_view_data.take_over_state = None;
            symbol_explorer_view_data.module_root_create_draft = ModuleRootCreateDraft::default();
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
        }
    }

    pub fn synchronize_inline_rename(
        symbol_explorer_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer synchronize inline rename state") else {
            return;
        };
        let Some(inline_rename_tree_node_key) = symbol_explorer_view_data.inline_rename_tree_node_key.as_ref() else {
            return;
        };

        let is_rename_target_still_present = if let Some(module_name) = inline_rename_tree_node_key.strip_prefix("module:") {
            project_symbol_catalog.find_symbol_module(module_name).is_some()
        } else {
            let symbol_locator_key = inline_rename_tree_node_key
                .strip_prefix("claim:")
                .unwrap_or(inline_rename_tree_node_key);

            project_symbol_catalog
                .get_symbol_claims()
                .iter()
                .any(|symbol_claim| symbol_claim.get_symbol_locator_key() == symbol_locator_key)
        };

        if !is_rename_target_still_present {
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
        }
    }

    pub fn synchronize_take_over_state(
        symbol_explorer_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer synchronize take over state") else {
            return;
        };

        let should_clear_take_over_state = match symbol_explorer_view_data.take_over_state.as_ref() {
            Some(SymbolExplorerTakeOverState::DeleteConfirmation { symbol_locator_key, .. }) => !project_symbol_catalog
                .get_symbol_claims()
                .iter()
                .any(|symbol_claim| symbol_claim.get_symbol_locator_key() == *symbol_locator_key),
            None => false,
        };

        if should_clear_take_over_state {
            symbol_explorer_view_data.take_over_state = None;
        }
    }

    fn selection_exists(
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_entry: &SymbolExplorerSelection,
    ) -> bool {
        match selected_entry {
            SymbolExplorerSelection::ModuleRoot(module_name) => project_symbol_catalog.find_symbol_module(module_name).is_some(),
            SymbolExplorerSelection::SymbolClaim(symbol_locator_key) => project_symbol_catalog
                .get_symbol_claims()
                .iter()
                .any(|symbol_claim| symbol_claim.get_symbol_locator_key() == *symbol_locator_key),
            SymbolExplorerSelection::DerivedNode(_) => true,
            SymbolExplorerSelection::CreateModuleRoot => true,
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
            .iter()
            .find(|symbol_tree_entry| !matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::ArrayPreviewTruncation { .. }))
            .map(|symbol_tree_entry| match symbol_tree_entry.get_kind() {
                SymbolTreeEntryKind::ModuleSpace { module_name, .. } => SymbolExplorerSelection::ModuleRoot(module_name.to_string()),
                SymbolTreeEntryKind::SymbolClaim { symbol_locator_key } => SymbolExplorerSelection::SymbolClaim(symbol_locator_key.to_string()),
                SymbolTreeEntryKind::StructField
                | SymbolTreeEntryKind::U8Segment { .. }
                | SymbolTreeEntryKind::ArrayElement
                | SymbolTreeEntryKind::PointerTarget => SymbolExplorerSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string()),
                SymbolTreeEntryKind::ArrayPreviewTruncation { .. } => unreachable!(),
            });
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
        projects::{project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_module::ProjectSymbolModule},
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    };

    fn create_dependency() -> Dependency<SymbolExplorerViewData> {
        let dependency_container = DependencyContainer::new();

        dependency_container.register(SymbolExplorerViewData::new())
    }

    #[test]
    fn synchronize_selection_prefers_first_symbol_claim() {
        let symbol_explorer_view_data = create_dependency();
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
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
        );

        SymbolExplorerViewData::synchronize_selection(symbol_explorer_view_data.clone(), &project_symbol_catalog, false);

        let selected_entry = symbol_explorer_view_data
            .read("Symbol explorer synchronize selection test")
            .and_then(|symbol_explorer_view_data| symbol_explorer_view_data.get_selected_entry().cloned());

        assert_eq!(selected_entry, Some(SymbolExplorerSelection::SymbolClaim(String::from("absolute:1234"))));
    }

    #[test]
    fn synchronize_selection_prefers_first_module_root() {
        let symbol_explorer_view_data = create_dependency();
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![ProjectSymbolModule::new(String::from("game.exe"), 0x2000)],
            Vec::new(),
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("player.stats"),
            )],
        );

        SymbolExplorerViewData::synchronize_selection(symbol_explorer_view_data.clone(), &project_symbol_catalog, false);

        let selected_entry = symbol_explorer_view_data
            .read("Symbol explorer synchronize module selection test")
            .and_then(|symbol_explorer_view_data| symbol_explorer_view_data.get_selected_entry().cloned());

        assert_eq!(selected_entry, Some(SymbolExplorerSelection::ModuleRoot(String::from("game.exe"))));
    }

    #[test]
    fn synchronize_selection_ignores_struct_layouts_without_symbol_claims() {
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

        SymbolExplorerViewData::synchronize_selection(symbol_explorer_view_data.clone(), &project_symbol_catalog, false);

        let selected_entry = symbol_explorer_view_data
            .read("Symbol explorer struct layout synchronize selection test")
            .and_then(|symbol_explorer_view_data| symbol_explorer_view_data.get_selected_entry().cloned());

        assert_eq!(selected_entry, None);
    }

    #[test]
    fn begin_create_module_root_selects_create_module_state() {
        let symbol_explorer_view_data = create_dependency();

        SymbolExplorerViewData::begin_create_module_root(symbol_explorer_view_data.clone());

        let symbol_explorer_view_data = symbol_explorer_view_data
            .read("Symbol explorer begin create module root test")
            .expect("Expected symbol explorer dependency read access in test.");

        assert_eq!(symbol_explorer_view_data.get_selected_entry(), Some(&SymbolExplorerSelection::CreateModuleRoot));
    }

    #[test]
    fn set_selected_entry_clears_inline_rename_state() {
        let symbol_explorer_view_data = create_dependency();

        SymbolExplorerViewData::begin_inline_rename(symbol_explorer_view_data.clone(), String::from("absolute:1234"));
        SymbolExplorerViewData::set_selected_entry(
            symbol_explorer_view_data.clone(),
            Some(SymbolExplorerSelection::SymbolClaim(String::from("absolute:1234"))),
        );

        let symbol_explorer_view_data = symbol_explorer_view_data
            .read("Symbol explorer inline rename clear test")
            .expect("Expected symbol explorer dependency read access in test.");

        assert_eq!(symbol_explorer_view_data.get_inline_rename_tree_node_key(), None);
    }

    #[test]
    fn synchronize_selection_can_leave_selection_empty_when_default_selection_is_suppressed() {
        let symbol_explorer_view_data = create_dependency();
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            vec![],
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("player.stats"),
            )],
        );

        SymbolExplorerViewData::synchronize_selection(symbol_explorer_view_data.clone(), &project_symbol_catalog, true);

        let selected_entry = symbol_explorer_view_data
            .read("Symbol explorer suppressed synchronize selection test")
            .and_then(|symbol_explorer_view_data| symbol_explorer_view_data.get_selected_entry().cloned());

        assert_eq!(selected_entry, None);
    }

    #[test]
    fn synchronize_inline_rename_clears_missing_symbol() {
        let symbol_explorer_view_data = create_dependency();
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            vec![],
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("player.stats"),
            )],
        );

        SymbolExplorerViewData::begin_inline_rename(symbol_explorer_view_data.clone(), String::from("absolute:9999"));
        SymbolExplorerViewData::synchronize_inline_rename(symbol_explorer_view_data.clone(), &project_symbol_catalog);

        let symbol_explorer_view_data = symbol_explorer_view_data
            .read("Symbol explorer synchronize inline rename test")
            .expect("Expected symbol explorer dependency read access in test.");

        assert_eq!(symbol_explorer_view_data.get_inline_rename_tree_node_key(), None);
    }
}
