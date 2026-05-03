use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use crate::views::symbol_explorer::view_data::symbol_tree_entry::{SymbolTreeEntry, SymbolTreeEntryKind};
use epaint::Pos2;
use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteModuleRangeMode;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::projects::project_symbol_locator::ProjectSymbolLocator;
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
    DeleteSymbolClaimConfirmation {
        symbol_locator_key: String,
        display_name: String,
    },
    DeleteModuleRootConfirmation {
        module_name: String,
    },
    DeleteModuleRangeConfirmation {
        module_name: String,
        offset: u64,
        length: u64,
        display_name: String,
        mode: ProjectSymbolsDeleteModuleRangeMode,
    },
    DefineFieldFromU8Segment {
        module_name: String,
        offset: u64,
        length: u64,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModuleRootCreateDraft {
    pub module_name: String,
    pub size_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefineFieldDraft {
    pub display_name: String,
    pub relative_offset_text: String,
    pub data_type_selection: DataTypeSelection,
}

impl Default for DefineFieldDraft {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            relative_offset_text: String::from("0x0"),
            data_type_selection: DataTypeSelection::new(DataTypeRef::new("i32")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SymbolExplorerContextMenuTarget {
    tree_node_key: String,
    position: Pos2,
}

impl SymbolExplorerContextMenuTarget {
    pub fn new(
        tree_node_key: String,
        position: Pos2,
    ) -> Self {
        Self { tree_node_key, position }
    }

    pub fn get_tree_node_key(&self) -> &str {
        &self.tree_node_key
    }

    pub fn get_position(&self) -> Pos2 {
        self.position
    }
}

#[derive(Clone, Default)]
pub struct SymbolExplorerViewData {
    selected_entry: Option<SymbolExplorerSelection>,
    take_over_state: Option<SymbolExplorerTakeOverState>,
    module_root_create_draft: ModuleRootCreateDraft,
    define_field_draft: DefineFieldDraft,
    inline_rename_tree_node_key: Option<String>,
    expanded_tree_node_keys: HashSet<String>,
    context_menu_target: Option<SymbolExplorerContextMenuTarget>,
}

impl SymbolExplorerViewData {
    pub fn new() -> Self {
        Self {
            selected_entry: None,
            take_over_state: None,
            module_root_create_draft: ModuleRootCreateDraft::default(),
            define_field_draft: DefineFieldDraft::default(),
            inline_rename_tree_node_key: None,
            expanded_tree_node_keys: HashSet::new(),
            context_menu_target: None,
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

    pub fn get_define_field_draft(&self) -> &DefineFieldDraft {
        &self.define_field_draft
    }

    pub fn get_inline_rename_tree_node_key(&self) -> Option<&str> {
        self.inline_rename_tree_node_key.as_deref()
    }

    pub fn get_expanded_tree_node_keys(&self) -> &HashSet<String> {
        &self.expanded_tree_node_keys
    }

    pub fn get_context_menu_target(&self) -> Option<&SymbolExplorerContextMenuTarget> {
        self.context_menu_target.as_ref()
    }

    pub fn set_selected_entry(
        symbol_explorer_view_data: Dependency<Self>,
        selected_entry: Option<SymbolExplorerSelection>,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer set selected entry") {
            symbol_explorer_view_data.selected_entry = selected_entry;
            symbol_explorer_view_data.take_over_state = None;
            symbol_explorer_view_data.module_root_create_draft = ModuleRootCreateDraft::default();
            symbol_explorer_view_data.define_field_draft = DefineFieldDraft::default();
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
            symbol_explorer_view_data.context_menu_target = None;
        }
    }

    pub fn begin_create_module_root(symbol_explorer_view_data: Dependency<Self>) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer begin create module root") {
            symbol_explorer_view_data.selected_entry = Some(SymbolExplorerSelection::CreateModuleRoot);
            symbol_explorer_view_data.take_over_state = None;
            symbol_explorer_view_data.module_root_create_draft = ModuleRootCreateDraft::default();
            symbol_explorer_view_data.define_field_draft = DefineFieldDraft::default();
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
            symbol_explorer_view_data.context_menu_target = None;
        }
    }

    pub fn begin_inline_rename(
        symbol_explorer_view_data: Dependency<Self>,
        tree_node_key: String,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer begin inline rename") {
            symbol_explorer_view_data.inline_rename_tree_node_key = Some(tree_node_key);
            symbol_explorer_view_data.take_over_state = None;
            symbol_explorer_view_data.context_menu_target = None;
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

    pub fn set_define_field_draft(
        symbol_explorer_view_data: Dependency<Self>,
        define_field_draft: DefineFieldDraft,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer set define field draft") {
            symbol_explorer_view_data.define_field_draft = define_field_draft;
        }
    }

    pub fn request_delete_symbol_claim_confirmation(
        symbol_explorer_view_data: Dependency<Self>,
        symbol_locator_key: String,
        display_name: String,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer request delete confirmation") {
            symbol_explorer_view_data.take_over_state = Some(SymbolExplorerTakeOverState::DeleteSymbolClaimConfirmation {
                symbol_locator_key,
                display_name,
            });
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
            symbol_explorer_view_data.context_menu_target = None;
        }
    }

    pub fn request_delete_module_root_confirmation(
        symbol_explorer_view_data: Dependency<Self>,
        module_name: String,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer request delete module confirmation") {
            symbol_explorer_view_data.take_over_state = Some(SymbolExplorerTakeOverState::DeleteModuleRootConfirmation { module_name });
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
            symbol_explorer_view_data.context_menu_target = None;
        }
    }

    pub fn request_delete_module_range_confirmation(
        symbol_explorer_view_data: Dependency<Self>,
        module_name: String,
        offset: u64,
        length: u64,
        display_name: String,
        mode: ProjectSymbolsDeleteModuleRangeMode,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer request delete module range confirmation") {
            symbol_explorer_view_data.take_over_state = Some(SymbolExplorerTakeOverState::DeleteModuleRangeConfirmation {
                module_name,
                offset,
                length,
                display_name,
                mode,
            });
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
            symbol_explorer_view_data.context_menu_target = None;
        }
    }

    pub fn cancel_take_over_state(symbol_explorer_view_data: Dependency<Self>) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer cancel take over state") {
            symbol_explorer_view_data.take_over_state = None;
            symbol_explorer_view_data.define_field_draft = DefineFieldDraft::default();
        }
    }

    pub fn begin_define_field_from_u8_segment(
        symbol_explorer_view_data: Dependency<Self>,
        module_name: String,
        offset: u64,
        length: u64,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer begin define field from u8 segment") {
            symbol_explorer_view_data.take_over_state = Some(SymbolExplorerTakeOverState::DefineFieldFromU8Segment { module_name, offset, length });
            symbol_explorer_view_data.define_field_draft = DefineFieldDraft {
                display_name: format!("field_{:08X}", offset),
                ..Default::default()
            };
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
            symbol_explorer_view_data.context_menu_target = None;
        }
    }

    pub fn show_context_menu(
        symbol_explorer_view_data: Dependency<Self>,
        target: SymbolExplorerContextMenuTarget,
    ) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer show context menu") {
            symbol_explorer_view_data.context_menu_target = Some(target);
            symbol_explorer_view_data.take_over_state = None;
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
        }
    }

    pub fn hide_context_menu(symbol_explorer_view_data: Dependency<Self>) {
        if let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer hide context menu") {
            symbol_explorer_view_data.context_menu_target = None;
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
                symbol_explorer_view_data.define_field_draft = DefineFieldDraft::default();
                symbol_explorer_view_data.inline_rename_tree_node_key = None;
                symbol_explorer_view_data.context_menu_target = None;
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
            symbol_explorer_view_data.define_field_draft = DefineFieldDraft::default();
            symbol_explorer_view_data.inline_rename_tree_node_key = None;
            symbol_explorer_view_data.context_menu_target = None;
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

            Self::symbol_locator_exists(project_symbol_catalog, symbol_locator_key)
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
            Some(SymbolExplorerTakeOverState::DeleteSymbolClaimConfirmation { symbol_locator_key, .. }) => {
                !Self::symbol_locator_exists(project_symbol_catalog, symbol_locator_key)
            }
            Some(SymbolExplorerTakeOverState::DeleteModuleRootConfirmation { module_name }) => project_symbol_catalog.find_symbol_module(module_name).is_none(),
            Some(SymbolExplorerTakeOverState::DeleteModuleRangeConfirmation { module_name, .. }) => {
                project_symbol_catalog.find_symbol_module(module_name).is_none()
            }
            Some(SymbolExplorerTakeOverState::DefineFieldFromU8Segment { module_name, .. }) => project_symbol_catalog.find_symbol_module(module_name).is_none(),
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
            SymbolExplorerSelection::SymbolClaim(symbol_locator_key) => Self::symbol_locator_exists(project_symbol_catalog, symbol_locator_key),
            SymbolExplorerSelection::DerivedNode(_) => true,
            SymbolExplorerSelection::CreateModuleRoot => true,
        }
    }

    fn symbol_locator_exists(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_locator_key: &str,
    ) -> bool {
        project_symbol_catalog
            .find_symbol_claim(symbol_locator_key)
            .is_some()
            || project_symbol_catalog
                .find_module_field(symbol_locator_key)
                .is_some()
    }

    pub fn synchronize_selection_to_tree_entries(
        symbol_explorer_view_data: Dependency<Self>,
        symbol_tree_entries: &[SymbolTreeEntry],
    ) {
        let Some(mut symbol_explorer_view_data) = symbol_explorer_view_data.write("Symbol explorer synchronize selection to tree entries") else {
            return;
        };

        if let Some(context_menu_target) = symbol_explorer_view_data.context_menu_target.as_ref() {
            if !symbol_tree_entries
                .iter()
                .any(|symbol_tree_entry| symbol_tree_entry.get_node_key() == context_menu_target.get_tree_node_key())
            {
                symbol_explorer_view_data.context_menu_target = None;
            }
        }

        if let Some(SymbolExplorerTakeOverState::DefineFieldFromU8Segment { module_name, offset, length }) = symbol_explorer_view_data.take_over_state.as_ref()
        {
            let is_target_segment_still_available = symbol_tree_entries
                .iter()
                .any(|symbol_tree_entry| match symbol_tree_entry.get_kind() {
                    SymbolTreeEntryKind::U8Segment {
                        module_name: segment_module_name,
                        offset: segment_offset,
                        length: segment_length,
                    } => segment_module_name == module_name && segment_offset == offset && segment_length == length,
                    SymbolTreeEntryKind::SymbolClaim { .. } => {
                        let ProjectSymbolLocator::ModuleOffset {
                            module_name: claim_module_name,
                            offset: claim_offset,
                        } = symbol_tree_entry.get_locator()
                        else {
                            return false;
                        };

                        claim_module_name == module_name
                            && claim_offset == offset
                            && symbol_tree_entry.get_symbol_type_id() == "u8"
                            && symbol_tree_entry.get_container_type() == ContainerType::ArrayFixed(*length)
                    }
                    _ => false,
                });

            if !is_target_segment_still_available {
                symbol_explorer_view_data.take_over_state = None;
                symbol_explorer_view_data.define_field_draft = DefineFieldDraft::default();
            }
        }

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
            .next()
            .map(|symbol_tree_entry| match symbol_tree_entry.get_kind() {
                SymbolTreeEntryKind::ModuleSpace { module_name, .. } => SymbolExplorerSelection::ModuleRoot(module_name.to_string()),
                SymbolTreeEntryKind::SymbolClaim { symbol_locator_key } => SymbolExplorerSelection::SymbolClaim(symbol_locator_key.to_string()),
                SymbolTreeEntryKind::StructField | SymbolTreeEntryKind::U8Segment { .. } | SymbolTreeEntryKind::PointerTarget => {
                    SymbolExplorerSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string())
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::{DefineFieldDraft, SymbolExplorerContextMenuTarget, SymbolExplorerSelection, SymbolExplorerTakeOverState, SymbolExplorerViewData};
    use crate::views::symbol_explorer::view_data::symbol_tree_entry::{SymbolTreeEntry, SymbolTreeEntryKind, build_symbol_tree_entries};
    use epaint::pos2;
    use squalr_engine_api::dependency_injection::dependency::Dependency;
    use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        projects::{
            project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator,
            project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField,
        },
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    };
    use std::collections::{HashMap, HashSet};

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
    fn synchronize_selection_keeps_module_field_selection() {
        let symbol_explorer_view_data = create_dependency();
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x2000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Tail"), 0x1000, String::from("u8[128]")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());

        SymbolExplorerViewData::set_selected_entry(
            symbol_explorer_view_data.clone(),
            Some(SymbolExplorerSelection::SymbolClaim(String::from("module:game.exe:1000"))),
        );
        SymbolExplorerViewData::synchronize_selection(symbol_explorer_view_data.clone(), &project_symbol_catalog, false);

        let selected_entry = symbol_explorer_view_data
            .read("Symbol explorer module field selection test")
            .and_then(|symbol_explorer_view_data| symbol_explorer_view_data.get_selected_entry().cloned());

        assert_eq!(selected_entry, Some(SymbolExplorerSelection::SymbolClaim(String::from("module:game.exe:1000"))));
    }

    #[test]
    fn synchronize_selection_keeps_tail_split_module_field_visible_in_tree() {
        let symbol_explorer_view_data = create_dependency();
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x10);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("u8_00000000"), 0x00, String::from("u8[8]")));
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("u8_00000008"), 0x08, String::from("u8[4]")));
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("u8_0000000C"), 0x0C, String::from("u8[4]")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let mut expanded_tree_node_keys = HashSet::new();
        expanded_tree_node_keys.insert(String::from("module:game.exe"));
        let symbol_tree_entries = build_symbol_tree_entries(&project_symbol_catalog, &expanded_tree_node_keys, &HashMap::new(), |data_type_ref| {
            (data_type_ref.get_data_type_id() == "u8").then_some(1)
        });

        SymbolExplorerViewData::set_selected_entry(
            symbol_explorer_view_data.clone(),
            Some(SymbolExplorerSelection::SymbolClaim(String::from("module:game.exe:C"))),
        );
        SymbolExplorerViewData::synchronize_selection(symbol_explorer_view_data.clone(), &project_symbol_catalog, false);
        SymbolExplorerViewData::synchronize_selection_to_tree_entries(symbol_explorer_view_data.clone(), &symbol_tree_entries);

        let selected_entry = symbol_explorer_view_data
            .read("Symbol explorer tail split module field selection test")
            .and_then(|symbol_explorer_view_data| symbol_explorer_view_data.get_selected_entry().cloned());

        assert_eq!(selected_entry, Some(SymbolExplorerSelection::SymbolClaim(String::from("module:game.exe:C"))));
        assert!(
            symbol_tree_entries
                .iter()
                .any(|symbol_tree_entry| symbol_tree_entry.get_node_key() == "claim:module:game.exe:C")
        );
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
    fn show_context_menu_tracks_tree_node_and_position() {
        let symbol_explorer_view_data = create_dependency();
        let context_menu_position = pos2(12.0, 34.0);

        SymbolExplorerViewData::show_context_menu(
            symbol_explorer_view_data.clone(),
            SymbolExplorerContextMenuTarget::new(String::from("claim:absolute:1234"), context_menu_position),
        );

        let context_menu_target = symbol_explorer_view_data
            .read("Symbol explorer context menu target test")
            .and_then(|symbol_explorer_view_data| symbol_explorer_view_data.get_context_menu_target().cloned());

        assert_eq!(
            context_menu_target,
            Some(SymbolExplorerContextMenuTarget::new(String::from("claim:absolute:1234"), context_menu_position))
        );
    }

    #[test]
    fn begin_inline_rename_clears_context_menu() {
        let symbol_explorer_view_data = create_dependency();

        SymbolExplorerViewData::show_context_menu(
            symbol_explorer_view_data.clone(),
            SymbolExplorerContextMenuTarget::new(String::from("claim:absolute:1234"), pos2(12.0, 34.0)),
        );
        SymbolExplorerViewData::begin_inline_rename(symbol_explorer_view_data.clone(), String::from("claim:absolute:1234"));

        let context_menu_target = symbol_explorer_view_data
            .read("Symbol explorer context menu clear test")
            .and_then(|symbol_explorer_view_data| symbol_explorer_view_data.get_context_menu_target().cloned());

        assert_eq!(context_menu_target, None);
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

    #[test]
    fn synchronize_inline_rename_keeps_module_field_target() {
        let symbol_explorer_view_data = create_dependency();
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x2000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Tail"), 0x1000, String::from("u8[128]")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());

        SymbolExplorerViewData::begin_inline_rename(symbol_explorer_view_data.clone(), String::from("claim:module:game.exe:1000"));
        SymbolExplorerViewData::synchronize_inline_rename(symbol_explorer_view_data.clone(), &project_symbol_catalog);

        let inline_rename_tree_node_key = symbol_explorer_view_data
            .read("Symbol explorer module field inline rename test")
            .and_then(|symbol_explorer_view_data| {
                symbol_explorer_view_data
                    .get_inline_rename_tree_node_key()
                    .map(str::to_string)
            });

        assert_eq!(inline_rename_tree_node_key, Some(String::from("claim:module:game.exe:1000")));
    }

    #[test]
    fn synchronize_take_over_state_clears_missing_module_delete_confirmation() {
        let symbol_explorer_view_data = create_dependency();
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![ProjectSymbolModule::new(String::from("engine.dll"), 0x4000)],
            Vec::new(),
            Vec::new(),
        );

        SymbolExplorerViewData::request_delete_module_root_confirmation(symbol_explorer_view_data.clone(), String::from("game.exe"));
        SymbolExplorerViewData::synchronize_take_over_state(symbol_explorer_view_data.clone(), &project_symbol_catalog);

        let symbol_explorer_view_data = symbol_explorer_view_data
            .read("Symbol explorer synchronize module delete takeover test")
            .expect("Expected symbol explorer dependency read access in test.");

        assert_eq!(symbol_explorer_view_data.get_take_over_state(), None);
    }

    #[test]
    fn request_delete_module_root_confirmation_tracks_module_name() {
        let symbol_explorer_view_data = create_dependency();

        SymbolExplorerViewData::request_delete_module_root_confirmation(symbol_explorer_view_data.clone(), String::from("game.exe"));

        let take_over_state = symbol_explorer_view_data
            .read("Symbol explorer request module delete confirmation test")
            .and_then(|symbol_explorer_view_data| symbol_explorer_view_data.get_take_over_state().cloned());

        assert_eq!(
            take_over_state,
            Some(SymbolExplorerTakeOverState::DeleteModuleRootConfirmation {
                module_name: String::from("game.exe"),
            })
        );
    }

    #[test]
    fn begin_define_field_from_u8_segment_initializes_takeover_and_draft() {
        let symbol_explorer_view_data = create_dependency();

        SymbolExplorerViewData::begin_define_field_from_u8_segment(symbol_explorer_view_data.clone(), String::from("game.exe"), 0x40, 0x100);

        let symbol_explorer_view_data = symbol_explorer_view_data
            .read("Symbol explorer begin define field test")
            .expect("Expected symbol explorer dependency read access in test.");

        assert_eq!(
            symbol_explorer_view_data.get_take_over_state(),
            Some(&SymbolExplorerTakeOverState::DefineFieldFromU8Segment {
                module_name: String::from("game.exe"),
                offset: 0x40,
                length: 0x100,
            })
        );
        assert_eq!(
            symbol_explorer_view_data.get_define_field_draft(),
            &DefineFieldDraft {
                display_name: String::from("field_00000040"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn synchronize_selection_to_tree_entries_clears_missing_define_field_target() {
        let symbol_explorer_view_data = create_dependency();

        SymbolExplorerViewData::begin_define_field_from_u8_segment(symbol_explorer_view_data.clone(), String::from("game.exe"), 0x40, 0x100);
        SymbolExplorerViewData::synchronize_selection_to_tree_entries(symbol_explorer_view_data.clone(), &[]);

        let take_over_state = symbol_explorer_view_data
            .read("Symbol explorer missing define field target test")
            .and_then(|symbol_explorer_view_data| symbol_explorer_view_data.get_take_over_state().cloned());

        assert_eq!(take_over_state, None);
    }

    #[test]
    fn synchronize_selection_to_tree_entries_keeps_module_u8_field_define_field_target() {
        let symbol_explorer_view_data = create_dependency();
        let symbol_tree_entries = vec![SymbolTreeEntry::new(
            String::from("claim:module:game.exe:40"),
            SymbolTreeEntryKind::SymbolClaim {
                symbol_locator_key: String::from("module:game.exe:40"),
            },
            1,
            String::from("u8_00000040"),
            String::from("game.exe.u8_00000040"),
            String::from("module:game.exe:40"),
            ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x40),
            String::from("u8"),
            ContainerType::ArrayFixed(0x100),
            false,
            false,
        )];

        SymbolExplorerViewData::begin_define_field_from_u8_segment(symbol_explorer_view_data.clone(), String::from("game.exe"), 0x40, 0x100);
        SymbolExplorerViewData::synchronize_selection_to_tree_entries(symbol_explorer_view_data.clone(), &symbol_tree_entries);

        let take_over_state = symbol_explorer_view_data
            .read("Symbol explorer module u8 field define target test")
            .and_then(|symbol_explorer_view_data| symbol_explorer_view_data.get_take_over_state().cloned());

        assert_eq!(
            take_over_state,
            Some(SymbolExplorerTakeOverState::DefineFieldFromU8Segment {
                module_name: String::from("game.exe"),
                offset: 0x40,
                length: 0x100,
            })
        );
    }
}
