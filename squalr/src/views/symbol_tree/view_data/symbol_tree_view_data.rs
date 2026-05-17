use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use epaint::Pos2;
use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteModuleRangeMode;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::{anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType};
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::projects::symbol_tree::symbol_tree_node::{SymbolTreeNode, SymbolTreeNodeKind};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolTreeSelection {
    ModuleRoot(String),
    SymbolClaim(String),
    DerivedNode(String),
    CreateModuleRoot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolTreeTakeOverState {
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
    DefineFieldFromUnassignedSegment {
        module_name: String,
        offset: u64,
        length: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleRootCreateDraft {
    pub module_name: String,
    pub size_text: String,
    pub size_format: AnonymousValueStringFormat,
}

impl Default for ModuleRootCreateDraft {
    fn default() -> Self {
        Self {
            module_name: String::new(),
            size_text: String::from("1000"),
            size_format: AnonymousValueStringFormat::Hexadecimal,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefineFieldDraft {
    pub display_name: String,
    pub relative_offset_text: String,
    pub relative_offset_format: AnonymousValueStringFormat,
    pub container_type: ContainerType,
    pub data_type_selection: DataTypeSelection,
}

impl Default for DefineFieldDraft {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            relative_offset_text: String::from("0"),
            relative_offset_format: AnonymousValueStringFormat::Decimal,
            container_type: ContainerType::None,
            data_type_selection: DataTypeSelection::new(DataTypeRef::new("i32")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SymbolTreeContextMenuTarget {
    tree_node_key: String,
    position: Pos2,
}

impl SymbolTreeContextMenuTarget {
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
pub struct SymbolTreeViewData {
    selected_entry: Option<SymbolTreeSelection>,
    take_over_state: Option<SymbolTreeTakeOverState>,
    module_root_create_draft: ModuleRootCreateDraft,
    define_field_draft: DefineFieldDraft,
    inline_rename_tree_node_key: Option<String>,
    expanded_tree_node_keys: HashSet<String>,
    context_menu_target: Option<SymbolTreeContextMenuTarget>,
}

impl SymbolTreeViewData {
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

    pub fn get_selected_entry(&self) -> Option<&SymbolTreeSelection> {
        self.selected_entry.as_ref()
    }

    fn is_module_field_node_key(tree_node_key: &str) -> bool {
        tree_node_key.starts_with("module_field:")
    }

    pub fn get_take_over_state(&self) -> Option<&SymbolTreeTakeOverState> {
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

    pub fn get_context_menu_target(&self) -> Option<&SymbolTreeContextMenuTarget> {
        self.context_menu_target.as_ref()
    }

    pub fn set_selected_entry(
        symbol_tree_view_data: Dependency<Self>,
        selected_entry: Option<SymbolTreeSelection>,
    ) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree set selected entry") {
            symbol_tree_view_data.selected_entry = selected_entry;
            symbol_tree_view_data.take_over_state = None;
            symbol_tree_view_data.module_root_create_draft = ModuleRootCreateDraft::default();
            symbol_tree_view_data.define_field_draft = DefineFieldDraft::default();
            symbol_tree_view_data.inline_rename_tree_node_key = None;
            symbol_tree_view_data.context_menu_target = None;
        }
    }

    pub fn begin_create_module_root(symbol_tree_view_data: Dependency<Self>) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree begin create module root") {
            symbol_tree_view_data.selected_entry = Some(SymbolTreeSelection::CreateModuleRoot);
            symbol_tree_view_data.take_over_state = None;
            symbol_tree_view_data.module_root_create_draft = ModuleRootCreateDraft::default();
            symbol_tree_view_data.define_field_draft = DefineFieldDraft::default();
            symbol_tree_view_data.inline_rename_tree_node_key = None;
            symbol_tree_view_data.context_menu_target = None;
        }
    }

    pub fn begin_inline_rename(
        symbol_tree_view_data: Dependency<Self>,
        tree_node_key: String,
    ) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree begin inline rename") {
            symbol_tree_view_data.inline_rename_tree_node_key = Some(tree_node_key);
            symbol_tree_view_data.take_over_state = None;
            symbol_tree_view_data.context_menu_target = None;
        }
    }

    pub fn cancel_inline_rename(symbol_tree_view_data: Dependency<Self>) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree cancel inline rename") {
            symbol_tree_view_data.inline_rename_tree_node_key = None;
        }
    }

    pub fn set_module_root_create_draft(
        symbol_tree_view_data: Dependency<Self>,
        module_root_create_draft: ModuleRootCreateDraft,
    ) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree set module root create draft") {
            symbol_tree_view_data.module_root_create_draft = module_root_create_draft;
        }
    }

    pub fn set_define_field_draft(
        symbol_tree_view_data: Dependency<Self>,
        define_field_draft: DefineFieldDraft,
    ) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree set define field draft") {
            symbol_tree_view_data.define_field_draft = define_field_draft;
        }
    }

    pub fn request_delete_symbol_claim_confirmation(
        symbol_tree_view_data: Dependency<Self>,
        symbol_locator_key: String,
        display_name: String,
    ) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree request delete confirmation") {
            symbol_tree_view_data.take_over_state = Some(SymbolTreeTakeOverState::DeleteSymbolClaimConfirmation {
                symbol_locator_key,
                display_name,
            });
            symbol_tree_view_data.inline_rename_tree_node_key = None;
            symbol_tree_view_data.context_menu_target = None;
        }
    }

    pub fn request_delete_module_root_confirmation(
        symbol_tree_view_data: Dependency<Self>,
        module_name: String,
    ) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree request delete module confirmation") {
            symbol_tree_view_data.take_over_state = Some(SymbolTreeTakeOverState::DeleteModuleRootConfirmation { module_name });
            symbol_tree_view_data.inline_rename_tree_node_key = None;
            symbol_tree_view_data.context_menu_target = None;
        }
    }

    pub fn request_delete_module_range_confirmation(
        symbol_tree_view_data: Dependency<Self>,
        module_name: String,
        offset: u64,
        length: u64,
        display_name: String,
        mode: ProjectSymbolsDeleteModuleRangeMode,
    ) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree request delete module range confirmation") {
            symbol_tree_view_data.take_over_state = Some(SymbolTreeTakeOverState::DeleteModuleRangeConfirmation {
                module_name,
                offset,
                length,
                display_name,
                mode,
            });
            symbol_tree_view_data.inline_rename_tree_node_key = None;
            symbol_tree_view_data.context_menu_target = None;
        }
    }

    pub fn cancel_take_over_state(symbol_tree_view_data: Dependency<Self>) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree cancel take over state") {
            symbol_tree_view_data.take_over_state = None;
            symbol_tree_view_data.define_field_draft = DefineFieldDraft::default();
        }
    }

    pub fn begin_define_field_from_unassigned_segment(
        symbol_tree_view_data: Dependency<Self>,
        module_name: String,
        offset: u64,
        length: u64,
    ) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree begin define field from unassigned segment") {
            symbol_tree_view_data.take_over_state = Some(SymbolTreeTakeOverState::DefineFieldFromUnassignedSegment { module_name, offset, length });
            symbol_tree_view_data.define_field_draft = DefineFieldDraft {
                display_name: format!("field_{:08X}", offset),
                ..Default::default()
            };
            symbol_tree_view_data.inline_rename_tree_node_key = None;
            symbol_tree_view_data.context_menu_target = None;
        }
    }

    pub fn show_context_menu(
        symbol_tree_view_data: Dependency<Self>,
        target: SymbolTreeContextMenuTarget,
    ) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree show context menu") {
            symbol_tree_view_data.context_menu_target = Some(target);
            symbol_tree_view_data.take_over_state = None;
            symbol_tree_view_data.inline_rename_tree_node_key = None;
        }
    }

    pub fn hide_context_menu(symbol_tree_view_data: Dependency<Self>) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree hide context menu") {
            symbol_tree_view_data.context_menu_target = None;
        }
    }

    pub fn toggle_tree_node_expansion(
        symbol_tree_view_data: Dependency<Self>,
        tree_node_key: &str,
    ) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree toggle tree node expansion") {
            if symbol_tree_view_data
                .expanded_tree_node_keys
                .contains(tree_node_key)
            {
                symbol_tree_view_data
                    .expanded_tree_node_keys
                    .remove(tree_node_key);
            } else {
                symbol_tree_view_data
                    .expanded_tree_node_keys
                    .insert(tree_node_key.to_string());
            }
        }
    }

    pub fn expand_tree_node(
        symbol_tree_view_data: Dependency<Self>,
        tree_node_key: &str,
    ) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree expand tree node") {
            symbol_tree_view_data
                .expanded_tree_node_keys
                .insert(tree_node_key.to_string());
        }
    }

    pub fn synchronize_selection(
        symbol_tree_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        suppress_default_selection: bool,
    ) {
        if let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree synchronize selection") {
            let has_valid_selection = symbol_tree_view_data
                .selected_entry
                .as_ref()
                .is_some_and(|selected_entry| Self::selection_exists(project_symbol_catalog, selected_entry));

            if has_valid_selection {
                return;
            }

            if suppress_default_selection {
                symbol_tree_view_data.selected_entry = None;
                symbol_tree_view_data.take_over_state = None;
                symbol_tree_view_data.module_root_create_draft = ModuleRootCreateDraft::default();
                symbol_tree_view_data.define_field_draft = DefineFieldDraft::default();
                symbol_tree_view_data.inline_rename_tree_node_key = None;
                symbol_tree_view_data.context_menu_target = None;
                return;
            }

            symbol_tree_view_data.selected_entry = project_symbol_catalog
                .get_symbol_modules()
                .first()
                .map(|symbol_module| SymbolTreeSelection::ModuleRoot(symbol_module.get_module_name().to_string()))
                .or_else(|| {
                    project_symbol_catalog
                        .get_symbol_claims()
                        .first()
                        .map(|symbol_claim| SymbolTreeSelection::SymbolClaim(symbol_claim.get_symbol_locator_key().to_string()))
                });

            symbol_tree_view_data.take_over_state = None;
            symbol_tree_view_data.module_root_create_draft = ModuleRootCreateDraft::default();
            symbol_tree_view_data.define_field_draft = DefineFieldDraft::default();
            symbol_tree_view_data.inline_rename_tree_node_key = None;
            symbol_tree_view_data.context_menu_target = None;
        }
    }

    pub fn synchronize_inline_rename(
        symbol_tree_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree synchronize inline rename state") else {
            return;
        };
        let Some(inline_rename_tree_node_key) = symbol_tree_view_data.inline_rename_tree_node_key.as_ref() else {
            return;
        };

        let is_rename_target_still_present = if let Some(module_name) = inline_rename_tree_node_key.strip_prefix("module:") {
            project_symbol_catalog.find_symbol_module(module_name).is_some()
        } else {
            let symbol_locator_key = inline_rename_tree_node_key
                .strip_prefix("module_field:")
                .or_else(|| inline_rename_tree_node_key.strip_prefix("claim:"))
                .unwrap_or(inline_rename_tree_node_key);

            Self::symbol_locator_exists(project_symbol_catalog, symbol_locator_key)
        };

        if !is_rename_target_still_present {
            symbol_tree_view_data.inline_rename_tree_node_key = None;
        }
    }

    pub fn synchronize_take_over_state(
        symbol_tree_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree synchronize take over state") else {
            return;
        };

        let should_clear_take_over_state = match symbol_tree_view_data.take_over_state.as_ref() {
            Some(SymbolTreeTakeOverState::DeleteSymbolClaimConfirmation { symbol_locator_key, .. }) => {
                !Self::symbol_locator_exists(project_symbol_catalog, symbol_locator_key)
            }
            Some(SymbolTreeTakeOverState::DeleteModuleRootConfirmation { module_name }) => project_symbol_catalog.find_symbol_module(module_name).is_none(),
            Some(SymbolTreeTakeOverState::DeleteModuleRangeConfirmation { module_name, .. }) => {
                project_symbol_catalog.find_symbol_module(module_name).is_none()
            }
            Some(SymbolTreeTakeOverState::DefineFieldFromUnassignedSegment { module_name, .. }) => {
                project_symbol_catalog.find_symbol_module(module_name).is_none()
            }
            None => false,
        };

        if should_clear_take_over_state {
            symbol_tree_view_data.take_over_state = None;
        }
    }

    fn selection_exists(
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_entry: &SymbolTreeSelection,
    ) -> bool {
        match selected_entry {
            SymbolTreeSelection::ModuleRoot(module_name) => project_symbol_catalog.find_symbol_module(module_name).is_some(),
            SymbolTreeSelection::SymbolClaim(symbol_locator_key) => Self::symbol_locator_exists(project_symbol_catalog, symbol_locator_key),
            SymbolTreeSelection::DerivedNode(_) => true,
            SymbolTreeSelection::CreateModuleRoot => true,
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
        symbol_tree_view_data: Dependency<Self>,
        symbol_tree_entries: &[SymbolTreeNode],
    ) {
        let Some(mut symbol_tree_view_data) = symbol_tree_view_data.write("Symbol tree synchronize selection to tree entries") else {
            return;
        };

        if let Some(context_menu_target) = symbol_tree_view_data.context_menu_target.as_ref() {
            if !symbol_tree_entries
                .iter()
                .any(|symbol_tree_entry| symbol_tree_entry.get_node_key() == context_menu_target.get_tree_node_key())
            {
                symbol_tree_view_data.context_menu_target = None;
            }
        }

        if let Some(SymbolTreeTakeOverState::DefineFieldFromUnassignedSegment { module_name, offset, length }) = symbol_tree_view_data.take_over_state.as_ref()
        {
            let is_target_segment_still_available = symbol_tree_entries
                .iter()
                .any(|symbol_tree_entry| match symbol_tree_entry.get_kind() {
                    SymbolTreeNodeKind::UnassignedSegment {
                        module_name: segment_module_name,
                        offset: segment_offset,
                        length: segment_length,
                    } => segment_module_name == module_name && segment_offset == offset && segment_length == length,
                    _ => false,
                });

            if !is_target_segment_still_available {
                symbol_tree_view_data.take_over_state = None;
                symbol_tree_view_data.define_field_draft = DefineFieldDraft::default();
            }
        }

        if let Some(SymbolTreeSelection::SymbolClaim(selected_symbol_locator_key)) = symbol_tree_view_data.selected_entry.as_ref() {
            let has_catalog_symbol_claim_entry = symbol_tree_entries.iter().any(|symbol_tree_entry| {
                !Self::is_module_field_node_key(symbol_tree_entry.get_node_key())
                    && matches!(
                        symbol_tree_entry.get_kind(),
                        SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } if symbol_locator_key == selected_symbol_locator_key
                    )
            });

            if has_catalog_symbol_claim_entry {
                return;
            }

            if let Some(module_field_tree_entry) = symbol_tree_entries.iter().find(|symbol_tree_entry| {
                Self::is_module_field_node_key(symbol_tree_entry.get_node_key())
                    && matches!(
                        symbol_tree_entry.get_kind(),
                        SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } if symbol_locator_key == selected_symbol_locator_key
                    )
            }) {
                symbol_tree_view_data.selected_entry = Some(SymbolTreeSelection::DerivedNode(module_field_tree_entry.get_node_key().to_string()));
                return;
            }
        }

        let Some(SymbolTreeSelection::DerivedNode(selected_node_key)) = symbol_tree_view_data.selected_entry.as_ref() else {
            return;
        };

        if symbol_tree_entries
            .iter()
            .any(|symbol_tree_entry| symbol_tree_entry.get_node_key() == selected_node_key)
        {
            return;
        }

        symbol_tree_view_data.selected_entry = symbol_tree_entries
            .iter()
            .next()
            .map(|symbol_tree_entry| match symbol_tree_entry.get_kind() {
                SymbolTreeNodeKind::ModuleSpace { module_name, .. } => SymbolTreeSelection::ModuleRoot(module_name.to_string()),
                SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } => {
                    if Self::is_module_field_node_key(symbol_tree_entry.get_node_key()) {
                        SymbolTreeSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string())
                    } else {
                        SymbolTreeSelection::SymbolClaim(symbol_locator_key.to_string())
                    }
                }
                SymbolTreeNodeKind::StructField | SymbolTreeNodeKind::UnassignedSegment { .. } | SymbolTreeNodeKind::PointerTarget => {
                    SymbolTreeSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string())
                }
            });
    }
}
