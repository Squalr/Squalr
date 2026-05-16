use crate::app_context::AppContext;
use crate::ui::list_navigation::{ListNavigationDirection, resolve_next_index};
use crate::ui::widgets::controls::{
    button::Button as ThemeButton,
    context_menu::context_menu::{ContextMenu, ContextMenuSizing},
    toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
};
use crate::views::{
    code_viewer::{code_viewer_view::CodeViewerView, view_data::code_viewer_view_data::CodeViewerViewData},
    context_menu_labels::{OPEN_IN_CODE_VIEWER_LABEL, OPEN_IN_MEMORY_VIEWER_LABEL},
    memory_viewer::{memory_viewer_view::MemoryViewerView, view_data::memory_viewer_view_data::MemoryViewerViewData},
    struct_viewer::view_data::struct_viewer_focus_target::StructViewerFocusTarget,
    struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData,
    symbol_layout_editor::{symbol_layout_editor_view::SymbolLayoutEditorView, view_data::symbol_layout_editor_view_data::SymbolLayoutEditorViewData},
    symbol_tree::symbol_tree_define_field_view::{SymbolTreeDefineFieldAction, SymbolTreeDefineFieldView},
    symbol_tree::symbol_tree_delete_confirmation_view::{SymbolTreeDeleteConfirmationAction, SymbolTreeDeleteConfirmationView},
    symbol_tree::symbol_tree_entry_view::SymbolTreeEntryView,
    symbol_tree::symbol_tree_inline_rename_view::SymbolTreeInlineRenameView,
    symbol_tree::symbol_tree_module_create_view::{SymbolTreeModuleCreateAction, SymbolTreeModuleCreateView},
    symbol_tree::symbol_tree_toolbar_view::{SymbolTreeToolbarAction, SymbolTreeToolbarView},
    symbol_tree::view_data::{
        symbol_tree_scalar_value::SymbolTreeScalarValue,
        symbol_tree_view_data::{SymbolTreeContextMenuTarget, SymbolTreeSelection, SymbolTreeTakeOverState, SymbolTreeViewData},
    },
};
use eframe::egui::{Align, Color32, Direction, Id, Key, Layout, Response, RichText, ScrollArea, Ui, UiBuilder, Widget};
use epaint::pos2;
use squalr_engine_api::commands::{
    memory::{
        read::{memory_read_request::MemoryReadRequest, memory_read_response::MemoryReadResponse},
        write::memory_write_request::MemoryWriteRequest,
    },
    privileged_command_request::PrivilegedCommandRequest,
    privileged_command_response::TypedPrivilegedCommandResponse,
    project_symbols::{
        create::project_symbols_create_request::ProjectSymbolsCreateRequest,
        create_module::project_symbols_create_module_request::ProjectSymbolsCreateModuleRequest,
        delete::project_symbols_delete_request::{ProjectSymbolsDeleteModuleRange, ProjectSymbolsDeleteModuleRangeMode, ProjectSymbolsDeleteRequest},
        execute_plugin_action::project_symbols_execute_plugin_action_request::ProjectSymbolsExecutePluginActionRequest,
        rename::project_symbols_rename_request::ProjectSymbolsRenameRequest,
        rename_module::project_symbols_rename_module_request::ProjectSymbolsRenameModuleRequest,
    },
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::plugins::symbol_tree::symbol_tree_action::{SymbolTreeActionContext, SymbolTreeActionSelection};
use squalr_engine_api::structures::data_types::built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64};
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType};
use squalr_engine_api::structures::memory::{
    pointer::Pointer,
    symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink},
};
use squalr_engine_api::structures::projects::{
    project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress,
    project_symbol_catalog::ProjectSymbolCatalog,
    project_symbol_locator::ProjectSymbolLocator,
    symbol_tree::operations::{
        add_symbol_to_project::{AddSymbolToProjectTarget, build_add_symbol_project_item_create_request, build_add_symbol_to_project_target},
        define_field::DefineFieldPlan,
        delete_symbol::{ModuleChildRangeTarget, build_delete_module_range_confirmation_description, build_module_child_range_target},
        edit_symbol_layout::build_symbol_layout_edit_target,
    },
    symbol_tree::symbol_tree::SymbolTree,
    symbol_tree::symbol_tree_node::{ResolvedPointerTarget, SymbolTreeNode, SymbolTreeNodeKind, resolve_symbol_tree_node_size_in_bytes},
};
use squalr_engine_api::structures::structs::{
    symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
    symbolic_resolver_definition::SymbolicResolverEvaluationError,
    symbolic_struct_definition::SymbolicStructDefinition,
    valued_struct::ValuedStruct,
    valued_struct_field::ValuedStructField,
};
use squalr_engine_session::virtual_snapshots::virtual_snapshot_query::VirtualSnapshotQuery;
use squalr_engine_session::virtual_snapshots::virtual_snapshot_query_result::VirtualSnapshotQueryResult;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Clone)]
pub struct SymbolTreeView {
    app_context: Arc<AppContext>,
    symbol_tree_view_data: Dependency<SymbolTreeViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
    symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
    code_viewer_view_data: Dependency<CodeViewerViewData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SymbolTreePluginActionMenuItem {
    plugin_id: String,
    action_id: String,
    label: String,
}

impl SymbolTreeView {
    pub const WINDOW_ID: &'static str = "window_symbol_tree";
    const POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID: &'static str = "symbol_tree_pointer_children";
    const SCALAR_VALUES_VIRTUAL_SNAPSHOT_ID: &'static str = "symbol_tree_scalar_values";
    const PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID: &'static str = "symbol_tree_preview_values";
    const POINTER_CHILDREN_REFRESH_INTERVAL: Duration = Duration::from_millis(250);
    const SCALAR_VALUES_REFRESH_INTERVAL: Duration = Duration::from_millis(250);
    const PREVIEW_VALUES_REFRESH_INTERVAL: Duration = Duration::from_millis(250);
    const SYMBOL_TREE_TEXT_PADDING_X: f32 = 8.0;
    const STRUCT_VIEWER_SYMBOL_NAME_FIELD: &'static str = "display_name";
    const STRUCT_VIEWER_SYMBOL_SIZE_FIELD: &'static str = "size";
    const STRUCT_VIEWER_SYMBOL_PATH_FIELD: &'static str = "path";
    const INLINE_RENAME_TEXT_STORAGE_ID_PREFIX: &'static str = "symbol_tree_inline_rename_text";
    const INLINE_RENAME_HIGHLIGHT_STORAGE_ID_PREFIX: &'static str = "symbol_tree_inline_rename_highlight";
    const MAX_SYMBOL_PREVIEW_ELEMENT_COUNT: u64 = 4;
    const MAX_SYMBOL_PREVIEW_DISPLAY_ELEMENT_COUNT: usize = 3;
    const MAX_SYMBOL_PREVIEW_ARRAY_CHARACTER_COUNT: usize = 24;
    const SYMBOL_TREE_CTX_OPEN_MEMORY_VIEWER_LABEL: &str = OPEN_IN_MEMORY_VIEWER_LABEL;
    const SYMBOL_TREE_CTX_OPEN_MEMORY_VIEWER_ID: &str = "symbol_tree_ctx_open_memory_viewer";
    const SYMBOL_TREE_CTX_OPEN_CODE_VIEWER_LABEL: &str = OPEN_IN_CODE_VIEWER_LABEL;
    const SYMBOL_TREE_CTX_OPEN_CODE_VIEWER_ID: &str = "symbol_tree_ctx_open_code_viewer";
    const SYMBOL_TREE_CTX_ADD_TO_PROJECT_LABEL: &str = "Add to Project";
    const SYMBOL_TREE_CTX_ADD_TO_PROJECT_ID: &str = "symbol_tree_ctx_add_to_project";
    const SYMBOL_TREE_CTX_EDIT_SYMBOL_LAYOUT_LABEL: &str = "Edit Symbol Layout...";
    const SYMBOL_TREE_CTX_EDIT_SYMBOL_LAYOUT_ID: &str = "symbol_tree_ctx_edit_symbol_layout";
    const SYMBOL_TREE_CTX_RENAME_LABEL: &str = "Rename";
    const SYMBOL_TREE_CTX_RENAME_ID: &str = "symbol_tree_ctx_rename";
    const SYMBOL_TREE_CTX_NEW_MODULE_LABEL: &str = "New Module";
    const SYMBOL_TREE_CTX_NEW_MODULE_ID: &str = "symbol_tree_ctx_new_module";
    const SYMBOL_TREE_CTX_DELETE_LABEL: &str = "Delete";
    const SYMBOL_TREE_CTX_DELETE_ID: &str = "symbol_tree_ctx_delete";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let symbol_tree_view_data = app_context
            .dependency_container
            .register(SymbolTreeViewData::new());
        let struct_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<StructViewerViewData>();
        let symbol_layout_editor_view_data = app_context
            .dependency_container
            .get_dependency::<SymbolLayoutEditorViewData>();
        let memory_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<MemoryViewerViewData>();
        let code_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<CodeViewerViewData>();

        Self {
            app_context,
            symbol_tree_view_data,
            struct_viewer_view_data,
            symbol_layout_editor_view_data,
            memory_viewer_view_data,
            code_viewer_view_data,
        }
    }

    fn get_opened_project_symbol_catalog(&self) -> Option<ProjectSymbolCatalog> {
        let opened_project = self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project = opened_project.read().ok()?;

        opened_project.as_ref().map(|opened_project| {
            opened_project
                .get_project_info()
                .get_project_symbol_catalog()
                .clone()
        })
    }

