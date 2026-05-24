use crate::app_context::AppContext;
use crate::views::{
    code_viewer::{code_viewer_view::CodeViewerView, view_data::code_viewer_view_data::CodeViewerViewData},
    memory_viewer::{memory_viewer_view::MemoryViewerView, view_data::memory_viewer_view_data::MemoryViewerViewData},
    symbol_layout_editor::{symbol_layout_editor_view::SymbolLayoutEditorView, view_data::symbol_layout_editor_view_data::SymbolLayoutEditorViewData},
    symbol_tree::{
        symbol_tree_list_view::{SymbolTreeDeleteTarget, SymbolTreePluginActionMenuItem},
        view_data::symbol_tree_view_data::{SymbolTreeSelection, SymbolTreeViewData},
    },
};
use squalr_engine_api::commands::{
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
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog,
    project_symbol_claim::ProjectSymbolClaim,
    project_symbol_locator::ProjectSymbolLocator,
    symbol_tree::operations::{
        add_symbol_to_project::{AddSymbolToProjectTarget, build_add_symbol_project_item_create_request},
        define_field::DefineFieldPlan,
        delete_symbol::ModuleChildRangeTarget,
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub struct SymbolTreeCommandDispatcher {
    app_context: Arc<AppContext>,
    symbol_tree_view_data: Dependency<SymbolTreeViewData>,
    symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
    code_viewer_view_data: Dependency<CodeViewerViewData>,
}

impl SymbolTreeCommandDispatcher {
    pub fn new(
        app_context: Arc<AppContext>,
        symbol_tree_view_data: Dependency<SymbolTreeViewData>,
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        memory_viewer_view_data: Dependency<MemoryViewerViewData>,
        code_viewer_view_data: Dependency<CodeViewerViewData>,
    ) -> Self {
        Self {
            app_context,
            symbol_tree_view_data,
            symbol_layout_editor_view_data,
            memory_viewer_view_data,
            code_viewer_view_data,
        }
    }

    pub fn focus_memory_viewer_for_locator(
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

    pub fn focus_code_viewer_for_locator(
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

    pub fn rename_symbol_claim(
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

    pub fn rename_module_root(
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

    pub fn delete_symbol_claim(
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

    pub fn delete_module_range(
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

    pub fn delete_module_root(
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

    pub fn create_module_root(
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

    pub fn edit_symbol_tree_entry_symbol_layout(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_id: &str,
    ) {
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();
        SymbolLayoutEditorViewData::begin_open_symbol_layout(
            self.symbol_layout_editor_view_data.clone(),
            project_symbol_catalog,
            struct_layout_id,
            |data_type_ref| {
                let size_in_bytes = engine_unprivileged_state.get_unit_size_in_bytes(data_type_ref);

                (size_in_bytes > 0).then_some(size_in_bytes)
            },
        );

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

    pub fn add_symbol_to_project(
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

    pub fn create_define_field_from_unassigned_span_edit_target(
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

    pub fn request_delete_for_selection(
        &self,
        selected_symbol_claim: Option<&ProjectSymbolClaim>,
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

    pub fn request_delete_target(
        &self,
        delete_target: SymbolTreeDeleteTarget,
    ) {
        match delete_target {
            SymbolTreeDeleteTarget::ModuleRange(module_child_range_target) => {
                SymbolTreeViewData::request_delete_module_range_confirmation(
                    self.symbol_tree_view_data.clone(),
                    module_child_range_target.module_name,
                    module_child_range_target.offset,
                    module_child_range_target.length,
                    module_child_range_target.display_name,
                    module_child_range_target.delete_mode,
                );
            }
            SymbolTreeDeleteTarget::SymbolClaim {
                symbol_locator_key,
                display_name,
            } => {
                SymbolTreeViewData::request_delete_symbol_claim_confirmation(self.symbol_tree_view_data.clone(), symbol_locator_key, display_name);
            }
            SymbolTreeDeleteTarget::ModuleRoot { module_name } => {
                SymbolTreeViewData::request_delete_module_root_confirmation(self.symbol_tree_view_data.clone(), module_name);
            }
        }
    }

    pub fn execute_symbol_tree_plugin_action(
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
}