    fn focus_memory_viewer_for_locator(
        &self,
        locator: &ProjectSymbolLocator,
    ) {
        MemoryViewerViewData::request_focus_address(
            self.memory_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            locator.get_focus_address(),
            locator.get_focus_module_name().to_string(),
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(MemoryViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(MemoryViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening memory viewer from Symbol Tree: {}", error);
            }
        }
    }

    fn focus_code_viewer_for_locator(
        &self,
        locator: &ProjectSymbolLocator,
    ) {
        CodeViewerViewData::request_focus_address(
            self.code_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            locator.get_focus_address(),
            locator.get_focus_module_name().to_string(),
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(CodeViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(CodeViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening code viewer from Symbol Tree: {}", error);
            }
        }
    }

    fn rename_symbol_claim(
        &self,
        symbol_locator_key: &str,
        display_name: String,
    ) {
        let project_symbols_rename_request = ProjectSymbolsRenameRequest {
            symbol_locator_key: symbol_locator_key.to_string(),
            display_name,
        };

        project_symbols_rename_request.send(&self.app_context.engine_unprivileged_state, |_project_symbols_rename_response| {});
    }

    fn rename_module_root(
        &self,
        module_name: &str,
        new_module_name: String,
    ) {
        let symbol_tree_view_data = self.symbol_tree_view_data.clone();
        let project_symbols_rename_module_request = ProjectSymbolsRenameModuleRequest {
            module_name: module_name.to_string(),
            new_module_name,
        };

        project_symbols_rename_module_request.send(&self.app_context.engine_unprivileged_state, move |project_symbols_rename_module_response| {
            if project_symbols_rename_module_response.success {
                let module_name = project_symbols_rename_module_response.module_name;

                SymbolTreeViewData::set_selected_entry(symbol_tree_view_data.clone(), Some(SymbolTreeSelection::ModuleRoot(module_name.clone())));
                SymbolTreeViewData::expand_tree_node(symbol_tree_view_data, &format!("module:{}", module_name));
            }
        });
    }

    fn delete_symbol_claim(
        &self,
        symbol_locator_key: &str,
    ) {
        SymbolTreeViewData::cancel_take_over_state(self.symbol_tree_view_data.clone());
        let project_symbols_delete_request = ProjectSymbolsDeleteRequest {
            symbol_locator_keys: vec![symbol_locator_key.to_string()],
            module_names: Vec::new(),
            module_ranges: Vec::new(),
        };

        project_symbols_delete_request.send(&self.app_context.engine_unprivileged_state, |project_symbols_delete_response| {
            if !project_symbols_delete_response.success {
                log::warn!("Symbol delete request failed.");
            }
        });
    }

    fn delete_module_range(
        &self,
        module_name: &str,
        offset: u64,
        length: u64,
        mode: ProjectSymbolsDeleteModuleRangeMode,
    ) {
        SymbolTreeViewData::cancel_take_over_state(self.symbol_tree_view_data.clone());
        let project_symbols_delete_request = ProjectSymbolsDeleteRequest {
            symbol_locator_keys: Vec::new(),
            module_names: Vec::new(),
            module_ranges: vec![ProjectSymbolsDeleteModuleRange {
                module_name: module_name.to_string(),
                offset,
                length,
                mode,
            }],
        };

        project_symbols_delete_request.send(&self.app_context.engine_unprivileged_state, |project_symbols_delete_response| {
            if !project_symbols_delete_response.success {
                log::warn!("Module range delete request failed.");
            }
        });
    }

    fn delete_module_root(
        &self,
        module_name: &str,
    ) {
        SymbolTreeViewData::cancel_take_over_state(self.symbol_tree_view_data.clone());
        let project_symbols_delete_request = ProjectSymbolsDeleteRequest {
            symbol_locator_keys: Vec::new(),
            module_names: vec![module_name.to_string()],
            module_ranges: Vec::new(),
        };

        project_symbols_delete_request.send(&self.app_context.engine_unprivileged_state, |project_symbols_delete_response| {
            if !project_symbols_delete_response.success {
                log::warn!("Module delete request failed.");
            }
        });
    }

    fn create_module_root(
        &self,
        project_symbols_create_module_request: ProjectSymbolsCreateModuleRequest,
    ) {
        let symbol_tree_view_data = self.symbol_tree_view_data.clone();

        project_symbols_create_module_request.send(&self.app_context.engine_unprivileged_state, move |project_symbols_create_module_response| {
            if project_symbols_create_module_response.success {
                let module_name = project_symbols_create_module_response.module_name;

                SymbolTreeViewData::set_selected_entry(symbol_tree_view_data.clone(), Some(SymbolTreeSelection::ModuleRoot(module_name.clone())));
                SymbolTreeViewData::expand_tree_node(symbol_tree_view_data, &format!("module:{}", module_name));
            }
        });
    }

    fn send_project_symbols_create_requests_sequential<ExecutionContext>(
        engine_unprivileged_state: Arc<ExecutionContext>,
        symbol_tree_view_data: Dependency<SymbolTreeViewData>,
        mut project_symbols_create_requests: Vec<ProjectSymbolsCreateRequest>,
        selection_module_name: Option<String>,
        selected_create_request_position: Option<usize>,
        create_request_position: usize,
    ) where
        ExecutionContext: EngineExecutionContext + 'static,
    {
        if project_symbols_create_requests.is_empty() {
            return;
        }

        let project_symbols_create_request = project_symbols_create_requests.remove(0);
        let should_select_created_symbol = selected_create_request_position.is_some_and(|selected_position| selected_position == create_request_position);
        let next_engine_unprivileged_state = engine_unprivileged_state.clone();
        let next_symbol_tree_view_data = symbol_tree_view_data.clone();
        let next_selection_module_name = selection_module_name.clone();
        let next_selected_create_request_position = selected_create_request_position;

        project_symbols_create_request.send(&engine_unprivileged_state, move |project_symbols_create_response| {
            if !project_symbols_create_response.success {
                log::warn!("Stopping sequential symbol creation after a project-symbols create request failed.");
                return;
            }

            if should_select_created_symbol {
                let created_selection = if selection_module_name.is_some() {
                    SymbolTreeSelection::DerivedNode(format!("module_field:{}", project_symbols_create_response.created_symbol_locator_key))
                } else {
                    SymbolTreeSelection::SymbolClaim(project_symbols_create_response.created_symbol_locator_key)
                };
                SymbolTreeViewData::set_selected_entry(symbol_tree_view_data.clone(), Some(created_selection));

                if let Some(module_name) = selection_module_name {
                    SymbolTreeViewData::expand_tree_node(symbol_tree_view_data.clone(), &format!("module:{}", module_name));
                }
            }

            Self::send_project_symbols_create_requests_sequential(
                next_engine_unprivileged_state,
                next_symbol_tree_view_data,
                project_symbols_create_requests,
                next_selection_module_name,
                next_selected_create_request_position,
                create_request_position.saturating_add(1),
            );
        });
    }

    fn inline_rename_text_storage_id(symbol_locator_key: &str) -> Id {
        Id::new((Self::INLINE_RENAME_TEXT_STORAGE_ID_PREFIX, symbol_locator_key))
    }

    fn inline_rename_highlight_storage_id(symbol_locator_key: &str) -> Id {
        Id::new((Self::INLINE_RENAME_HIGHLIGHT_STORAGE_ID_PREFIX, symbol_locator_key))
    }

    fn clear_inline_rename_state(
        &self,
        user_interface: &mut Ui,
        symbol_locator_key: &str,
    ) {
        let rename_text_storage_id = Self::inline_rename_text_storage_id(symbol_locator_key);
        let rename_highlight_storage_id = Self::inline_rename_highlight_storage_id(symbol_locator_key);

        user_interface.ctx().data_mut(|data| {
            data.remove::<String>(rename_text_storage_id);
            data.remove::<bool>(rename_highlight_storage_id);
        });
        SymbolTreeViewData::cancel_inline_rename(self.symbol_tree_view_data.clone());
    }

    fn build_selected_symbol_tree_entry<'entry>(
        symbol_tree_entries: &'entry [SymbolTreeNode],
        selected_entry: Option<&SymbolTreeSelection>,
    ) -> Option<&'entry SymbolTreeNode> {
        match selected_entry {
            Some(SymbolTreeSelection::ModuleRoot(selected_module_name)) => symbol_tree_entries.iter().find(|symbol_tree_entry| {
                matches!(
                    symbol_tree_entry.get_kind(),
                    SymbolTreeNodeKind::ModuleSpace { module_name, .. } if module_name == selected_module_name
                )
            }),
            Some(SymbolTreeSelection::SymbolClaim(selected_symbol_locator_key)) => symbol_tree_entries.iter().find(|symbol_tree_entry| {
                if Self::is_module_field_tree_entry(symbol_tree_entry) {
                    return false;
                }

                matches!(
                    symbol_tree_entry.get_kind(),
                    SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } if symbol_locator_key == selected_symbol_locator_key
                )
            }),
            Some(SymbolTreeSelection::DerivedNode(selected_node_key)) => symbol_tree_entries
                .iter()
                .find(|symbol_tree_entry| symbol_tree_entry.get_node_key() == selected_node_key),
            _ => None,
        }
    }

    fn resolve_adjacent_symbol_tree_entry<'entry>(
        symbol_tree_entries: &'entry [SymbolTreeNode],
        selected_entry: Option<&SymbolTreeSelection>,
        direction: ListNavigationDirection,
    ) -> Option<&'entry SymbolTreeNode> {
        let selected_symbol_tree_entry = Self::build_selected_symbol_tree_entry(symbol_tree_entries, selected_entry);
        let selected_symbol_tree_index = selected_symbol_tree_entry.and_then(|selected_symbol_tree_entry| {
            symbol_tree_entries
                .iter()
                .position(|symbol_tree_entry| symbol_tree_entry.get_node_key() == selected_symbol_tree_entry.get_node_key())
        });
        let next_selection_index = resolve_next_index(selected_symbol_tree_index, symbol_tree_entries.len(), direction)?;

        symbol_tree_entries.get(next_selection_index)
    }

    fn is_module_field_tree_entry(symbol_tree_entry: &SymbolTreeNode) -> bool {
        symbol_tree_entry.get_node_key().starts_with("module_field:")
    }

    fn edit_symbol_tree_entry_symbol_layout(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_id: &str,
    ) {
        SymbolLayoutEditorViewData::begin_open_symbol_layout(self.symbol_layout_editor_view_data.clone(), project_symbol_catalog, struct_layout_id);

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(SymbolLayoutEditorView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(SymbolLayoutEditorView::WINDOW_ID);
            }
            Err(error) => {
                log::error!(
                    "Failed to acquire docking manager while opening Symbol Layout Editor from Symbol Tree: {}",
                    error
                );
            }
        }
    }

    fn add_symbol_to_project(
        &self,
        add_symbol_to_project_target: &AddSymbolToProjectTarget,
    ) {
        let project_items_create_request = build_add_symbol_project_item_create_request(add_symbol_to_project_target);

        project_items_create_request.send(&self.app_context.engine_unprivileged_state, |project_items_create_response| {
            if !project_items_create_response.success {
                log::warn!("Symbol Tree add-to-project command failed.");
            }
        });
    }

    fn create_define_field_from_unassigned_span_edit_target(
        &self,
        module_name: &str,
        define_field_plan: DefineFieldPlan,
    ) {
        Self::send_project_symbols_create_requests_sequential(
            self.app_context.engine_unprivileged_state.clone(),
            self.symbol_tree_view_data.clone(),
            vec![define_field_plan.project_symbols_create_request],
            Some(module_name.to_string()),
            Some(0),
            0,
        );
    }

    fn request_delete_for_selection(
        &self,
        selected_symbol_claim: Option<&squalr_engine_api::structures::projects::project_symbol_claim::ProjectSymbolClaim>,
        selected_module_name: Option<&str>,
        selected_module_child_range_target: Option<&ModuleChildRangeTarget>,
    ) {
        if let Some(module_child_range_target) = selected_module_child_range_target {
            SymbolTreeViewData::request_delete_module_range_confirmation(
                self.symbol_tree_view_data.clone(),
                module_child_range_target.module_name.clone(),
                module_child_range_target.offset,
                module_child_range_target.length,
                module_child_range_target.display_name.clone(),
                module_child_range_target.delete_mode,
            );
        } else if let Some(symbol_claim) = selected_symbol_claim {
            SymbolTreeViewData::request_delete_symbol_claim_confirmation(
                self.symbol_tree_view_data.clone(),
                symbol_claim.get_symbol_locator_key().to_string(),
                symbol_claim.get_display_name().to_string(),
            );
        } else if let Some(module_name) = selected_module_name {
            SymbolTreeViewData::request_delete_module_root_confirmation(self.symbol_tree_view_data.clone(), module_name.to_string());
        }
    }

    fn build_struct_viewer_focus_target_key(selected_symbol_tree_entry: Option<&SymbolTreeNode>) -> Option<String> {
        selected_symbol_tree_entry.map(|symbol_tree_entry| {
            format!(
                "{}|{}|{}",
                symbol_tree_entry.get_node_key(),
                symbol_tree_entry.get_display_name(),
                symbol_tree_entry.get_display_type_id()
            )
        })
    }

    fn build_struct_viewer_focus_target(selected_symbol_tree_entry: Option<&SymbolTreeNode>) -> Option<StructViewerFocusTarget> {
        Self::build_struct_viewer_focus_target_key(selected_symbol_tree_entry).map(|selection_key| StructViewerFocusTarget::SymbolTree { selection_key })
    }

    fn is_symbol_tree_entry_struct_viewer_focused(
        symbol_tree_entry: &SymbolTreeNode,
        shared_struct_viewer_focus_target: Option<&StructViewerFocusTarget>,
    ) -> bool {
        let Some(StructViewerFocusTarget::SymbolTree { selection_key }) = shared_struct_viewer_focus_target else {
            return false;
        };

        Self::build_struct_viewer_focus_target_key(Some(symbol_tree_entry))
            .as_ref()
            .is_some_and(|row_selection_key| row_selection_key == selection_key)
    }

    fn focus_symbol_tree_entry_in_struct_viewer(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_symbol_tree_entry: &SymbolTreeNode,
    ) {
        let symbol_layout = self.build_symbol_layout_for_tree_entry(project_symbol_catalog, selected_symbol_tree_entry);
        let struct_viewer_edit_callback = self.build_struct_viewer_edit_callback(project_symbol_catalog, selected_symbol_tree_entry);
        let focus_target = Self::build_struct_viewer_focus_target(Some(selected_symbol_tree_entry));

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            symbol_layout,
            struct_viewer_edit_callback,
            focus_target,
        );
    }

    fn sync_selected_symbol_into_struct_viewer(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_symbol_tree_entry: Option<&SymbolTreeNode>,
    ) {
        let current_focus_target = self
            .struct_viewer_view_data
            .read("Symbol tree current struct viewer focus target")
            .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned());
        let selected_symbol_tree_entry =
            selected_symbol_tree_entry.filter(|symbol_tree_entry| !matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. }));
        let desired_focus_target = Self::build_struct_viewer_focus_target(selected_symbol_tree_entry);

        if current_focus_target == desired_focus_target {
            return;
        }

        let Some(selected_symbol_tree_entry) = selected_symbol_tree_entry else {
            if matches!(current_focus_target, Some(StructViewerFocusTarget::SymbolTree { .. })) {
                StructViewerViewData::clear_focus(self.struct_viewer_view_data.clone());
            }
            return;
        };

        if matches!(current_focus_target, Some(StructViewerFocusTarget::ProjectHierarchy { .. })) {
            return;
        }

        self.focus_symbol_tree_entry_in_struct_viewer(project_symbol_catalog, selected_symbol_tree_entry);
    }

    fn build_struct_viewer_edit_callback(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_symbol_tree_entry: &SymbolTreeNode,
    ) -> Arc<dyn Fn(ValuedStructField) + Send + Sync> {
        let symbol_claim_locator_key = match selected_symbol_tree_entry.get_kind() {
            SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } => Some(symbol_locator_key.to_string()),
            _ => None,
        };
        let selected_symbol_tree_entry = selected_symbol_tree_entry.clone();
        let project_symbol_catalog = project_symbol_catalog.clone();
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();
        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();

        Arc::new(move |edited_field: ValuedStructField| {
            if edited_field.get_name() == Self::STRUCT_VIEWER_SYMBOL_NAME_FIELD {
                let Some(symbol_locator_key) = symbol_claim_locator_key.as_ref() else {
                    return;
                };
                let next_display_name = StructViewerViewData::read_utf8_field_text(&edited_field)
                    .trim()
                    .to_string();
                if next_display_name.is_empty() || next_display_name == selected_symbol_tree_entry.get_display_name() {
                    return;
                }

                ProjectSymbolsRenameRequest {
                    symbol_locator_key: symbol_locator_key.clone(),
                    display_name: next_display_name,
                }
                .send(&engine_unprivileged_state, |_project_symbols_rename_response| {});
                return;
            }

            let Some(memory_write_request) = Self::build_memory_write_request_for_symbol_value_edit(
                &engine_execution_context,
                &project_symbol_catalog,
                &selected_symbol_tree_entry,
                &edited_field,
            ) else {
                return;
            };

            memory_write_request.send(&engine_unprivileged_state, |memory_write_response| {
                if !memory_write_response.success {
                    log::warn!("Symbol Tree struct-viewer memory write command failed.");
                }
            });
        })
    }

    fn build_memory_write_request_for_symbol_value_edit(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_symbol_tree_entry: &SymbolTreeNode,
        edited_field: &ValuedStructField,
    ) -> Option<MemoryWriteRequest> {
        let edited_data_value = edited_field.get_data_value()?;
        let symbolic_struct_definition =
            Self::build_named_symbolic_struct_definition_for_value_edit(engine_execution_context, project_symbol_catalog, selected_symbol_tree_entry)?;
        let field_offset = Self::resolve_symbol_layout_field_offset(engine_execution_context, &symbolic_struct_definition, edited_field.get_name())?;
        let address = selected_symbol_tree_entry
            .get_locator()
            .get_focus_address()
            .checked_add(field_offset)?;

        Some(MemoryWriteRequest {
            address,
            module_name: selected_symbol_tree_entry
                .get_locator()
                .get_focus_module_name()
                .to_string(),
            value: edited_data_value.get_value_bytes().clone(),
        })
    }

    fn build_named_symbolic_struct_definition_for_value_edit(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeNode,
    ) -> Option<SymbolicStructDefinition> {
        let symbolic_struct_definition = Self::build_symbolic_struct_definition_for_symbol_type_for_context(
            engine_execution_context,
            project_symbol_catalog,
            symbol_tree_entry.get_symbol_type_id(),
        )?;

        if !symbolic_struct_definition.get_fields().is_empty() {
            return Some(symbolic_struct_definition);
        }

        Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
            DataTypeRef::new(symbol_tree_entry.get_symbol_type_id()),
            symbol_tree_entry.get_container_type(),
        )]))
    }

    fn resolve_symbol_layout_field_offset(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        symbolic_struct_definition: &SymbolicStructDefinition,
        edited_field_name: &str,
    ) -> Option<u64> {
        let mut cumulative_field_offset = 0_u64;

        for (field_index, symbolic_field_definition) in symbolic_struct_definition.get_fields().iter().enumerate() {
            if Self::normalize_symbol_value_field_name(symbolic_field_definition.get_field_name(), field_index) == edited_field_name {
                return Some(cumulative_field_offset);
            }

            cumulative_field_offset = cumulative_field_offset.checked_add(Self::resolve_symbolic_field_size_in_bytes(
                engine_execution_context,
                symbolic_field_definition,
                &mut HashSet::new(),
            )?)?;
        }

        None
    }

    fn resolve_symbolic_field_size_in_bytes(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        symbolic_field_definition: &SymbolicFieldDefinition,
        visited_type_ids: &mut HashSet<String>,
    ) -> Option<u64> {
        if let Some(pointer_size) = symbolic_field_definition
            .get_container_type()
            .get_pointer_size()
        {
            return Some(pointer_size.get_size_in_bytes());
        }

        let data_type_id = symbolic_field_definition
            .get_data_type_ref()
            .get_data_type_id()
            .to_string();
        let unit_size_in_bytes = if let Some(nested_symbolic_struct_definition) = engine_execution_context.resolve_struct_layout_definition(&data_type_id) {
            if !visited_type_ids.insert(data_type_id.clone()) {
                return None;
            }

            let nested_size_in_bytes =
                Self::resolve_symbolic_struct_size_in_bytes(engine_execution_context, &nested_symbolic_struct_definition, visited_type_ids)?;

            visited_type_ids.remove(&data_type_id);

            nested_size_in_bytes
        } else if let Some(default_value) = engine_execution_context.get_default_value(symbolic_field_definition.get_data_type_ref()) {
            default_value.get_size_in_bytes()
        } else {
            return None;
        };

        Some(
            symbolic_field_definition
                .get_container_type()
                .get_total_size_in_bytes(unit_size_in_bytes),
        )
    }

    fn resolve_symbolic_struct_size_in_bytes(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        symbolic_struct_definition: &SymbolicStructDefinition,
        visited_type_ids: &mut HashSet<String>,
    ) -> Option<u64> {
        let mut next_sequential_offset = 0_u64;

        for symbolic_field_definition in symbolic_struct_definition.get_fields() {
            let field_offset = match symbolic_field_definition.get_offset_resolution() {
                SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_)
                    if symbolic_struct_definition.get_layout_kind().is_union() =>
                {
                    0
                }
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
            };
            let field_size_in_bytes = Self::resolve_symbolic_field_size_in_bytes(engine_execution_context, symbolic_field_definition, visited_type_ids)?;

            next_sequential_offset = next_sequential_offset.max(field_offset.checked_add(field_size_in_bytes)?);
        }

        Some(
            next_sequential_offset.max(
                symbolic_struct_definition
                    .get_declared_size_in_bytes()
                    .unwrap_or(0),
            ),
        )
    }

    fn normalize_symbol_value_field_name(
        field_name: &str,
        field_index: usize,
    ) -> String {
        if field_name.trim().is_empty() {
            if field_index == 0 {
                String::from("value")
            } else {
                format!("value_{}", field_index)
            }
        } else {
            field_name.to_string()
        }
    }

    fn build_named_symbolic_struct_definition_for_preview(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeNode,
        truncate_preview_arrays: bool,
    ) -> Option<SymbolicStructDefinition> {
        let entry_field_definition = SymbolicFieldDefinition::from_str(&symbol_tree_entry.get_display_type_id()).ok()?;
        let preview_container_type = if truncate_preview_arrays {
            match entry_field_definition.get_container_type() {
                ContainerType::ArrayFixed(length) if length > Self::MAX_SYMBOL_PREVIEW_ELEMENT_COUNT => {
                    ContainerType::ArrayFixed(Self::MAX_SYMBOL_PREVIEW_ELEMENT_COUNT)
                }
                container_type => container_type,
            }
        } else {
            entry_field_definition.get_container_type()
        };

        let resolved_symbolic_struct_definition =
            self.build_symbolic_struct_definition_for_symbol_type(project_symbol_catalog, entry_field_definition.get_data_type_ref().get_data_type_id())?;

        if resolved_symbolic_struct_definition.get_fields().len() > 1 {
            return None;
        }

        if resolved_symbolic_struct_definition.get_fields().is_empty() || preview_container_type != ContainerType::None {
            return Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                entry_field_definition.get_data_type_ref().clone(),
                preview_container_type,
            )]));
        }

        Some(resolved_symbolic_struct_definition)
    }

    fn build_symbolic_struct_definition_for_symbol_type_static(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_type_id: &str,
    ) -> Option<SymbolicStructDefinition> {
        if let Some(project_struct_layout_descriptor) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == symbol_type_id)
        {
            return Some(
                project_struct_layout_descriptor
                    .get_struct_layout_definition()
                    .clone(),
            );
        }

        if let Ok(symbolic_struct_definition) = SymbolicStructDefinition::from_str(symbol_type_id) {
            return Some(symbolic_struct_definition);
        }

        if let Ok(symbolic_field_definition) = SymbolicFieldDefinition::from_str(symbol_type_id) {
            return Some(SymbolicStructDefinition::new_anonymous(vec![symbolic_field_definition]));
        }

        Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
            DataTypeRef::new(symbol_type_id),
            Default::default(),
        )]))
    }

    fn build_symbol_layout_for_tree_entry(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeNode,
    ) -> ValuedStruct {
        let include_symbol_claim_metadata = matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::SymbolClaim { .. });
        let engine_execution_context: Arc<dyn EngineExecutionContext> = self.app_context.engine_unprivileged_state.clone();
        let symbol_size_in_bytes = Self::resolve_symbol_tree_entry_size_for_struct_viewer(&engine_execution_context, symbol_tree_entry);

        if Self::symbol_tree_entry_should_use_external_value_viewer(symbol_tree_entry) {
            return Self::build_external_value_symbol_layout(symbol_tree_entry, include_symbol_claim_metadata, symbol_size_in_bytes);
        }

        let Some(symbolic_struct_definition) = self.build_named_symbolic_struct_definition_for_symbol_tree_entry(project_symbol_catalog, symbol_tree_entry)
        else {
            return self.build_symbol_layout_fallback(
                symbol_tree_entry,
                "Unable to resolve a struct definition for the selected symbol.",
                include_symbol_claim_metadata,
                symbol_size_in_bytes,
            );
        };

        let memory_read_response = Self::dispatch_memory_read_request(
            &engine_execution_context,
            symbol_tree_entry.get_locator().get_focus_address(),
            symbol_tree_entry.get_locator().get_focus_module_name(),
            &symbolic_struct_definition,
        );
        let Some(memory_read_response) = memory_read_response else {
            return self.build_symbol_layout_fallback(
                symbol_tree_entry,
                "Timed out while reading the selected symbol from memory.",
                include_symbol_claim_metadata,
                symbol_size_in_bytes,
            );
        };

        if !memory_read_response.success {
            return self.build_symbol_layout_fallback(
                symbol_tree_entry,
                "The selected symbol could not be read from memory.",
                include_symbol_claim_metadata,
                symbol_size_in_bytes,
            );
        }

        Self::normalize_symbol_memory_struct(
            memory_read_response.valued_struct,
            symbol_tree_entry,
            include_symbol_claim_metadata,
            symbol_size_in_bytes,
        )
    }

    fn build_named_symbolic_struct_definition_for_symbol_tree_entry(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeNode,
    ) -> Option<SymbolicStructDefinition> {
        self.build_symbolic_struct_definition_for_symbol_type(project_symbol_catalog, symbol_tree_entry.get_symbol_type_id())
            .map(|symbolic_struct_definition| {
                if !symbolic_struct_definition.get_fields().is_empty() {
                    return symbolic_struct_definition;
                }

                SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new(symbol_tree_entry.get_symbol_type_id()),
                    symbol_tree_entry.get_container_type(),
                )])
            })
    }

    fn normalize_symbol_memory_struct(
        valued_struct: ValuedStruct,
        symbol_tree_entry: &SymbolTreeNode,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
    ) -> ValuedStruct {
        let mut normalized_fields = Self::build_symbol_layout_metadata_fields(symbol_tree_entry, include_symbol_claim_metadata, symbol_size_in_bytes);

        normalized_fields.extend(
            valued_struct
                .get_fields()
                .iter()
                .enumerate()
                .map(|(field_index, valued_struct_field)| {
                    let resolved_field_name = Self::normalize_symbol_value_field_name(valued_struct_field.get_name(), field_index);

                    ValuedStructField::new(resolved_field_name, valued_struct_field.get_field_data().clone(), false)
                })
                .collect::<Vec<_>>(),
        );

        ValuedStruct::new_anonymous(normalized_fields)
    }

    fn build_symbol_layout_fallback(
        &self,
        symbol_tree_entry: &SymbolTreeNode,
        status_text: &str,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
    ) -> ValuedStruct {
        let mut fallback_fields = Self::build_symbol_layout_metadata_fields(symbol_tree_entry, include_symbol_claim_metadata, symbol_size_in_bytes);

        fallback_fields.extend([
            DataTypeStringUtf8::get_value_from_primitive_string(&symbol_tree_entry.get_locator().to_string())
                .to_named_valued_struct_field(String::from("locator"), true),
            DataTypeStringUtf8::get_value_from_primitive_string(status_text).to_named_valued_struct_field(String::from("status"), true),
        ]);

        ValuedStruct::new_anonymous(fallback_fields)
    }

    fn build_external_value_symbol_layout(
        symbol_tree_entry: &SymbolTreeNode,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
    ) -> ValuedStruct {
        let mut fields = Self::build_symbol_layout_metadata_fields(symbol_tree_entry, include_symbol_claim_metadata, symbol_size_in_bytes);

        fields.push(
            DataTypeStringUtf8::get_value_from_primitive_string("")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE.to_string(), true),
        );

        ValuedStruct::new_anonymous(fields)
    }

    fn build_symbol_layout_metadata_fields(
        symbol_tree_entry: &SymbolTreeNode,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
    ) -> Vec<ValuedStructField> {
        let mut metadata_fields = Vec::new();

        if include_symbol_claim_metadata {
            metadata_fields.push(
                DataTypeStringUtf8::get_value_from_primitive_string(symbol_tree_entry.get_display_name())
                    .to_named_valued_struct_field(Self::STRUCT_VIEWER_SYMBOL_NAME_FIELD.to_string(), false),
            );
        }

        metadata_fields.push(
            DataTypeStringUtf8::get_value_from_primitive_string(&symbol_tree_entry.get_display_type_id())
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), true),
        );

        metadata_fields.extend(Self::build_symbol_layout_location_fields(symbol_tree_entry, symbol_size_in_bytes));

        metadata_fields
    }

    fn build_symbol_layout_location_fields(
        symbol_tree_entry: &SymbolTreeNode,
        symbol_size_in_bytes: Option<u64>,
    ) -> Vec<ValuedStructField> {
        let mut location_fields = Vec::new();
        let locator = symbol_tree_entry.get_locator();

        location_fields.push(
            DataTypeU64::get_value_from_primitive(locator.get_focus_address())
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_ADDRESS.to_string(), true),
        );

        location_fields.push(
            DataTypeStringUtf8::get_value_from_primitive_string(locator.get_focus_module_name())
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_MODULE.to_string(), true),
        );

        if let Some(symbol_size_in_bytes) = symbol_size_in_bytes {
            location_fields.push(
                DataTypeU64::get_value_from_primitive(symbol_size_in_bytes)
                    .to_named_valued_struct_field(Self::STRUCT_VIEWER_SYMBOL_SIZE_FIELD.to_string(), true),
            );
        }

        if !symbol_tree_entry.get_full_path().is_empty() {
            location_fields.push(
                DataTypeStringUtf8::get_value_from_primitive_string(symbol_tree_entry.get_full_path())
                    .to_named_valued_struct_field(Self::STRUCT_VIEWER_SYMBOL_PATH_FIELD.to_string(), true),
            );
        }

        location_fields
    }

    fn resolve_symbol_tree_entry_size_for_struct_viewer(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        symbol_tree_entry: &SymbolTreeNode,
    ) -> Option<u64> {
        let symbolic_field_definition = SymbolicFieldDefinition::from_str(&symbol_tree_entry.get_display_type_id()).ok()?;

        Self::resolve_symbolic_field_size_in_bytes(engine_execution_context, &symbolic_field_definition, &mut HashSet::new())
    }

    fn symbol_tree_entry_should_use_external_value_viewer(symbol_tree_entry: &SymbolTreeNode) -> bool {
        if matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::UnassignedSegment { .. }) {
            return false;
        }

        matches!(symbol_tree_entry.get_container_type(), ContainerType::Array | ContainerType::ArrayFixed(_))
    }

    fn dispatch_memory_read_request(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        address: u64,
        module_name: &str,
        symbolic_struct_definition: &SymbolicStructDefinition,
    ) -> Option<MemoryReadResponse> {
        let memory_read_request = MemoryReadRequest {
            address,
            module_name: module_name.to_string(),
            symbolic_struct_definition: symbolic_struct_definition.clone(),
            suppress_logging: true,
        };
        let memory_read_command = memory_read_request.to_engine_command();
        let (memory_read_response_sender, memory_read_response_receiver) = mpsc::channel();

        let dispatch_result = match engine_execution_context.get_bindings().read() {
            Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
                memory_read_command,
                Box::new(move |engine_response| {
                    let conversion_result = match MemoryReadResponse::from_engine_response(engine_response) {
                        Ok(memory_read_response) => Ok(memory_read_response),
                        Err(unexpected_response) => Err(format!(
                            "Unexpected response variant for Symbol Tree memory read request: {:?}",
                            unexpected_response
                        )),
                    };
                    let _ = memory_read_response_sender.send(conversion_result);
                }),
            ),
            Err(error) => {
                log::error!("Failed to acquire engine bindings lock for Symbol Tree memory read request: {}", error);
                return None;
            }
        };

        if let Err(error) = dispatch_result {
            log::error!("Failed to dispatch Symbol Tree memory read request: {}", error);
            return None;
        }

        match memory_read_response_receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(memory_read_response)) => Some(memory_read_response),
            Ok(Err(error)) => {
                log::error!("Failed to convert Symbol Tree memory read response: {}", error);
                None
            }
            Err(error) => {
                log::error!("Timed out waiting for Symbol Tree memory read response: {}", error);
                None
            }
        }
    }

    fn sync_pointer_child_virtual_snapshot(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeNode],
        additional_pointer_snapshot_queries: Vec<VirtualSnapshotQuery>,
    ) {
        let mut pointer_snapshot_queries = self.build_pointer_snapshot_queries(project_symbol_catalog, symbol_tree_entries);

        pointer_snapshot_queries.extend(additional_pointer_snapshot_queries);

        self.app_context
            .engine_unprivileged_state
            .set_virtual_snapshot_queries(
                Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID,
                Self::POINTER_CHILDREN_REFRESH_INTERVAL,
                SymbolTreeScalarValue::deduplicate_queries_by_id(pointer_snapshot_queries),
            );
        self.app_context
            .engine_unprivileged_state
            .request_virtual_snapshot_refresh(Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID);
    }

    fn build_pointer_snapshot_queries(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeNode],
    ) -> Vec<VirtualSnapshotQuery> {
        symbol_tree_entries
            .iter()
            .filter(|symbol_tree_entry| {
                symbol_tree_entry.is_expanded()
                    && !matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::PointerTarget)
                    && symbol_tree_entry
                        .get_container_type()
                        .get_pointer_size()
                        .is_some()
            })
            .filter_map(|symbol_tree_entry| self.build_pointer_virtual_snapshot_query(project_symbol_catalog, symbol_tree_entry))
            .collect()
    }

    fn build_pointer_virtual_snapshot_query(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeNode,
    ) -> Option<VirtualSnapshotQuery> {
        let pointer_size = symbol_tree_entry.get_container_type().get_pointer_size()?;
        let symbolic_struct_definition =
            self.build_symbolic_struct_definition_for_symbol_type(project_symbol_catalog, symbol_tree_entry.get_symbol_type_id())?;

        Some(VirtualSnapshotQuery::Pointer {
            query_id: symbol_tree_entry.get_node_key().to_string(),
            pointer: Pointer::new_with_size(
                symbol_tree_entry.get_locator().get_focus_address(),
                vec![0],
                symbol_tree_entry
                    .get_locator()
                    .get_focus_module_name()
                    .to_string(),
                pointer_size,
            ),
            symbolic_struct_definition,
        })
    }

    fn resolve_global_pointer_chain_from_pointer_snapshot(
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolved_pointer_targets_by_query_id: &HashMap<String, ResolvedPointerTarget>,
        resolver_pointer_snapshot_queries: &RefCell<Vec<VirtualSnapshotQuery>>,
        pointer_chain: &SymbolicPointerChain,
    ) -> Result<i128, SymbolicResolverEvaluationError> {
        let query_id = Self::global_pointer_chain_query_id(pointer_chain);

        if let Some(resolved_pointer_target) = resolved_pointer_targets_by_query_id.get(&query_id) {
            return Ok(i128::from(resolved_pointer_target.get_target_locator().get_focus_address()));
        }

        let Some(pointer_snapshot_query) = Self::build_global_pointer_chain_virtual_snapshot_query(project_symbol_catalog, pointer_chain, query_id) else {
            return Err(SymbolicResolverEvaluationError::UnknownGlobalPointerChain(pointer_chain.to_string()));
        };

        resolver_pointer_snapshot_queries
            .borrow_mut()
            .push(pointer_snapshot_query);

        Err(SymbolicResolverEvaluationError::UnknownGlobalPointerChain(pointer_chain.to_string()))
    }

    fn resolve_relative_pointer_chain_from_pointer_snapshot(
        resolved_pointer_targets_by_query_id: &HashMap<String, ResolvedPointerTarget>,
        resolver_pointer_snapshot_queries: &RefCell<Vec<VirtualSnapshotQuery>>,
        root_locator: &ProjectSymbolLocator,
        pointer_chain: &SymbolicPointerChain,
    ) -> Result<i128, SymbolicResolverEvaluationError> {
        let query_id = Self::relative_pointer_chain_query_id(root_locator, pointer_chain);

        if let Some(resolved_pointer_target) = resolved_pointer_targets_by_query_id.get(&query_id) {
            return Ok(i128::from(resolved_pointer_target.get_target_locator().get_focus_address()));
        }

        let Some(pointer_snapshot_query) = Self::build_relative_pointer_chain_virtual_snapshot_query(root_locator, pointer_chain, query_id) else {
            return Err(SymbolicResolverEvaluationError::UnknownRelativePointerChain(pointer_chain.to_string()));
        };

        resolver_pointer_snapshot_queries
            .borrow_mut()
            .push(pointer_snapshot_query);

        Err(SymbolicResolverEvaluationError::UnknownRelativePointerChain(pointer_chain.to_string()))
    }

    fn build_global_pointer_chain_virtual_snapshot_query(
        project_symbol_catalog: &ProjectSymbolCatalog,
        pointer_chain: &SymbolicPointerChain,
        query_id: String,
    ) -> Option<VirtualSnapshotQuery> {
        let resolved_pointer_chain = pointer_chain.with_resolved_symbols(|module_name, symbol_name| {
            project_symbol_catalog
                .find_module_symbol_offset_by_display_name(module_name, symbol_name)
                .and_then(|symbol_offset| i64::try_from(symbol_offset).ok())
        })?;
        let root_offset = resolved_pointer_chain.get_numeric_root_offset()?;
        let root_offset = u64::try_from(root_offset).ok()?;

        Some(VirtualSnapshotQuery::Pointer {
            query_id,
            pointer: Pointer::new_with_size_and_segments(
                root_offset,
                resolved_pointer_chain.get_tail_links().to_vec(),
                resolved_pointer_chain.get_module_name().to_string(),
                resolved_pointer_chain.get_pointer_size(),
            ),
            symbolic_struct_definition: SymbolicStructDefinition::new_anonymous(Vec::new()),
        })
    }

    fn build_relative_pointer_chain_virtual_snapshot_query(
        root_locator: &ProjectSymbolLocator,
        pointer_chain: &SymbolicPointerChain,
        query_id: String,
    ) -> Option<VirtualSnapshotQuery> {
        let root_offset = pointer_chain.get_numeric_root_offset()?;
        let root_address = Pointer::apply_pointer_offset(root_locator.get_focus_address(), root_offset)?;

        Some(VirtualSnapshotQuery::Pointer {
            query_id,
            pointer: Pointer::new_with_size_and_segments(
                root_address,
                pointer_chain.get_tail_links().to_vec(),
                root_locator.get_focus_module_name().to_string(),
                pointer_chain.get_pointer_size(),
            ),
            symbolic_struct_definition: SymbolicStructDefinition::new_anonymous(Vec::new()),
        })
    }

    fn global_pointer_chain_query_id(pointer_chain: &SymbolicPointerChain) -> String {
        format!(
            "resolver_pointer:{}:{}:{}",
            pointer_chain.get_module_name(),
            pointer_chain.get_pointer_size(),
            SymbolicPointerChainLink::display_text_list(pointer_chain.get_links())
        )
    }

    fn relative_pointer_chain_query_id(
        root_locator: &ProjectSymbolLocator,
        pointer_chain: &SymbolicPointerChain,
    ) -> String {
        format!(
            "resolver_relative_pointer:{}:{}:{}",
            root_locator.to_locator_key(),
            pointer_chain.get_pointer_size(),
            SymbolicPointerChainLink::display_text_list(pointer_chain.get_links())
        )
    }

    fn build_symbolic_struct_definition_for_symbol_type(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_type_id: &str,
    ) -> Option<SymbolicStructDefinition> {
        let engine_execution_context: Arc<dyn EngineExecutionContext> = self.app_context.engine_unprivileged_state.clone();

        Self::build_symbolic_struct_definition_for_symbol_type_for_context(&engine_execution_context, project_symbol_catalog, symbol_type_id)
    }

    fn build_symbolic_struct_definition_for_symbol_type_for_context(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_type_id: &str,
    ) -> Option<SymbolicStructDefinition> {
        if let Some(project_struct_layout_descriptor) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == symbol_type_id)
        {
            return Some(
                project_struct_layout_descriptor
                    .get_struct_layout_definition()
                    .clone(),
            );
        }

        if let Ok(symbolic_struct_definition) = SymbolicStructDefinition::from_str(symbol_type_id) {
            return Some(symbolic_struct_definition);
        }

        if let Some(symbolic_struct_definition) = engine_execution_context.resolve_struct_layout_definition(symbol_type_id) {
            return Some(symbolic_struct_definition);
        }

        Self::build_symbolic_struct_definition_for_symbol_type_static(project_symbol_catalog, symbol_type_id)
    }

    fn collect_resolved_pointer_targets_by_node_key(&self) -> HashMap<String, ResolvedPointerTarget> {
        let Some(virtual_snapshot) = self
            .app_context
            .engine_unprivileged_state
            .get_virtual_snapshot(Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID)
        else {
            return HashMap::new();
        };

        virtual_snapshot
            .get_query_results()
            .iter()
            .filter_map(|(query_id, virtual_snapshot_query_result)| {
                let resolved_address = virtual_snapshot_query_result.resolved_address?;
                let target_locator = if virtual_snapshot_query_result.resolved_module_name.is_empty() {
                    ProjectSymbolLocator::new_absolute_address(resolved_address)
                } else {
                    ProjectSymbolLocator::new_module_offset(virtual_snapshot_query_result.resolved_module_name.clone(), resolved_address)
                };

                Some((
                    query_id.clone(),
                    ResolvedPointerTarget::new(target_locator, virtual_snapshot_query_result.evaluated_pointer_path.clone()),
                ))
            })
            .collect()
    }

    fn sync_symbol_scalar_virtual_snapshot(
        &self,
        scalar_snapshot_queries: Vec<VirtualSnapshotQuery>,
    ) {
        self.app_context
            .engine_unprivileged_state
            .set_virtual_snapshot_queries(
                Self::SCALAR_VALUES_VIRTUAL_SNAPSHOT_ID,
                Self::SCALAR_VALUES_REFRESH_INTERVAL,
                SymbolTreeScalarValue::deduplicate_queries_by_id(scalar_snapshot_queries),
            );
        self.app_context
            .engine_unprivileged_state
            .request_virtual_snapshot_refresh(Self::SCALAR_VALUES_VIRTUAL_SNAPSHOT_ID);
    }

    fn collect_scalar_values_by_query_id(&self) -> HashMap<String, i128> {
        let Some(virtual_snapshot) = self
            .app_context
            .engine_unprivileged_state
            .get_virtual_snapshot(Self::SCALAR_VALUES_VIRTUAL_SNAPSHOT_ID)
        else {
            return HashMap::new();
        };

        virtual_snapshot
            .get_query_results()
            .iter()
            .filter_map(|(query_id, virtual_snapshot_query_result)| {
                let memory_read_response = virtual_snapshot_query_result.memory_read_response.as_ref()?;

                if !memory_read_response.success {
                    return None;
                }

                let first_read_field_data_value = memory_read_response
                    .valued_struct
                    .get_fields()
                    .first()
                    .and_then(|valued_struct_field| valued_struct_field.get_data_value())?;
                let scalar_value = self
                    .app_context
                    .engine_unprivileged_state
                    .read_scalar_integer_value(first_read_field_data_value)
                    .ok()
                    .flatten()?;

                Some((query_id.clone(), scalar_value))
            })
            .collect()
    }

    fn sync_symbol_preview_virtual_snapshot(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeNode],
    ) {
        let preview_snapshot_queries = self.build_symbol_preview_snapshot_queries(project_symbol_catalog, symbol_tree_entries);

        self.app_context
            .engine_unprivileged_state
            .set_virtual_snapshot_queries(
                Self::PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID,
                Self::PREVIEW_VALUES_REFRESH_INTERVAL,
                preview_snapshot_queries,
            );
        self.app_context
            .engine_unprivileged_state
            .request_virtual_snapshot_refresh(Self::PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID);
    }

    fn build_symbol_preview_snapshot_queries(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeNode],
    ) -> Vec<VirtualSnapshotQuery> {
        symbol_tree_entries
            .iter()
            .filter(|symbol_tree_entry| Self::symbol_tree_entry_should_query_preview(symbol_tree_entry))
            .filter_map(|symbol_tree_entry| self.build_symbol_preview_virtual_snapshot_query(project_symbol_catalog, symbol_tree_entry))
            .collect()
    }

    fn symbol_tree_entry_should_query_preview(symbol_tree_entry: &SymbolTreeNode) -> bool {
        !matches!(
            symbol_tree_entry.get_kind(),
            SymbolTreeNodeKind::ModuleSpace { .. } | SymbolTreeNodeKind::UnassignedSegment { .. }
        )
    }

    fn build_symbol_preview_virtual_snapshot_query(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeNode,
    ) -> Option<VirtualSnapshotQuery> {
        let symbolic_struct_definition = self.build_named_symbolic_struct_definition_for_preview(project_symbol_catalog, symbol_tree_entry, true)?;

        Some(VirtualSnapshotQuery::Address {
            query_id: symbol_tree_entry.get_node_key().to_string(),
            address: symbol_tree_entry.get_locator().get_focus_address(),
            module_name: symbol_tree_entry
                .get_locator()
                .get_focus_module_name()
                .to_string(),
            symbolic_struct_definition,
        })
    }

    fn collect_preview_values_by_node_key(
        &self,
        symbol_tree_entries: &[SymbolTreeNode],
    ) -> HashMap<String, String> {
        let Some(virtual_snapshot) = self
            .app_context
            .engine_unprivileged_state
            .get_virtual_snapshot(Self::PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID)
        else {
            return HashMap::new();
        };

        symbol_tree_entries
            .iter()
            .filter_map(|symbol_tree_entry| {
                let virtual_snapshot_query_result = virtual_snapshot
                    .get_query_results()
                    .get(symbol_tree_entry.get_node_key())?;
                let preview_value = self.build_symbol_preview_value(symbol_tree_entry, virtual_snapshot_query_result);

                (!preview_value.is_empty()).then(|| (symbol_tree_entry.get_node_key().to_string(), preview_value))
            })
            .collect()
    }

    fn build_symbol_preview_value(
        &self,
        symbol_tree_entry: &SymbolTreeNode,
        virtual_snapshot_query_result: &VirtualSnapshotQueryResult,
    ) -> String {
        let Some(memory_read_response) = virtual_snapshot_query_result.memory_read_response.as_ref() else {
            return String::new();
        };

        if !memory_read_response.success {
            return String::new();
        }

        let Some(first_read_field_data_value) = memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field| valued_struct_field.get_data_value())
        else {
            return String::new();
        };

        let default_anonymous_value_string_format = self
            .app_context
            .engine_unprivileged_state
            .get_default_anonymous_value_string_format(first_read_field_data_value.get_data_type_ref());

        self.app_context
            .engine_unprivileged_state
            .anonymize_value(first_read_field_data_value, default_anonymous_value_string_format)
            .map(|anonymous_value_string| {
                Self::format_symbol_preview_value(
                    &anonymous_value_string,
                    symbol_tree_entry.get_container_type(),
                    Self::symbol_preview_was_truncated(symbol_tree_entry),
                )
            })
            .unwrap_or_default()
    }

    fn symbol_preview_was_truncated(symbol_tree_entry: &SymbolTreeNode) -> bool {
        matches!(
            symbol_tree_entry.get_container_type(),
            ContainerType::ArrayFixed(length) if length > Self::MAX_SYMBOL_PREVIEW_ELEMENT_COUNT
        )
    }

    fn format_symbol_preview_value(
        anonymous_value_string: &AnonymousValueString,
        symbolic_field_container_type: ContainerType,
        preview_was_truncated: bool,
    ) -> String {
        let effective_container_type = if matches!(anonymous_value_string.get_container_type(), ContainerType::Array | ContainerType::ArrayFixed(_)) {
            anonymous_value_string.get_container_type()
        } else {
            symbolic_field_container_type
        };
        let display_value = anonymous_value_string.get_anonymous_value_string();

        if matches!(effective_container_type, ContainerType::Array | ContainerType::ArrayFixed(_)) && !display_value.is_empty() {
            let preview_value = if preview_was_truncated {
                Self::append_symbol_preview_ellipsis(display_value)
            } else {
                Self::truncate_symbol_preview_value(display_value)
            };

            format!("[{}]", preview_value)
        } else {
            display_value.to_string()
        }
    }

    fn append_symbol_preview_ellipsis(display_value: &str) -> String {
        if let Some(truncated_array_preview) = Self::format_symbol_preview_from_elements(display_value, true) {
            return truncated_array_preview;
        }

        let trimmed_display_value = display_value.trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'));

        if trimmed_display_value.is_empty() {
            String::from("...")
        } else {
            format!("{}...", trimmed_display_value)
        }
    }

    fn truncate_symbol_preview_value(display_value: &str) -> String {
        if let Some(truncated_array_preview) = Self::format_symbol_preview_from_elements(display_value, false) {
            return truncated_array_preview;
        }

        let display_value_character_count = display_value.chars().count();

        if display_value_character_count <= Self::MAX_SYMBOL_PREVIEW_ARRAY_CHARACTER_COUNT {
            return display_value.to_string();
        }

        let truncated_prefix: String = display_value
            .chars()
            .take(Self::MAX_SYMBOL_PREVIEW_ARRAY_CHARACTER_COUNT)
            .collect::<String>()
            .trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'))
            .to_string();

        format!("{}...", truncated_prefix)
    }

    fn format_symbol_preview_from_elements(
        display_value: &str,
        force_ellipsis: bool,
    ) -> Option<String> {
        let array_elements = Self::split_symbol_preview_elements(display_value);

        if array_elements.len() <= 1 {
            return None;
        }

        let visible_element_count = array_elements
            .len()
            .min(Self::MAX_SYMBOL_PREVIEW_DISPLAY_ELEMENT_COUNT);
        let mut preview_elements = array_elements
            .iter()
            .take(visible_element_count)
            .map(|array_element| (*array_element).to_string())
            .collect::<Vec<_>>();
        let has_hidden_elements = force_ellipsis || array_elements.len() > visible_element_count;

        if has_hidden_elements {
            preview_elements.push(String::from("..."));
        }

        Some(preview_elements.join(", "))
    }

    fn split_symbol_preview_elements(display_value: &str) -> Vec<&str> {
        display_value
            .split([',', ';'])
            .map(str::trim)
            .filter(|array_element| !array_element.is_empty())
            .collect::<Vec<_>>()
    }

    fn format_symbol_tree_size_preview(size_in_bytes: u64) -> String {
        const KIB: f64 = 1024.0;
        const SIZE_UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];

        if size_in_bytes < 1024 {
            return format!("{} B", size_in_bytes);
        }

        let mut unit_index = 0_usize;
        let mut size_value = size_in_bytes as f64;

        while size_value >= KIB && unit_index + 1 < SIZE_UNITS.len() {
            size_value /= KIB;
            unit_index += 1;
        }

        let formatted_value = if size_value >= 100.0 {
            format!("{:.0}", size_value)
        } else if size_value >= 10.0 {
            format!("{:.1}", size_value)
        } else {
            format!("{:.2}", size_value)
        };
        let formatted_value = formatted_value
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();

        format!("{} {}", formatted_value, SIZE_UNITS[unit_index])
    }

    fn format_symbol_tree_size_tooltip(size_in_bytes: u64) -> String {
        let scaled_size_text = Self::format_symbol_tree_size_preview(size_in_bytes);
        let raw_size_text = format!("{} B", size_in_bytes);

        if scaled_size_text == raw_size_text {
            raw_size_text
        } else {
            format!("{} ({})", raw_size_text, scaled_size_text)
        }
    }

    fn draw_text_button(
        &self,
        user_interface: &mut Ui,
        label: &str,
        fill_color: Color32,
        enabled: bool,
        minimum_width: f32,
    ) -> Response {
        let theme = &self.app_context.theme;
        let text_color = if enabled { theme.foreground } else { theme.foreground_preview };
        let label_galley = user_interface
            .painter()
            .layout_no_wrap(label.to_string(), theme.font_library.font_noto_sans.font_normal.clone(), text_color);
        let desired_width = (label_galley.size().x + 18.0).max(minimum_width);
        let button_response = user_interface.add_sized(
            [desired_width, 28.0],
            ThemeButton::new_from_theme(theme)
                .disabled(!enabled)
                .background_color(fill_color),
        );
        let text_position = pos2(
            button_response.rect.center().x - label_galley.size().x * 0.5,
            button_response.rect.center().y - label_galley.size().y * 0.5,
        );

        user_interface
            .painter()
            .galley(text_position, label_galley, text_color);

        button_response
    }

    fn build_symbol_tree_action_context(symbol_tree_entry: &SymbolTreeNode) -> SymbolTreeActionContext {
        match symbol_tree_entry.get_kind() {
            SymbolTreeNodeKind::ModuleSpace { module_name, .. } => SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
                module_name: module_name.to_string(),
            }),
            SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } => SymbolTreeActionContext::new(SymbolTreeActionSelection::SymbolLocator {
                symbol_locator_key: symbol_locator_key.to_string(),
            }),
            SymbolTreeNodeKind::UnassignedSegment { module_name, offset, length } => SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRange {
                module_name: module_name.to_string(),
                offset: *offset,
                length: *length,
            }),
            SymbolTreeNodeKind::StructField | SymbolTreeNodeKind::PointerTarget => SymbolTreeActionContext::new(SymbolTreeActionSelection::DerivedNode {
                tree_node_key: symbol_tree_entry.get_node_key().to_string(),
            }),
        }
    }

    fn build_symbol_tree_plugin_action_menu_items(
        &self,
        context: &SymbolTreeActionContext,
    ) -> Vec<SymbolTreePluginActionMenuItem> {
        let plugin_registry = self.app_context.engine_unprivileged_state.get_plugin_registry();

        plugin_registry
            .get_enabled_symbol_tree_actions()
            .into_iter()
            .filter(|(plugin_id, symbol_tree_action)| {
                symbol_tree_action.is_visible(context) && plugin_registry.plugin_action_has_required_permissions(plugin_id, symbol_tree_action.as_ref())
            })
            .map(|(plugin_id, symbol_tree_action)| SymbolTreePluginActionMenuItem {
                plugin_id,
                action_id: symbol_tree_action.action_id().to_string(),
                label: symbol_tree_action.label(context),
            })
            .collect()
    }

    fn execute_symbol_tree_plugin_action(
        &self,
        menu_item: &SymbolTreePluginActionMenuItem,
        context: SymbolTreeActionContext,
    ) {
        let symbol_tree_view_data = self.symbol_tree_view_data.clone();
        let project_symbols_execute_plugin_action_request = ProjectSymbolsExecutePluginActionRequest {
            plugin_id: menu_item.plugin_id.clone(),
            action_id: menu_item.action_id.clone(),
            context: context.clone(),
        };

        project_symbols_execute_plugin_action_request.send(
            &self.app_context.engine_unprivileged_state,
            move |project_symbols_execute_plugin_action_response| {
                if !project_symbols_execute_plugin_action_response.success {
                    log::warn!(
                        "Symbol Tree plugin action failed: {}",
                        project_symbols_execute_plugin_action_response
                            .error
                            .unwrap_or_else(|| String::from("unknown error"))
                    );
                    return;
                }

                if let SymbolTreeActionSelection::ModuleRoot { module_name } = context.get_selection() {
                    SymbolTreeViewData::expand_tree_node(symbol_tree_view_data, &format!("module:{module_name}"));
                }
            },
        );
    }

    #[allow(dead_code)]
    fn render_symbol_tree_list_legacy(
        &self,
        user_interface: &mut Ui,
        symbol_tree_entries: &[SymbolTreeNode],
        selected_entry: Option<&SymbolTreeSelection>,
    ) {
        user_interface.horizontal(|user_interface| {
            user_interface.add_space(Self::SYMBOL_TREE_TEXT_PADDING_X);
            user_interface.label(
                RichText::new(format!(
                    "Symbol Tree ({})",
                    symbol_tree_entries
                        .iter()
                        .filter(|symbol_tree_entry| matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. }))
                        .count()
                ))
                .font(
                    self.app_context
                        .theme
                        .font_library
                        .font_noto_sans
                        .font_header
                        .clone(),
                )
                .color(self.app_context.theme.foreground),
            );
        });
        user_interface.add_space(6.0);

        for symbol_tree_entry in symbol_tree_entries {
            let is_selected = matches!(
                selected_entry,
                Some(SymbolTreeSelection::SymbolClaim(selected_symbol_locator_key))
                    if !Self::is_module_field_tree_entry(symbol_tree_entry)
                        && matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } if selected_symbol_locator_key == symbol_locator_key)
            ) || matches!(
                selected_entry,
                Some(SymbolTreeSelection::DerivedNode(selected_node_key)) if selected_node_key == symbol_tree_entry.get_node_key()
            );

            user_interface.horizontal(|user_interface| {
                user_interface.add_space(symbol_tree_entry.get_depth() as f32 * 16.0);

                if symbol_tree_entry.can_expand() {
                    let expansion_label = if symbol_tree_entry.is_expanded() { "▾" } else { "▸" };

                    if self
                        .draw_text_button(user_interface, expansion_label, self.app_context.theme.background_control_secondary, true, 24.0)
                        .clicked()
                    {
                        SymbolTreeViewData::toggle_tree_node_expansion(self.symbol_tree_view_data.clone(), symbol_tree_entry.get_node_key());
                    }
                } else {
                    user_interface.add_space(24.0);
                }

                let row_label = format!(
                    "{}  [{}{}]",
                    symbol_tree_entry.get_display_name(),
                    symbol_tree_entry.get_symbol_type_id(),
                    symbol_tree_entry.get_container_type()
                );
                let response = user_interface.selectable_label(is_selected, RichText::new(row_label).color(self.app_context.theme.foreground));

                if response.clicked() {
                    if let Some(selection) = Self::build_selection_for_tree_entry(symbol_tree_entry) {
                        SymbolTreeViewData::set_selected_entry(self.symbol_tree_view_data.clone(), Some(selection));
                    }
                }
            });

            user_interface.label(
                RichText::new(symbol_tree_entry.get_locator().to_string())
                    .font(
                        self.app_context
                            .theme
                            .font_library
                            .font_noto_sans
                            .font_small
                            .clone(),
                    )
                    .color(self.app_context.theme.foreground_preview),
            );
            user_interface.add_space(6.0);
        }
    }

    fn render_symbol_tree_list(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeNode],
        preview_values_by_node_key: &HashMap<String, String>,
        selected_entry: Option<&SymbolTreeSelection>,
        inline_rename_tree_node_key: Option<&str>,
        context_menu_target: Option<&SymbolTreeContextMenuTarget>,
        shared_struct_viewer_focus_target: Option<&StructViewerFocusTarget>,
        allow_interaction: bool,
    ) {
        for symbol_tree_entry in symbol_tree_entries {
            let is_locally_selected = matches!(
                selected_entry,
                Some(SymbolTreeSelection::ModuleRoot(selected_module_name))
                    if matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { module_name, .. } if selected_module_name == module_name)
            ) || matches!(
                selected_entry,
                Some(SymbolTreeSelection::SymbolClaim(selected_symbol_locator_key))
                    if !Self::is_module_field_tree_entry(symbol_tree_entry)
                        && matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } if selected_symbol_locator_key == symbol_locator_key)
            ) || matches!(
                selected_entry,
                Some(SymbolTreeSelection::DerivedNode(selected_node_key)) if selected_node_key == symbol_tree_entry.get_node_key()
            );
            let is_selected = is_locally_selected
                && (matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. })
                    || Self::is_symbol_tree_entry_struct_viewer_focused(symbol_tree_entry, shared_struct_viewer_focus_target));

            let is_inline_rename_row = inline_rename_tree_node_key
                .is_some_and(|active_inline_rename_tree_node_key| symbol_tree_entry.get_node_key() == active_inline_rename_tree_node_key);

            if is_inline_rename_row {
                let rename_target_key = symbol_tree_entry.get_node_key();
                let rename_text_storage_id = Self::inline_rename_text_storage_id(rename_target_key);
                let rename_highlight_storage_id = Self::inline_rename_highlight_storage_id(rename_target_key);
                let mut rename_text = user_interface
                    .ctx()
                    .data_mut(|data| data.get_temp::<String>(rename_text_storage_id))
                    .unwrap_or_else(|| symbol_tree_entry.get_display_name().to_string());
                let mut should_highlight_text = user_interface
                    .ctx()
                    .data_mut(|data| data.get_temp::<bool>(rename_highlight_storage_id))
                    .unwrap_or(true);
                let inline_rename_response = SymbolTreeInlineRenameView::new(
                    self.app_context.clone(),
                    rename_target_key,
                    symbol_tree_entry,
                    &mut rename_text,
                    &mut should_highlight_text,
                    is_selected,
                )
                .show(user_interface);

                if inline_rename_response.should_commit {
                    let trimmed_rename_text = rename_text.trim().to_string();

                    if !trimmed_rename_text.is_empty() && trimmed_rename_text != symbol_tree_entry.get_display_name() {
                        match symbol_tree_entry.get_kind() {
                            SymbolTreeNodeKind::ModuleSpace { module_name, .. } => self.rename_module_root(module_name, trimmed_rename_text),
                            SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } => self.rename_symbol_claim(symbol_locator_key, trimmed_rename_text),
                            _ => {}
                        }
                    }

                    self.clear_inline_rename_state(user_interface, rename_target_key);
                }

                if inline_rename_response.should_cancel {
                    self.clear_inline_rename_state(user_interface, rename_target_key);
                }

                user_interface.ctx().data_mut(|data| {
                    data.insert_temp(rename_text_storage_id, rename_text);
                    data.insert_temp(rename_highlight_storage_id, should_highlight_text);
                });

                continue;
            }

            let preview_value = preview_values_by_node_key
                .get(symbol_tree_entry.get_node_key())
                .map(String::as_str)
                .unwrap_or("");
            let size_in_bytes = resolve_symbol_tree_node_size_in_bytes(project_symbol_catalog, symbol_tree_entry, |data_type_ref| {
                self.app_context
                    .engine_unprivileged_state
                    .get_default_value(data_type_ref)
                    .map(|default_value| default_value.get_size_in_bytes())
            });
            let size_preview_text = Self::format_symbol_tree_size_preview(size_in_bytes);
            let size_tooltip_text = Self::format_symbol_tree_size_tooltip(size_in_bytes);
            let symbol_tree_entry_view_response = SymbolTreeEntryView::new(
                self.app_context.clone(),
                symbol_tree_entry,
                &size_preview_text,
                &size_tooltip_text,
                preview_value,
                is_selected,
            )
            .show(user_interface);

            if allow_interaction && symbol_tree_entry_view_response.did_click_expand_arrow {
                if let Some(selection) = Self::build_selection_for_tree_entry(symbol_tree_entry) {
                    SymbolTreeViewData::set_selected_entry(self.symbol_tree_view_data.clone(), Some(selection));
                }

                SymbolTreeViewData::toggle_tree_node_expansion(self.symbol_tree_view_data.clone(), symbol_tree_entry.get_node_key());
            }

            if allow_interaction && symbol_tree_entry_view_response.row_response.double_clicked() && !symbol_tree_entry_view_response.did_click_expand_arrow {
                if let Some(selection) = Self::build_selection_for_tree_entry(symbol_tree_entry) {
                    SymbolTreeViewData::set_selected_entry(self.symbol_tree_view_data.clone(), Some(selection));
                }

                if !matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. }) {
                    self.focus_symbol_tree_entry_in_struct_viewer(project_symbol_catalog, symbol_tree_entry);
                }

                continue;
            }

            if allow_interaction && symbol_tree_entry_view_response.did_click_row {
                let Some(selection) = Self::build_selection_for_tree_entry(symbol_tree_entry) else {
                    continue;
                };

                SymbolTreeViewData::set_selected_entry(self.symbol_tree_view_data.clone(), Some(selection));
                if !matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. }) {
                    self.focus_symbol_tree_entry_in_struct_viewer(project_symbol_catalog, symbol_tree_entry);
                }
            }

            if allow_interaction && symbol_tree_entry_view_response.row_response.secondary_clicked() {
                let Some(selection) = Self::build_selection_for_tree_entry(symbol_tree_entry) else {
                    continue;
                };
                let context_menu_position = symbol_tree_entry_view_response
                    .row_response
                    .interact_pointer_pos()
                    .unwrap_or(symbol_tree_entry_view_response.row_response.rect.left_top());

                SymbolTreeViewData::set_selected_entry(self.symbol_tree_view_data.clone(), Some(selection));
                if !matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. }) {
                    self.focus_symbol_tree_entry_in_struct_viewer(project_symbol_catalog, symbol_tree_entry);
                }
                SymbolTreeViewData::show_context_menu(
                    self.symbol_tree_view_data.clone(),
                    SymbolTreeContextMenuTarget::new(symbol_tree_entry.get_node_key().to_string(), context_menu_position),
                );
            }

            if allow_interaction
                && context_menu_target
                    .as_ref()
                    .is_some_and(|context_menu_target| context_menu_target.get_tree_node_key() == symbol_tree_entry.get_node_key())
            {
                let can_open_symbol_tree_entry = !matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. });
                let can_rename_symbol_tree_entry = matches!(
                    symbol_tree_entry.get_kind(),
                    SymbolTreeNodeKind::ModuleSpace { .. } | SymbolTreeNodeKind::SymbolClaim { .. }
                );
                let context_menu_symbol_claim = match symbol_tree_entry.get_kind() {
                    SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } => project_symbol_catalog
                        .get_symbol_claims()
                        .iter()
                        .find(|symbol_claim| symbol_claim.get_symbol_locator_key() == *symbol_locator_key),
                    _ => None,
                };
                let context_menu_module_name = match symbol_tree_entry.get_kind() {
                    SymbolTreeNodeKind::ModuleSpace { module_name, .. } => Some(module_name.as_str()),
                    _ => None,
                };
                let context_menu_module_child_range_target = build_module_child_range_target(project_symbol_catalog, symbol_tree_entry, |data_type_ref| {
                    self.app_context
                        .engine_unprivileged_state
                        .get_default_value(data_type_ref)
                        .map(|default_value| default_value.get_size_in_bytes())
                });
                let context_menu_add_symbol_to_project_target = build_add_symbol_to_project_target(symbol_tree_entry);
                let context_menu_symbol_layout_edit_target = build_symbol_layout_edit_target(project_symbol_catalog, symbol_tree_entries, symbol_tree_entry);
                let context_menu_symbol_tree_action_context = Self::build_symbol_tree_action_context(symbol_tree_entry);
                let context_menu_plugin_action_menu_items = self.build_symbol_tree_plugin_action_menu_items(&context_menu_symbol_tree_action_context);
                let can_delete_symbol_tree_entry =
                    context_menu_module_child_range_target.is_some() || context_menu_symbol_claim.is_some() || context_menu_module_name.is_some();
                let mut context_menu_labels = Vec::new();
                if can_open_symbol_tree_entry {
                    context_menu_labels.push(Self::SYMBOL_TREE_CTX_OPEN_MEMORY_VIEWER_LABEL.to_string());
                    context_menu_labels.push(Self::SYMBOL_TREE_CTX_OPEN_CODE_VIEWER_LABEL.to_string());
                }
                if context_menu_add_symbol_to_project_target.is_some() {
                    context_menu_labels.push(Self::SYMBOL_TREE_CTX_ADD_TO_PROJECT_LABEL.to_string());
                }
                if context_menu_symbol_layout_edit_target.is_some() {
                    context_menu_labels.push(Self::SYMBOL_TREE_CTX_EDIT_SYMBOL_LAYOUT_LABEL.to_string());
                }
                if can_rename_symbol_tree_entry {
                    context_menu_labels.push(Self::SYMBOL_TREE_CTX_RENAME_LABEL.to_string());
                }
                context_menu_labels.extend(
                    context_menu_plugin_action_menu_items
                        .iter()
                        .map(|menu_item| menu_item.label.clone()),
                );
                context_menu_labels.push(Self::SYMBOL_TREE_CTX_NEW_MODULE_LABEL.to_string());
                if can_delete_symbol_tree_entry {
                    context_menu_labels.push(Self::SYMBOL_TREE_CTX_DELETE_LABEL.to_string());
                }
                let context_menu_width =
                    ContextMenuSizing::width_for_labels(self.app_context.as_ref(), user_interface, context_menu_labels.iter().map(String::as_str));
                let mut is_context_menu_open = true;

                ContextMenu::new(
                    self.app_context.clone(),
                    "symbol_tree_context_menu",
                    context_menu_target
                        .as_ref()
                        .map(|context_menu_target| context_menu_target.get_position())
                        .unwrap_or(symbol_tree_entry_view_response.row_response.rect.left_top()),
                    |user_interface, should_close| {
                        if can_open_symbol_tree_entry {
                            if user_interface
                                .add(
                                    ToolbarMenuItemView::new(
                                        self.app_context.clone(),
                                        Self::SYMBOL_TREE_CTX_OPEN_MEMORY_VIEWER_LABEL,
                                        Self::SYMBOL_TREE_CTX_OPEN_MEMORY_VIEWER_ID,
                                        &None,
                                        context_menu_width,
                                    )
                                    .icon(
                                        self.app_context
                                            .theme
                                            .icon_library
                                            .icon_handle_scan_collect_values
                                            .clone(),
                                    ),
                                )
                                .clicked()
                            {
                                self.focus_memory_viewer_for_locator(symbol_tree_entry.get_locator());
                                *should_close = true;
                            }

                            if user_interface
                                .add(
                                    ToolbarMenuItemView::new(
                                        self.app_context.clone(),
                                        Self::SYMBOL_TREE_CTX_OPEN_CODE_VIEWER_LABEL,
                                        Self::SYMBOL_TREE_CTX_OPEN_CODE_VIEWER_ID,
                                        &None,
                                        context_menu_width,
                                    )
                                    .icon(
                                        self.app_context
                                            .theme
                                            .icon_library
                                            .icon_handle_project_cpu_instruction
                                            .clone(),
                                    ),
                                )
                                .clicked()
                            {
                                self.focus_code_viewer_for_locator(symbol_tree_entry.get_locator());
                                *should_close = true;
                            }
                        }

                        if let Some(add_symbol_to_project_target) = context_menu_add_symbol_to_project_target.as_ref() {
                            if user_interface
                                .add(
                                    ToolbarMenuItemView::new(
                                        self.app_context.clone(),
                                        Self::SYMBOL_TREE_CTX_ADD_TO_PROJECT_LABEL,
                                        Self::SYMBOL_TREE_CTX_ADD_TO_PROJECT_ID,
                                        &None,
                                        context_menu_width,
                                    )
                                    .icon(
                                        self.app_context
                                            .theme
                                            .icon_library
                                            .icon_handle_common_add
                                            .clone(),
                                    ),
                                )
                                .clicked()
                            {
                                self.add_symbol_to_project(add_symbol_to_project_target);
                                *should_close = true;
                            }
                        }

                        if (can_open_symbol_tree_entry || context_menu_add_symbol_to_project_target.is_some())
                            && (context_menu_symbol_layout_edit_target.is_some() || can_rename_symbol_tree_entry)
                        {
                            user_interface.separator();
                        }

                        if let Some(struct_layout_id) = context_menu_symbol_layout_edit_target.as_deref() {
                            if user_interface
                                .add(
                                    ToolbarMenuItemView::new(
                                        self.app_context.clone(),
                                        Self::SYMBOL_TREE_CTX_EDIT_SYMBOL_LAYOUT_LABEL,
                                        Self::SYMBOL_TREE_CTX_EDIT_SYMBOL_LAYOUT_ID,
                                        &None,
                                        context_menu_width,
                                    )
                                    .icon(
                                        self.app_context
                                            .theme
                                            .icon_library
                                            .icon_handle_common_edit
                                            .clone(),
                                    ),
                                )
                                .clicked()
                            {
                                self.edit_symbol_tree_entry_symbol_layout(project_symbol_catalog, struct_layout_id);
                                *should_close = true;
                            }
                        }

                        if context_menu_symbol_layout_edit_target.is_some() && can_rename_symbol_tree_entry {
                            user_interface.separator();
                        }

                        if can_rename_symbol_tree_entry
                            && user_interface
                                .add(
                                    ToolbarMenuItemView::new(
                                        self.app_context.clone(),
                                        Self::SYMBOL_TREE_CTX_RENAME_LABEL,
                                        Self::SYMBOL_TREE_CTX_RENAME_ID,
                                        &None,
                                        context_menu_width,
                                    )
                                    .icon(
                                        self.app_context
                                            .theme
                                            .icon_library
                                            .icon_handle_common_edit
                                            .clone(),
                                    ),
                                )
                                .clicked()
                        {
                            SymbolTreeViewData::begin_inline_rename(self.symbol_tree_view_data.clone(), symbol_tree_entry.get_node_key().to_string());
                            *should_close = true;
                        }

                        let has_symbol_tree_edit_menu_items =
                            can_open_symbol_tree_entry || context_menu_symbol_layout_edit_target.is_some() || can_rename_symbol_tree_entry;

                        if has_symbol_tree_edit_menu_items && !context_menu_plugin_action_menu_items.is_empty() {
                            user_interface.separator();
                        }

                        for plugin_action_menu_item in &context_menu_plugin_action_menu_items {
                            let plugin_action_widget_id = format!(
                                "symbol_tree_ctx_plugin_action_{}_{}",
                                plugin_action_menu_item.plugin_id, plugin_action_menu_item.action_id
                            );

                            if user_interface
                                .add(
                                    ToolbarMenuItemView::new(
                                        self.app_context.clone(),
                                        &plugin_action_menu_item.label,
                                        &plugin_action_widget_id,
                                        &None,
                                        context_menu_width,
                                    )
                                    .icon(
                                        self.app_context
                                            .theme
                                            .icon_library
                                            .icon_handle_data_type_blue_blocks_4
                                            .clone(),
                                    ),
                                )
                                .clicked()
                            {
                                self.execute_symbol_tree_plugin_action(plugin_action_menu_item, context_menu_symbol_tree_action_context.clone());
                                *should_close = true;
                            }
                        }

                        if has_symbol_tree_edit_menu_items || !context_menu_plugin_action_menu_items.is_empty() {
                            user_interface.separator();
                        }

                        if user_interface
                            .add(
                                ToolbarMenuItemView::new(
                                    self.app_context.clone(),
                                    Self::SYMBOL_TREE_CTX_NEW_MODULE_LABEL,
                                    Self::SYMBOL_TREE_CTX_NEW_MODULE_ID,
                                    &None,
                                    context_menu_width,
                                )
                                .icon(
                                    self.app_context
                                        .theme
                                        .icon_library
                                        .icon_handle_common_add
                                        .clone(),
                                ),
                            )
                            .clicked()
                        {
                            SymbolTreeViewData::begin_create_module_root(self.symbol_tree_view_data.clone());
                            *should_close = true;
                        }

                        if can_delete_symbol_tree_entry {
                            user_interface.separator();

                            if user_interface
                                .add(
                                    ToolbarMenuItemView::new(
                                        self.app_context.clone(),
                                        Self::SYMBOL_TREE_CTX_DELETE_LABEL,
                                        Self::SYMBOL_TREE_CTX_DELETE_ID,
                                        &None,
                                        context_menu_width,
                                    )
                                    .icon(
                                        self.app_context
                                            .theme
                                            .icon_library
                                            .icon_handle_common_delete
                                            .clone(),
                                    )
                                    .icon_background(
                                        self.app_context.theme.background_control_danger,
                                        self.app_context.theme.background_control_danger_dark,
                                    ),
                                )
                                .clicked()
                            {
                                self.request_delete_for_selection(
                                    context_menu_symbol_claim,
                                    context_menu_module_name,
                                    context_menu_module_child_range_target.as_ref(),
                                );
                                *should_close = true;
                            }
                        }
                    },
                )
                .width(context_menu_width)
                .corner_radius(8)
                .show(user_interface, &mut is_context_menu_open);

                if !is_context_menu_open {
                    SymbolTreeViewData::hide_context_menu(self.symbol_tree_view_data.clone());
                }
            }
        }
    }

    fn build_selection_for_tree_entry(symbol_tree_entry: &SymbolTreeNode) -> Option<SymbolTreeSelection> {
        match symbol_tree_entry.get_kind() {
            SymbolTreeNodeKind::ModuleSpace { module_name, .. } => Some(SymbolTreeSelection::ModuleRoot(module_name.to_string())),
            SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } => {
                if Self::is_module_field_tree_entry(symbol_tree_entry) {
                    Some(SymbolTreeSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string()))
                } else {
                    Some(SymbolTreeSelection::SymbolClaim(symbol_locator_key.to_string()))
                }
            }
            SymbolTreeNodeKind::StructField | SymbolTreeNodeKind::UnassignedSegment { .. } | SymbolTreeNodeKind::PointerTarget => {
                Some(SymbolTreeSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string()))
            }
        }
    }
}

impl Widget for SymbolTreeView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let Some(project_symbol_catalog) = self.get_opened_project_symbol_catalog() else {
            self.app_context
                .engine_unprivileged_state
                .set_virtual_snapshot_queries(Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID, Self::POINTER_CHILDREN_REFRESH_INTERVAL, Vec::new());
            self.app_context
                .engine_unprivileged_state
                .set_virtual_snapshot_queries(Self::SCALAR_VALUES_VIRTUAL_SNAPSHOT_ID, Self::SCALAR_VALUES_REFRESH_INTERVAL, Vec::new());
            self.app_context
                .engine_unprivileged_state
                .set_virtual_snapshot_queries(Self::PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID, Self::PREVIEW_VALUES_REFRESH_INTERVAL, Vec::new());

            return user_interface
                .allocate_ui_with_layout(
                    user_interface.available_size(),
                    Layout::centered_and_justified(Direction::TopDown),
                    |user_interface| {
                        user_interface.label(RichText::new("Open a project to browse the Symbol Tree.").color(self.app_context.theme.foreground_preview));
                    },
                )
                .response;
        };

        let shared_struct_viewer_focus_target = self
            .struct_viewer_view_data
            .read("Symbol tree shared struct viewer focus target")
            .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned());
        let suppress_default_selection = matches!(shared_struct_viewer_focus_target, Some(StructViewerFocusTarget::ProjectHierarchy { .. }));

        SymbolTreeViewData::synchronize_selection(self.symbol_tree_view_data.clone(), &project_symbol_catalog, suppress_default_selection);
        SymbolTreeViewData::synchronize_inline_rename(self.symbol_tree_view_data.clone(), &project_symbol_catalog);
        SymbolTreeViewData::synchronize_take_over_state(self.symbol_tree_view_data.clone(), &project_symbol_catalog);
        let expanded_tree_node_keys = self
            .symbol_tree_view_data
            .read("Symbol tree expanded tree nodes")
            .map(|symbol_tree_view_data| symbol_tree_view_data.get_expanded_tree_node_keys().clone())
            .unwrap_or_default();
        let scalar_values_by_query_id = self.collect_scalar_values_by_query_id();
        let scalar_snapshot_queries = RefCell::new(Vec::new());
        let resolve_primitive_size_in_bytes = |data_type_ref: &DataTypeRef| {
            self.app_context
                .engine_unprivileged_state
                .get_default_value(data_type_ref)
                .map(|default_value| default_value.get_size_in_bytes())
        };
        let read_scalar_field = |project_symbol_locator: &ProjectSymbolLocator, field_definition: &SymbolicFieldDefinition, field_size_in_bytes: u64| {
            let scalar_query_id = SymbolTreeScalarValue::query_id(project_symbol_locator, field_definition);

            if let Some(scalar_snapshot_query) =
                SymbolTreeScalarValue::build_query(project_symbol_locator, field_definition, field_size_in_bytes, |data_type_ref| {
                    self.app_context
                        .engine_unprivileged_state
                        .supports_scalar_integer_values(data_type_ref)
                })
            {
                scalar_snapshot_queries.borrow_mut().push(scalar_snapshot_query);
            }

            if let Some(scalar_value) = scalar_values_by_query_id.get(&scalar_query_id) {
                return Ok(Some(*scalar_value));
            }

            Ok(None)
        };
        let previous_resolved_pointer_targets_by_node_key = self.collect_resolved_pointer_targets_by_node_key();
        let resolver_pointer_snapshot_queries = RefCell::new(Vec::new());
        let resolve_relative_pointer_chain = |root_locator: &ProjectSymbolLocator, pointer_chain: &SymbolicPointerChain| {
            Self::resolve_relative_pointer_chain_from_pointer_snapshot(
                &previous_resolved_pointer_targets_by_node_key,
                &resolver_pointer_snapshot_queries,
                root_locator,
                pointer_chain,
            )
        };
        let resolve_global_pointer_chain = |pointer_chain: &SymbolicPointerChain| {
            Self::resolve_global_pointer_chain_from_pointer_snapshot(
                &project_symbol_catalog,
                &previous_resolved_pointer_targets_by_node_key,
                &resolver_pointer_snapshot_queries,
                pointer_chain,
            )
        };
        let structural_symbol_tree_entries = SymbolTree::build_with_scalar_reader_and_pointer_chains(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &HashMap::new(),
            resolve_primitive_size_in_bytes,
            read_scalar_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
        )
        .into_nodes();
        self.sync_symbol_scalar_virtual_snapshot(scalar_snapshot_queries.borrow().clone());
        self.sync_pointer_child_virtual_snapshot(
            &project_symbol_catalog,
            &structural_symbol_tree_entries,
            resolver_pointer_snapshot_queries.borrow().clone(),
        );
        let resolved_pointer_targets_by_node_key = self.collect_resolved_pointer_targets_by_node_key();
        let resolve_relative_pointer_chain = |root_locator: &ProjectSymbolLocator, pointer_chain: &SymbolicPointerChain| {
            Self::resolve_relative_pointer_chain_from_pointer_snapshot(
                &resolved_pointer_targets_by_node_key,
                &resolver_pointer_snapshot_queries,
                root_locator,
                pointer_chain,
            )
        };
        let resolve_global_pointer_chain = |pointer_chain: &SymbolicPointerChain| {
            Self::resolve_global_pointer_chain_from_pointer_snapshot(
                &project_symbol_catalog,
                &resolved_pointer_targets_by_node_key,
                &resolver_pointer_snapshot_queries,
                pointer_chain,
            )
        };
        let symbol_tree_entries = SymbolTree::build_with_scalar_reader_and_pointer_chains(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
            read_scalar_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
        )
        .into_nodes();
        self.sync_symbol_scalar_virtual_snapshot(scalar_snapshot_queries.borrow().clone());
        self.sync_pointer_child_virtual_snapshot(
            &project_symbol_catalog,
            &symbol_tree_entries,
            resolver_pointer_snapshot_queries.borrow().clone(),
        );
        self.sync_symbol_preview_virtual_snapshot(&project_symbol_catalog, &symbol_tree_entries);
        let preview_values_by_node_key = self.collect_preview_values_by_node_key(&symbol_tree_entries);
        SymbolTreeViewData::synchronize_selection_to_tree_entries(self.symbol_tree_view_data.clone(), &symbol_tree_entries);
        let (selected_entry, take_over_state, inline_rename_tree_node_key, context_menu_target, current_module_root_create_draft, current_define_field_draft) =
            self.symbol_tree_view_data
                .read("Symbol tree view")
                .map(|symbol_tree_view_data| {
                    (
                        symbol_tree_view_data.get_selected_entry().cloned(),
                        symbol_tree_view_data.get_take_over_state().cloned(),
                        symbol_tree_view_data
                            .get_inline_rename_tree_node_key()
                            .map(str::to_string),
                        symbol_tree_view_data.get_context_menu_target().cloned(),
                        symbol_tree_view_data.get_module_root_create_draft().clone(),
                        symbol_tree_view_data.get_define_field_draft().clone(),
                    )
                })
                .unwrap_or((None, None, None, None, Default::default(), Default::default()));
        let selected_symbol_claim = match selected_entry.as_ref() {
            Some(SymbolTreeSelection::SymbolClaim(selected_symbol_locator_key)) => project_symbol_catalog
                .get_symbol_claims()
                .iter()
                .find(|symbol_claim| symbol_claim.get_symbol_locator_key() == *selected_symbol_locator_key),
            _ => None,
        };
        let selected_module_name = match selected_entry.as_ref() {
            Some(SymbolTreeSelection::ModuleRoot(module_name)) if project_symbol_catalog.find_symbol_module(module_name).is_some() => {
                Some(module_name.to_string())
            }
            _ => None,
        };
        let selected_symbol_tree_entry = Self::build_selected_symbol_tree_entry(&symbol_tree_entries, selected_entry.as_ref());
        let selected_module_child_range_target = selected_symbol_tree_entry.and_then(|symbol_tree_entry| {
            build_module_child_range_target(&project_symbol_catalog, symbol_tree_entry, |data_type_ref| {
                self.app_context
                    .engine_unprivileged_state
                    .get_default_value(data_type_ref)
                    .map(|default_value| default_value.get_size_in_bytes())
            })
        });
        let create_module_root_request = match selected_entry.as_ref() {
            Some(SymbolTreeSelection::CreateModuleRoot) => {
                SymbolTreeModuleCreateView::build_module_root_create_request_from_draft(&current_module_root_create_draft)
            }
            _ => None,
        };
        self.sync_selected_symbol_into_struct_viewer(&project_symbol_catalog, selected_symbol_tree_entry);
        let theme = self.app_context.theme.clone();
        let is_delete_confirmation_active = take_over_state.is_some();
        let is_inline_rename_active = inline_rename_tree_node_key.is_some();
        let is_create_module_root_active = matches!(selected_entry.as_ref(), Some(SymbolTreeSelection::CreateModuleRoot));
        let can_use_standard_toolbar_actions = !is_delete_confirmation_active && !is_inline_rename_active && !is_create_module_root_active;
        let is_window_focused = self
            .app_context
            .window_focus_manager
            .is_window_focused(Self::WINDOW_ID);
        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID);

        if is_window_focused && is_delete_confirmation_active && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            match take_over_state.as_ref() {
                Some(SymbolTreeTakeOverState::DeleteSymbolClaimConfirmation { symbol_locator_key, .. }) => self.delete_symbol_claim(symbol_locator_key),
                Some(SymbolTreeTakeOverState::DeleteModuleRootConfirmation { module_name }) => self.delete_module_root(module_name),
                Some(SymbolTreeTakeOverState::DeleteModuleRangeConfirmation {
                    module_name,
                    offset,
                    length,
                    mode,
                    ..
                }) => self.delete_module_range(module_name, *offset, *length, *mode),
                Some(SymbolTreeTakeOverState::DefineFieldFromUnassignedSegment { .. }) => {}
                None => {}
            }
        }

        if is_window_focused && is_delete_confirmation_active && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) {
            SymbolTreeViewData::cancel_take_over_state(self.symbol_tree_view_data.clone());
        }

        if is_window_focused && is_create_module_root_active && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) {
            SymbolTreeViewData::set_selected_entry(self.symbol_tree_view_data.clone(), None);
        }

        if is_window_focused && is_create_module_root_active && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            if let Some(project_symbols_create_module_request) = create_module_root_request.clone() {
                self.create_module_root(project_symbols_create_module_request);
            }
        }

        if can_use_standard_toolbar_actions && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowUp)) {
            if let Some(next_symbol_tree_entry) =
                Self::resolve_adjacent_symbol_tree_entry(&symbol_tree_entries, selected_entry.as_ref(), ListNavigationDirection::Up)
            {
                if let Some(selection) = Self::build_selection_for_tree_entry(next_symbol_tree_entry) {
                    SymbolTreeViewData::set_selected_entry(self.symbol_tree_view_data.clone(), Some(selection));

                    if !matches!(next_symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. }) {
                        self.focus_symbol_tree_entry_in_struct_viewer(&project_symbol_catalog, next_symbol_tree_entry);
                    }
                }
            }
        }

        if can_use_standard_toolbar_actions && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowDown)) {
            if let Some(next_symbol_tree_entry) =
                Self::resolve_adjacent_symbol_tree_entry(&symbol_tree_entries, selected_entry.as_ref(), ListNavigationDirection::Down)
            {
                if let Some(selection) = Self::build_selection_for_tree_entry(next_symbol_tree_entry) {
                    SymbolTreeViewData::set_selected_entry(self.symbol_tree_view_data.clone(), Some(selection));

                    if !matches!(next_symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. }) {
                        self.focus_symbol_tree_entry_in_struct_viewer(&project_symbol_catalog, next_symbol_tree_entry);
                    }
                }
            }
        }

        if can_use_standard_toolbar_actions && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowRight)) {
            if let Some(selected_symbol_tree_entry) = selected_symbol_tree_entry.filter(|symbol_tree_entry| symbol_tree_entry.can_expand()) {
                SymbolTreeViewData::expand_tree_node(self.symbol_tree_view_data.clone(), selected_symbol_tree_entry.get_node_key());
            }
        }

        if can_use_standard_toolbar_actions && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowLeft)) {
            if let Some(selected_symbol_tree_entry) =
                selected_symbol_tree_entry.filter(|symbol_tree_entry| symbol_tree_entry.can_expand() && symbol_tree_entry.is_expanded())
            {
                SymbolTreeViewData::toggle_tree_node_expansion(self.symbol_tree_view_data.clone(), selected_symbol_tree_entry.get_node_key());
            }
        }

        if !is_delete_confirmation_active
            && !is_inline_rename_active
            && !is_create_module_root_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::Delete))
        {
            self.request_delete_for_selection(
                selected_symbol_claim,
                selected_module_name.as_deref(),
                selected_module_child_range_target.as_ref(),
            );
        }

        let can_rename_selected_entry = selected_symbol_tree_entry.is_some_and(|symbol_tree_entry| {
            matches!(
                symbol_tree_entry.get_kind(),
                SymbolTreeNodeKind::ModuleSpace { .. } | SymbolTreeNodeKind::SymbolClaim { .. }
            )
        });
        if !is_delete_confirmation_active
            && !is_inline_rename_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::F2))
        {
            if can_rename_selected_entry {
                if let Some(symbol_tree_entry) = selected_symbol_tree_entry {
                    SymbolTreeViewData::begin_inline_rename(self.symbol_tree_view_data.clone(), symbol_tree_entry.get_node_key().to_string());
                }
            }
        }

        if is_window_focused && is_inline_rename_active && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) {
            if let Some(active_inline_rename_tree_node_key) = inline_rename_tree_node_key.as_deref() {
                self.clear_inline_rename_state(user_interface, active_inline_rename_tree_node_key);
            }
        }

        user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let mut list_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(user_interface.available_rect_before_wrap())
                        .layout(Layout::top_down(Align::Min)),
                );
                let toolbar_action = if can_use_standard_toolbar_actions {
                    SymbolTreeToolbarView::new(self.app_context.clone())
                        .can_create_module_root(true)
                        .show(&mut list_user_interface)
                } else {
                    None
                };

                match toolbar_action {
                    Some(SymbolTreeToolbarAction::CreateModuleRoot) => {
                        SymbolTreeViewData::begin_create_module_root(self.symbol_tree_view_data.clone());
                    }
                    None => {}
                }

                match take_over_state.as_ref() {
                    Some(SymbolTreeTakeOverState::DeleteSymbolClaimConfirmation {
                        symbol_locator_key,
                        display_name,
                    }) => {
                        let description_text = String::from("This removes the authored symbol from the project.");

                        list_user_interface.add_space(8.0);
                        match SymbolTreeDeleteConfirmationView::new(
                            self.app_context.clone(),
                            "Delete this symbol",
                            display_name,
                            &description_text,
                            false,
                            "Delete",
                        )
                        .show(&mut list_user_interface)
                        {
                            SymbolTreeDeleteConfirmationAction::Confirm => self.delete_symbol_claim(symbol_locator_key),
                            SymbolTreeDeleteConfirmationAction::Cancel => SymbolTreeViewData::cancel_take_over_state(self.symbol_tree_view_data.clone()),
                            SymbolTreeDeleteConfirmationAction::None => {}
                        }

                        return;
                    }
                    Some(SymbolTreeTakeOverState::DeleteModuleRootConfirmation { module_name }) => {
                        let description_text = String::from("This removes the module root and all symbol claims inside it.");

                        list_user_interface.add_space(8.0);
                        match SymbolTreeDeleteConfirmationView::new(
                            self.app_context.clone(),
                            "Delete this module",
                            module_name,
                            &description_text,
                            false,
                            "Delete",
                        )
                        .show(&mut list_user_interface)
                        {
                            SymbolTreeDeleteConfirmationAction::Confirm => self.delete_module_root(module_name),
                            SymbolTreeDeleteConfirmationAction::Cancel => SymbolTreeViewData::cancel_take_over_state(self.symbol_tree_view_data.clone()),
                            SymbolTreeDeleteConfirmationAction::None => {}
                        }

                        return;
                    }
                    Some(SymbolTreeTakeOverState::DeleteModuleRangeConfirmation {
                        module_name,
                        offset,
                        length,
                        display_name,
                        mode,
                    }) => {
                        let delete_confirmation_description = build_delete_module_range_confirmation_description(module_name, *length, *mode);
                        let description_text = delete_confirmation_description.text;

                        list_user_interface.add_space(8.0);
                        match SymbolTreeDeleteConfirmationView::new(
                            self.app_context.clone(),
                            "Delete this field",
                            display_name,
                            &description_text,
                            delete_confirmation_description.is_warning,
                            "Delete",
                        )
                        .show(&mut list_user_interface)
                        {
                            SymbolTreeDeleteConfirmationAction::Confirm => self.delete_module_range(module_name, *offset, *length, *mode),
                            SymbolTreeDeleteConfirmationAction::Cancel => SymbolTreeViewData::cancel_take_over_state(self.symbol_tree_view_data.clone()),
                            SymbolTreeDeleteConfirmationAction::None => {}
                        }

                        return;
                    }
                    Some(SymbolTreeTakeOverState::DefineFieldFromUnassignedSegment {
                        module_name, offset, length, ..
                    }) => {
                        list_user_interface.add_space(8.0);
                        match SymbolTreeDefineFieldView::new(
                            self.app_context.clone(),
                            &project_symbol_catalog,
                            module_name,
                            *offset,
                            *length,
                            &current_define_field_draft,
                        )
                        .show(&mut list_user_interface)
                        {
                            SymbolTreeDefineFieldAction::Cancel => SymbolTreeViewData::cancel_take_over_state(self.symbol_tree_view_data.clone()),
                            SymbolTreeDefineFieldAction::Create(define_field_plan) => {
                                SymbolTreeViewData::cancel_take_over_state(self.symbol_tree_view_data.clone());
                                self.create_define_field_from_unassigned_span_edit_target(module_name, define_field_plan);
                            }
                            SymbolTreeDefineFieldAction::DraftChanged(define_field_draft) => {
                                SymbolTreeViewData::set_define_field_draft(self.symbol_tree_view_data.clone(), define_field_draft);
                            }
                            SymbolTreeDefineFieldAction::None => {}
                        }

                        return;
                    }
                    None => {}
                }

                if matches!(selected_entry.as_ref(), Some(SymbolTreeSelection::CreateModuleRoot)) {
                    list_user_interface.add_space(8.0);
                    match SymbolTreeModuleCreateView::new(self.app_context.clone(), &current_module_root_create_draft).show(&mut list_user_interface) {
                        SymbolTreeModuleCreateAction::Cancel => SymbolTreeViewData::set_selected_entry(self.symbol_tree_view_data.clone(), None),
                        SymbolTreeModuleCreateAction::Create(project_symbols_create_module_request) => {
                            self.create_module_root(project_symbols_create_module_request);
                        }
                        SymbolTreeModuleCreateAction::DraftChanged(module_root_create_draft) => {
                            SymbolTreeViewData::set_module_root_create_draft(self.symbol_tree_view_data.clone(), module_root_create_draft);
                        }
                        SymbolTreeModuleCreateAction::None => {}
                    }

                    return;
                }

                list_user_interface.add_space(8.0);
                ScrollArea::vertical()
                    .id_salt("symbol_tree_list")
                    .auto_shrink([false, false])
                    .show(&mut list_user_interface, |user_interface| {
                        self.render_symbol_tree_list(
                            user_interface,
                            &project_symbol_catalog,
                            &symbol_tree_entries,
                            &preview_values_by_node_key,
                            selected_entry.as_ref(),
                            inline_rename_tree_node_key.as_deref(),
                            context_menu_target.as_ref(),
                            shared_struct_viewer_focus_target.as_ref(),
                            !is_inline_rename_active,
                        );
                        if project_symbol_catalog.is_empty() {
                            user_interface.add_space(12.0);
                            user_interface.horizontal(|user_interface| {
                                user_interface.add_space(Self::SYMBOL_TREE_TEXT_PADDING_X);
                                user_interface.label(
                                    RichText::new("This project has no authored symbols yet.")
                                        .font(theme.font_library.font_noto_sans.font_normal.clone())
                                        .color(theme.foreground_preview),
                                );
                            });
                        }
                    });
            })
            .response
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolTreeView;
    use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
    use crate::views::struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData};
    use crate::views::symbol_tree::symbol_tree_define_field_view::{ModuleFieldTypeOption, ModuleFieldTypeOptionKind, SymbolTreeDefineFieldView};
    use crate::views::symbol_tree::symbol_tree_module_create_view::SymbolTreeModuleCreateView;
    use crate::views::symbol_tree::view_data::symbol_tree_view_data::{DefineFieldDraft, ModuleRootCreateDraft};
    use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteModuleRangeMode;
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::projects::symbol_tree::operations::{
        add_symbol_to_project::{build_add_symbol_project_item_create_request, build_add_symbol_to_project_target},
        define_field::{filter_registered_pointer_sizes, parse_define_field_relative_offset},
        delete_symbol::{build_delete_module_range_confirmation_description, build_module_child_range_target},
        edit_symbol_layout::build_symbol_layout_edit_target,
    };
    use squalr_engine_api::structures::projects::symbol_tree::symbol_tree_node::{SymbolTreeNode, SymbolTreeNodeKind};
    use squalr_engine_api::structures::{
        data_types::{built_in_types::u32::data_type_u32::DataTypeU32, data_type_ref::DataTypeRef},
        data_values::{anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
        memory::{
            pointer_chain_segment::PointerChainSegment,
            symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink},
        },
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        projects::{
            project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress, project_symbol_catalog::ProjectSymbolCatalog,
            project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator, project_symbol_module::ProjectSymbolModule,
            project_symbol_module_field::ProjectSymbolModuleField,
        },
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition, valued_struct::ValuedStruct},
    };
    use squalr_engine_session::virtual_snapshots::virtual_snapshot_query::VirtualSnapshotQuery;

    fn create_symbol_claim_tree_entry(
        display_name: &str,
        symbol_type_id: &str,
    ) -> SymbolTreeNode {
        SymbolTreeNode::new(
            String::from("claim:absolute:1234"),
            SymbolTreeNodeKind::SymbolClaim {
                symbol_locator_key: String::from("absolute:1234"),
            },
            1,
            display_name.to_string(),
            display_name.to_string(),
            String::from("absolute:1234"),
            ProjectSymbolLocator::new_absolute_address(0x1234),
            symbol_type_id.to_string(),
            ContainerType::None,
            false,
            false,
        )
    }

    fn create_module_tree_entry(module_name: &str) -> SymbolTreeNode {
        SymbolTreeNode::new(
            format!("module:{}", module_name),
            SymbolTreeNodeKind::ModuleSpace {
                module_name: module_name.to_string(),
                size: 0x2000,
            },
            0,
            module_name.to_string(),
            module_name.to_string(),
            String::new(),
            ProjectSymbolLocator::new_absolute_address(0),
            String::from("u8"),
            ContainerType::ArrayFixed(0x2000),
            true,
            false,
        )
    }

    fn create_module_symbol_claim_tree_entry() -> SymbolTreeNode {
        SymbolTreeNode::new(
            String::from("claim:module:game.exe:4"),
            SymbolTreeNodeKind::SymbolClaim {
                symbol_locator_key: String::from("module:game.exe:4"),
            },
            1,
            String::from("Health"),
            String::from("game.exe.Health"),
            String::from("module:game.exe:4"),
            ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x04),
            String::from("u32"),
            ContainerType::None,
            false,
            false,
        )
    }

    fn create_unassigned_segment_tree_entry() -> SymbolTreeNode {
        SymbolTreeNode::new(
            String::from("unassigned:game.exe:0:1234"),
            SymbolTreeNodeKind::UnassignedSegment {
                module_name: String::from("game.exe"),
                offset: 0,
                length: 0x1234,
            },
            1,
            String::from("UNASSIGNED_00000000"),
            String::from("game.exe.UNASSIGNED_00000000"),
            String::new(),
            ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0),
            String::from("UNASSIGNED"),
            ContainerType::ArrayFixed(0x1234),
            false,
            false,
        )
    }

    fn create_struct_field_tree_entry() -> SymbolTreeNode {
        SymbolTreeNode::new(
            String::from("module_field:module:game.exe:0::NTHeaders::FileHeader"),
            SymbolTreeNodeKind::StructField,
            3,
            String::from("FileHeader"),
            String::from("PE Headers.NTHeaders.FileHeader"),
            String::from("module:game.exe:0"),
            ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x84),
            String::from("win.pe.IMAGE_FILE_HEADER"),
            ContainerType::None,
            true,
            false,
        )
    }

    fn create_fixed_array_symbol_claim_tree_entry(
        data_type_id: &str,
        array_length: u64,
    ) -> SymbolTreeNode {
        SymbolTreeNode::new(
            format!("claim:module:game.exe:40:{}", data_type_id),
            SymbolTreeNodeKind::SymbolClaim {
                symbol_locator_key: String::from("module:game.exe:40"),
            },
            1,
            format!("{}_array", data_type_id),
            format!("game.exe.{}_array", data_type_id),
            String::from("module:game.exe:40"),
            ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x40),
            data_type_id.to_string(),
            ContainerType::ArrayFixed(array_length),
            false,
            false,
        )
    }

    #[test]
    fn build_global_pointer_chain_virtual_snapshot_query_resolves_symbolic_links() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x1000);

        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Globals"), 0x80, String::from("globals")));

        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let pointer_chain = SymbolicPointerChain::new(
            String::from("game.exe"),
            vec![
                SymbolicPointerChainLink::Symbol(String::from("Globals")),
                SymbolicPointerChainLink::Offset(0x20),
            ],
            PointerScanPointerSize::Pointer64,
        );
        let pointer_snapshot_query =
            SymbolTreeView::build_global_pointer_chain_virtual_snapshot_query(&project_symbol_catalog, &pointer_chain, String::from("resolver_pointer"))
                .expect("Expected pointer snapshot query.");

        let VirtualSnapshotQuery::Pointer {
            query_id,
            pointer,
            symbolic_struct_definition,
        } = pointer_snapshot_query
        else {
            panic!("Expected pointer query.");
        };

        assert_eq!(query_id, "resolver_pointer");
        assert_eq!(pointer.get_address(), 0x80);
        assert_eq!(pointer.get_module_name(), "game.exe");
        assert_eq!(pointer.get_offset_segments(), &[SymbolicPointerChainLink::Offset(0x20)]);
        assert!(symbolic_struct_definition.get_fields().is_empty());
    }

    #[test]
    fn build_relative_pointer_chain_virtual_snapshot_query_uses_instance_root() {
        let root_locator = ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x200);
        let pointer_chain = SymbolicPointerChain::new_absolute(
            vec![
                SymbolicPointerChainLink::Offset(0x10),
                SymbolicPointerChainLink::Offset(0x20),
            ],
            PointerScanPointerSize::Pointer64,
        );
        let pointer_snapshot_query =
            SymbolTreeView::build_relative_pointer_chain_virtual_snapshot_query(&root_locator, &pointer_chain, String::from("resolver_relative_pointer"))
                .expect("Expected relative pointer snapshot query.");

        let VirtualSnapshotQuery::Pointer {
            query_id,
            pointer,
            symbolic_struct_definition,
        } = pointer_snapshot_query
        else {
            panic!("Expected pointer query.");
        };

        assert_eq!(query_id, "resolver_relative_pointer");
        assert_eq!(pointer.get_address(), 0x210);
        assert_eq!(pointer.get_module_name(), "game.exe");
        assert_eq!(pointer.get_offset_segments(), &[SymbolicPointerChainLink::Offset(0x20)]);
        assert!(symbolic_struct_definition.get_fields().is_empty());
    }

    #[test]
    fn build_add_symbol_to_project_request_targets_address_item() {
        let module_symbol_claim_entry = create_module_symbol_claim_tree_entry();
        let add_symbol_to_project_target = build_add_symbol_to_project_target(&module_symbol_claim_entry).expect("Expected address add-to-project target.");
        let project_items_create_request = build_add_symbol_project_item_create_request(&add_symbol_to_project_target);

        assert_eq!(project_items_create_request.project_item_name, "Health");
        assert_eq!(project_items_create_request.address, Some(4));
        assert_eq!(project_items_create_request.module_name, Some(String::from("game.exe")));
        assert_eq!(project_items_create_request.data_type_id, Some(String::from("u32")));
        assert_eq!(
            project_items_create_request.pointer_offsets,
            Some(vec![PointerChainSegment::Symbol(String::from("Health"))])
        );
        assert!(
            project_items_create_request
                .parent_directory_path
                .as_os_str()
                .is_empty()
        );
    }

    #[test]
    fn build_add_symbol_to_project_target_accepts_struct_field_rows() {
        let struct_field_entry = create_struct_field_tree_entry();
        let add_symbol_to_project_target =
            build_add_symbol_to_project_target(&struct_field_entry).expect("Expected derived struct field add-to-project target.");

        assert_eq!(add_symbol_to_project_target.project_item_name, "PE Headers.NTHeaders.FileHeader");
        assert_eq!(add_symbol_to_project_target.address, 0x84);
        assert_eq!(add_symbol_to_project_target.module_name, "game.exe");
        assert_eq!(add_symbol_to_project_target.data_type_id, "win.pe.IMAGE_FILE_HEADER");
        assert_eq!(add_symbol_to_project_target.pointer_offsets, None);
    }

    #[test]
    fn build_add_symbol_to_project_target_ignores_module_roots() {
        let module_entry = create_module_tree_entry("game.exe");
        let unassigned_segment_entry = create_unassigned_segment_tree_entry();

        assert_eq!(build_add_symbol_to_project_target(&module_entry), None);
        assert_eq!(build_add_symbol_to_project_target(&unassigned_segment_entry), None);
    }

    #[test]
    fn build_symbol_layout_edit_target_resolves_project_symbol_layout_rows() {
        let project_symbol_catalog = ProjectSymbolCatalog::new(vec![StructLayoutDescriptor::new(
            String::from("win.pe.IMAGE_FILE_HEADER"),
            SymbolicStructDefinition::new(
                String::from("win.pe.IMAGE_FILE_HEADER"),
                vec![SymbolicFieldDefinition::new_named(
                    String::from("NumberOfSections"),
                    DataTypeRef::new("u16"),
                    ContainerType::None,
                )],
            ),
        )]);
        let struct_field_entry = create_struct_field_tree_entry();

        assert_eq!(
            build_symbol_layout_edit_target(&project_symbol_catalog, &[struct_field_entry.clone()], &struct_field_entry),
            Some(String::from("win.pe.IMAGE_FILE_HEADER"))
        );
    }

    #[test]
    fn build_symbol_layout_edit_target_resolves_module_root_layout_rows() {
        let project_symbol_catalog = ProjectSymbolCatalog::new(vec![StructLayoutDescriptor::new(
            String::from("game.exe"),
            SymbolicStructDefinition::new(String::from("game.exe"), Vec::new()).with_declared_size_in_bytes(Some(0x1000)),
        )]);
        let module_entry = create_module_tree_entry("game.exe");

        assert_eq!(
            build_symbol_layout_edit_target(&project_symbol_catalog, &[module_entry.clone()], &module_entry),
            Some(String::from("game.exe"))
        );
    }

    #[test]
    fn build_symbol_layout_edit_target_resolves_unassigned_segments_to_module_layout() {
        let project_symbol_catalog = ProjectSymbolCatalog::new(vec![StructLayoutDescriptor::new(
            String::from("game.exe"),
            SymbolicStructDefinition::new(String::from("game.exe"), Vec::new()).with_declared_size_in_bytes(Some(0x2000)),
        )]);
        let module_entry = create_module_tree_entry("game.exe");
        let unassigned_segment_entry = create_unassigned_segment_tree_entry();

        assert_eq!(
            build_symbol_layout_edit_target(
                &project_symbol_catalog,
                &[module_entry, unassigned_segment_entry.clone()],
                &unassigned_segment_entry
            ),
            Some(String::from("game.exe"))
        );
    }

    #[test]
    fn build_symbol_layout_edit_target_resolves_nearest_ancestor_struct_layout() {
        let project_symbol_catalog = ProjectSymbolCatalog::new(vec![StructLayoutDescriptor::new(
            String::from("win.pe.IMAGE_FILE_HEADER"),
            SymbolicStructDefinition::new(
                String::from("win.pe.IMAGE_FILE_HEADER"),
                vec![SymbolicFieldDefinition::new_named(
                    String::from("NumberOfSections"),
                    DataTypeRef::new("u16"),
                    ContainerType::None,
                )],
            ),
        )]);
        let parent_struct_field_entry = create_struct_field_tree_entry();
        let primitive_child_entry = SymbolTreeNode::new(
            String::from("module_field:module:game.exe:0::NTHeaders::FileHeader::NumberOfSections"),
            SymbolTreeNodeKind::StructField,
            4,
            String::from("NumberOfSections"),
            String::from("PE Headers.NTHeaders.FileHeader.NumberOfSections"),
            String::from("module:game.exe:0"),
            ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x86),
            String::from("u16"),
            ContainerType::None,
            false,
            false,
        );

        assert_eq!(
            build_symbol_layout_edit_target(
                &project_symbol_catalog,
                &[parent_struct_field_entry, primitive_child_entry.clone()],
                &primitive_child_entry
            ),
            Some(String::from("win.pe.IMAGE_FILE_HEADER"))
        );
    }

    #[test]
    fn struct_viewer_focus_target_key_includes_display_name_and_type() {
        let player_entry = create_symbol_claim_tree_entry("Player", "i32");
        let manager_entry = create_symbol_claim_tree_entry("Player Manager", "u64");

        let player_focus_key = SymbolTreeView::build_struct_viewer_focus_target_key(Some(&player_entry));
        let manager_focus_key = SymbolTreeView::build_struct_viewer_focus_target_key(Some(&manager_entry));

        assert_ne!(player_focus_key, manager_focus_key);
    }

    #[test]
    fn symbol_tree_entry_is_struct_viewer_focused_when_focus_target_matches_row_key() {
        let player_entry = create_symbol_claim_tree_entry("Player", "i32");
        let focus_target = SymbolTreeView::build_struct_viewer_focus_target(Some(&player_entry));

        assert!(SymbolTreeView::is_symbol_tree_entry_struct_viewer_focused(&player_entry, focus_target.as_ref(),));
    }

    #[test]
    fn symbol_tree_entry_is_not_struct_viewer_focused_for_other_origin() {
        let player_entry = create_symbol_claim_tree_entry("Player", "i32");
        let focus_target = StructViewerFocusTarget::ProjectHierarchy {
            project_item_paths: Vec::new(),
        };

        assert!(!SymbolTreeView::is_symbol_tree_entry_struct_viewer_focused(&player_entry, Some(&focus_target),));
    }

    #[test]
    fn build_selection_for_tree_entry_selects_module_roots_and_unassigned_segments() {
        let module_entry = create_module_tree_entry("game.exe");
        let unassigned_segment_entry = create_unassigned_segment_tree_entry();

        assert_eq!(
            SymbolTreeView::build_selection_for_tree_entry(&module_entry),
            Some(crate::views::symbol_tree::view_data::symbol_tree_view_data::SymbolTreeSelection::ModuleRoot(
                String::from("game.exe")
            ))
        );
        assert_eq!(
            SymbolTreeView::build_selection_for_tree_entry(&unassigned_segment_entry),
            Some(crate::views::symbol_tree::view_data::symbol_tree_view_data::SymbolTreeSelection::DerivedNode(
                String::from("unassigned:game.exe:0:1234")
            ))
        );
    }

    #[test]
    fn symbol_tree_entry_preview_queries_skip_unassigned_segments_and_modules() {
        let module_entry = create_module_tree_entry("game.exe");
        let unassigned_segment_entry = create_unassigned_segment_tree_entry();

        assert!(!SymbolTreeView::symbol_tree_entry_should_query_preview(&module_entry));
        assert!(!SymbolTreeView::symbol_tree_entry_should_query_preview(&unassigned_segment_entry));
    }

    #[test]
    fn format_symbol_tree_size_preview_uses_scaled_byte_units() {
        assert_eq!(SymbolTreeView::format_symbol_tree_size_preview(4), "4 B");
        assert_eq!(SymbolTreeView::format_symbol_tree_size_preview(1024), "1 KB");
        assert_eq!(SymbolTreeView::format_symbol_tree_size_preview(1536), "1.5 KB");
        assert_eq!(SymbolTreeView::format_symbol_tree_size_preview(1024 * 1024), "1 MB");
    }

    #[test]
    fn format_symbol_tree_size_tooltip_keeps_raw_bytes_with_scaled_units() {
        assert_eq!(SymbolTreeView::format_symbol_tree_size_tooltip(512), "512 B");
        assert_eq!(SymbolTreeView::format_symbol_tree_size_tooltip(1536), "1536 B (1.5 KB)");
    }

    #[test]
    fn build_module_child_range_target_handles_unassigned_segments_and_direct_module_claims() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![ProjectSymbolClaim::new_module_offset(
                String::from("Health"),
                String::from("game.exe"),
                0x04,
                String::from("u32"),
            )],
        );
        let unassigned_segment_entry = create_unassigned_segment_tree_entry();
        let module_symbol_claim_entry = create_module_symbol_claim_tree_entry();
        let unassigned_segment_target = build_module_child_range_target(&project_symbol_catalog, &unassigned_segment_entry, |data_type_ref| {
            (data_type_ref.get_data_type_id() == "u8").then_some(1)
        })
        .expect("Expected unassigned segment to resolve as a module child range.");
        let symbol_claim_target = build_module_child_range_target(&project_symbol_catalog, &module_symbol_claim_entry, |data_type_ref| {
            match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u32" => Some(4),
                _ => None,
            }
        })
        .expect("Expected direct module symbol claim to resolve as a module child range.");

        assert_eq!(unassigned_segment_target.module_name, "game.exe");
        assert_eq!(unassigned_segment_target.offset, 0);
        assert_eq!(unassigned_segment_target.length, 0x1234);
        assert_eq!(unassigned_segment_target.delete_mode, ProjectSymbolsDeleteModuleRangeMode::ShiftLeft);
        assert_eq!(symbol_claim_target.module_name, "game.exe");
        assert_eq!(symbol_claim_target.offset, 0x04);
        assert_eq!(symbol_claim_target.length, 4);
        assert_eq!(symbol_claim_target.delete_mode, ProjectSymbolsDeleteModuleRangeMode::ReplaceWithUnassigned);
    }

    #[test]
    fn build_delete_module_range_confirmation_description_marks_shift_left_as_warning() {
        let delete_confirmation_description =
            build_delete_module_range_confirmation_description("winmine.exe", 389, ProjectSymbolsDeleteModuleRangeMode::ShiftLeft);

        assert_eq!(
            delete_confirmation_description.text,
            "WARNING: winmine.exe will be 389 byte(s) smaller. Proceeding fields will be shifted left."
        );
        assert!(delete_confirmation_description.is_warning);
    }

    #[test]
    fn build_delete_module_range_confirmation_description_keeps_replace_with_unassigned_non_warning() {
        let delete_confirmation_description =
            build_delete_module_range_confirmation_description("winmine.exe", 389, ProjectSymbolsDeleteModuleRangeMode::ReplaceWithUnassigned);

        assert_eq!(
            delete_confirmation_description.text,
            "This removes the field definition and leaves the bytes unassigned."
        );
        assert!(!delete_confirmation_description.is_warning);
    }

    #[test]
    fn build_module_field_type_options_includes_builtins_and_symbol_layouts_without_pointer_variants() {
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
        let type_options = SymbolTreeDefineFieldView::build_module_field_type_options(&project_symbol_catalog);

        assert!(
            type_options
                .iter()
                .any(|type_option| { type_option.data_type_ref == DataTypeRef::new("i32") && type_option.kind == ModuleFieldTypeOptionKind::BuiltIn })
        );
        assert!(type_options.iter().any(|type_option| {
            type_option.data_type_ref == DataTypeRef::new("player.stats") && type_option.kind == ModuleFieldTypeOptionKind::SymbolLayout
        }));
        assert!(
            !type_options
                .iter()
                .any(|type_option| type_option.data_type_ref == DataTypeRef::new("player.stats*(u64)"))
        );
    }

    #[test]
    fn filter_module_field_type_options_matches_symbol_layouts() {
        let type_options = vec![
            ModuleFieldTypeOption {
                data_type_ref: DataTypeRef::new("i32"),
                label: String::from("i32"),
                kind: ModuleFieldTypeOptionKind::BuiltIn,
            },
            ModuleFieldTypeOption {
                data_type_ref: DataTypeRef::new("player.stats"),
                label: String::from("player.stats"),
                kind: ModuleFieldTypeOptionKind::SymbolLayout,
            },
        ];
        let filtered_type_options = SymbolTreeDefineFieldView::filter_module_field_type_options(&type_options, "stats");

        assert_eq!(filtered_type_options.len(), 1);
        assert!(
            filtered_type_options
                .iter()
                .all(|type_option| { !SymbolTreeDefineFieldView::module_field_type_option_uses_icon(type_option.kind) })
        );
    }

    #[test]
    fn define_field_type_popup_width_allows_two_builtin_columns() {
        assert_eq!(SymbolTreeDefineFieldView::define_field_type_popup_width(160.0), 260.0);
        assert_eq!(SymbolTreeDefineFieldView::define_field_type_popup_width(320.0), 320.0);
    }

    #[test]
    fn define_field_builtin_type_item_width_fits_inside_popup() {
        assert_eq!(SymbolTreeDefineFieldView::define_field_builtin_type_item_width(260.0), 128.0);
        assert_eq!(SymbolTreeDefineFieldView::define_field_builtin_type_item_width(320.0), 158.0);
    }

    #[test]
    fn filter_registered_pointer_sizes_omits_plugin_backed_sizes_when_unregistered() {
        let pointer_sizes = filter_registered_pointer_sizes(&[
            DataTypeRef::new("u32"),
            DataTypeRef::new("u32be"),
            DataTypeRef::new("u64"),
            DataTypeRef::new("u64be"),
        ]);

        assert_eq!(
            pointer_sizes,
            vec![
                PointerScanPointerSize::Pointer32,
                PointerScanPointerSize::Pointer32be,
                PointerScanPointerSize::Pointer64,
                PointerScanPointerSize::Pointer64be,
            ]
        );
    }

    #[test]
    fn filter_registered_pointer_sizes_includes_plugin_backed_sizes_when_registered() {
        let pointer_sizes = filter_registered_pointer_sizes(&[
            DataTypeRef::new("u24"),
            DataTypeRef::new("u24be"),
            DataTypeRef::new("u32"),
            DataTypeRef::new("u64"),
        ]);

        assert_eq!(
            pointer_sizes,
            vec![
                PointerScanPointerSize::Pointer24,
                PointerScanPointerSize::Pointer24be,
                PointerScanPointerSize::Pointer32,
                PointerScanPointerSize::Pointer64,
            ]
        );
    }

    #[test]
    fn build_define_field_plan_offsets_into_unassigned_segment() {
        let define_field_draft = DefineFieldDraft {
            display_name: String::from("health"),
            relative_offset_text: String::from("0x10"),
            relative_offset_format: AnonymousValueStringFormat::Hexadecimal,
            container_type: ContainerType::None,
            data_type_selection: DataTypeSelection::new(DataTypeRef::new("i32")),
        };
        let define_field_plan = SymbolTreeDefineFieldView::build_define_field_plan(&define_field_draft, "game.exe", 0x100, 0x40, |struct_layout_id| {
            (struct_layout_id == "i32").then_some(4)
        })
        .expect("Expected valid define-field request.");

        assert_eq!(define_field_plan.project_symbols_create_request.display_name, "health");
        assert_eq!(
            define_field_plan
                .project_symbols_create_request
                .struct_layout_id,
            "i32"
        );
        assert_eq!(define_field_plan.project_symbols_create_request.module_name, Some(String::from("game.exe")));
        assert_eq!(define_field_plan.project_symbols_create_request.offset, Some(0x110));
    }

    #[test]
    fn build_define_field_plan_accepts_pointer_container_for_struct_selection() {
        let define_field_draft = DefineFieldDraft {
            display_name: String::from("player_stats_pointer"),
            relative_offset_text: String::from("0"),
            relative_offset_format: AnonymousValueStringFormat::Decimal,
            container_type: ContainerType::Pointer(PointerScanPointerSize::Pointer64),
            data_type_selection: DataTypeSelection::new(DataTypeRef::new("player.stats")),
        };
        let define_field_plan = SymbolTreeDefineFieldView::build_define_field_plan(&define_field_draft, "game.exe", 0x100, 0x40, |struct_layout_id| {
            (struct_layout_id == "player.stats*(u64)").then_some(8)
        })
        .expect("Expected pointer-to-struct define-field request.");

        assert_eq!(
            define_field_plan
                .project_symbols_create_request
                .struct_layout_id,
            "player.stats*(u64)"
        );
        assert_eq!(define_field_plan.project_symbols_create_request.offset, Some(0x100));
    }

    #[test]
    fn build_define_field_plan_accepts_pointer_container_for_builtin_selection() {
        let define_field_draft = DefineFieldDraft {
            display_name: String::from("health_pointer"),
            relative_offset_text: String::from("0"),
            relative_offset_format: AnonymousValueStringFormat::Decimal,
            container_type: ContainerType::Pointer(PointerScanPointerSize::Pointer32),
            data_type_selection: DataTypeSelection::new(DataTypeRef::new("i32")),
        };
        let define_field_plan = SymbolTreeDefineFieldView::build_define_field_plan(&define_field_draft, "game.exe", 0x100, 0x40, |struct_layout_id| {
            (struct_layout_id == "i32*(u32)").then_some(4)
        })
        .expect("Expected pointer-to-builtin define-field request.");

        assert_eq!(
            define_field_plan
                .project_symbols_create_request
                .struct_layout_id,
            "i32*(u32)"
        );
        assert_eq!(define_field_plan.project_symbols_create_request.offset, Some(0x100));
    }

    #[test]
    fn build_define_field_plan_rejects_out_of_bounds_type() {
        let define_field_draft = DefineFieldDraft {
            display_name: String::from("health"),
            relative_offset_text: String::from("0x3E"),
            relative_offset_format: AnonymousValueStringFormat::Hexadecimal,
            container_type: ContainerType::None,
            data_type_selection: DataTypeSelection::new(DataTypeRef::new("i32")),
        };
        let define_field_plan = SymbolTreeDefineFieldView::build_define_field_plan(&define_field_draft, "game.exe", 0x100, 0x40, |struct_layout_id| {
            (struct_layout_id == "i32").then_some(4)
        });

        assert!(define_field_plan.is_err());
    }

    #[test]
    fn parse_define_field_relative_offset_accepts_hex_and_decimal() {
        assert_eq!(parse_define_field_relative_offset("0x10", AnonymousValueStringFormat::Decimal), Ok(16));
        assert_eq!(parse_define_field_relative_offset("10", AnonymousValueStringFormat::Hexadecimal), Ok(16));
        assert_eq!(parse_define_field_relative_offset("16", AnonymousValueStringFormat::Decimal), Ok(16));
        assert_eq!(parse_define_field_relative_offset("10000", AnonymousValueStringFormat::Binary), Ok(16));
    }

    #[test]
    fn module_root_create_draft_defaults_size_to_hex_1000() {
        let module_root_create_draft = ModuleRootCreateDraft::default();

        assert_eq!(module_root_create_draft.size_text, "1000");
        assert_eq!(module_root_create_draft.size_format, AnonymousValueStringFormat::Hexadecimal);
        assert_eq!(
            SymbolTreeModuleCreateView::parse_module_root_size(&module_root_create_draft.size_text, module_root_create_draft.size_format),
            Some(0x1000)
        );
    }

    #[test]
    fn build_module_root_create_request_uses_size_format() {
        let module_root_create_draft = ModuleRootCreateDraft {
            module_name: String::from("game.exe"),
            size_text: String::from("1000"),
            size_format: AnonymousValueStringFormat::Hexadecimal,
        };
        let create_request =
            SymbolTreeModuleCreateView::build_module_root_create_request_from_draft(&module_root_create_draft).expect("Expected module-root create request.");

        assert_eq!(create_request.module_name, "game.exe");
        assert_eq!(create_request.size, 0x1000);
    }

    #[test]
    fn normalize_symbol_memory_struct_prepends_claim_metadata_and_keeps_value_rows_editable() {
        let symbol_claim_tree_entry = create_symbol_claim_tree_entry("Player", "i32");
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeU32::get_value_from_primitive(100).to_named_valued_struct_field(String::from("health"), false),
        ]);

        let normalized_struct = SymbolTreeView::normalize_symbol_memory_struct(valued_struct, &symbol_claim_tree_entry, true, Some(4));
        let normalized_fields = normalized_struct.get_fields();

        assert_eq!(normalized_fields[0].get_name(), SymbolTreeView::STRUCT_VIEWER_SYMBOL_NAME_FIELD);
        assert_eq!(
            normalized_fields[1].get_name(),
            ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE
        );
        assert!(normalized_fields[1].get_is_read_only());
        assert_eq!(normalized_fields[2].get_name(), ProjectItemTypeAddress::PROPERTY_ADDRESS);
        assert_eq!(normalized_fields[3].get_name(), ProjectItemTypeAddress::PROPERTY_MODULE);
        assert_eq!(normalized_fields[4].get_name(), SymbolTreeView::STRUCT_VIEWER_SYMBOL_SIZE_FIELD);
        assert_eq!(normalized_fields[5].get_name(), SymbolTreeView::STRUCT_VIEWER_SYMBOL_PATH_FIELD);
        assert_eq!(normalized_fields[6].get_name(), "health");
        assert!(!normalized_fields[6].get_is_read_only());
    }

    #[test]
    fn build_external_value_symbol_layout_routes_arrays_through_memory_viewer_value_field() {
        let symbol_tree_entry = create_fixed_array_symbol_claim_tree_entry("u8", 0x1234);
        let symbol_layout = SymbolTreeView::build_external_value_symbol_layout(&symbol_tree_entry, false, Some(0x1234));
        let fields = symbol_layout.get_fields();

        assert!(
            symbol_layout
                .get_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE)
                .is_some()
        );
        assert!(
            symbol_layout
                .get_field(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE)
                .is_some()
        );
        assert_eq!(
            fields
                .iter()
                .find(|field| field.get_name() == ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE)
                .map(|field| field.get_is_read_only()),
            Some(true)
        );
        assert_eq!(
            fields
                .iter()
                .find(|field| field.get_name() == ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE)
                .map(|field| field.get_is_read_only()),
            Some(true)
        );
    }

    #[test]
    fn build_external_value_symbol_layout_is_not_limited_to_u8_arrays() {
        let symbol_tree_entry = create_fixed_array_symbol_claim_tree_entry("u16", 4);

        assert!(SymbolTreeView::symbol_tree_entry_should_use_external_value_viewer(&symbol_tree_entry));

        let symbol_layout = SymbolTreeView::build_external_value_symbol_layout(&symbol_tree_entry, true, Some(8));

        assert_eq!(
            symbol_layout
                .get_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE)
                .map(StructViewerViewData::read_utf8_field_text),
            Some(String::from("u16[4]"))
        );
        assert!(
            symbol_layout
                .get_field(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE)
                .is_some()
        );
    }
}
