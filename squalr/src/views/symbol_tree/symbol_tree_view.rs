use crate::app_context::AppContext;
use crate::ui::list_navigation::{ListNavigationDirection, resolve_next_index};
use crate::views::{
    code_viewer::{code_viewer_view::CodeViewerView, view_data::code_viewer_view_data::CodeViewerViewData},
    memory_viewer::{memory_viewer_view::MemoryViewerView, view_data::memory_viewer_view_data::MemoryViewerViewData},
    struct_viewer::view_data::struct_viewer_focus_target::StructViewerFocusTarget,
    struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData,
    symbol_layout_editor::{symbol_layout_editor_view::SymbolLayoutEditorView, view_data::symbol_layout_editor_view_data::SymbolLayoutEditorViewData},
    symbol_tree::symbol_tree_define_field_view::{SymbolTreeDefineFieldAction, SymbolTreeDefineFieldView},
    symbol_tree::symbol_tree_delete_confirmation_view::{SymbolTreeDeleteConfirmationAction, SymbolTreeDeleteConfirmationView},
    symbol_tree::symbol_tree_list_view::{
        SymbolTreeDeleteTarget, SymbolTreeListAction, SymbolTreeListView, SymbolTreePluginActionMenuItem, SymbolTreeRenameTarget,
    },
    symbol_tree::symbol_tree_module_create_view::{SymbolTreeModuleCreateAction, SymbolTreeModuleCreateView},
    symbol_tree::symbol_tree_runtime_data_controller::SymbolTreeRuntimeDataController,
    symbol_tree::symbol_tree_toolbar_view::{SymbolTreeToolbarAction, SymbolTreeToolbarView},
    symbol_tree::view_data::symbol_tree_view_data::{SymbolTreeSelection, SymbolTreeTakeOverState, SymbolTreeViewData},
};
use eframe::egui::{Align, Direction, Key, Layout, Response, RichText, ScrollArea, Ui, UiBuilder, Widget};
use squalr_engine_api::commands::{
    project_symbols::{
        create::project_symbols_create_request::ProjectSymbolsCreateRequest,
        create_module::project_symbols_create_module_request::ProjectSymbolsCreateModuleRequest,
        delete::project_symbols_delete_request::{ProjectSymbolsDeleteModuleRange, ProjectSymbolsDeleteModuleRangeMode, ProjectSymbolsDeleteRequest},
        execute_plugin_action::project_symbols_execute_plugin_action_request::ProjectSymbolsExecutePluginActionRequest,
        rename::project_symbols_rename_request::ProjectSymbolsRenameRequest,
        rename_module::project_symbols_rename_module_request::ProjectSymbolsRenameModuleRequest,
        write_value::project_symbols_write_value_request::ProjectSymbolsWriteValueRequest,
    },
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::plugins::symbol_tree::symbol_tree_action::{SymbolTreeActionContext, SymbolTreeActionSelection};
use squalr_engine_api::structures::data_values::{
    anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
    data_value::DataValue,
};
use squalr_engine_api::structures::details::{DetailsEdit, DetailsFieldSource, DetailsValue};
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog,
    project_symbol_locator::ProjectSymbolLocator,
    symbol_tree::operations::{
        add_symbol_to_project::{AddSymbolToProjectTarget, build_add_symbol_project_item_create_request},
        define_field::DefineFieldPlan,
        delete_symbol::{ModuleChildRangeTarget, build_delete_module_range_confirmation_description, build_module_child_range_target},
    },
    symbol_tree::symbol_tree_node::{SymbolTreeNode, SymbolTreeNodeKind},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct SymbolTreeView {
    app_context: Arc<AppContext>,
    symbol_tree_view_data: Dependency<SymbolTreeViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
    symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
    code_viewer_view_data: Dependency<CodeViewerViewData>,
}

impl SymbolTreeView {
    pub const WINDOW_ID: &'static str = "window_symbol_tree";
    const SYMBOL_TREE_TEXT_PADDING_X: f32 = 8.0;

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
                if symbol_tree_entry.get_node_key().starts_with("module_field:") {
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

    fn request_delete_target(
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

    fn apply_symbol_tree_list_action(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        list_action: SymbolTreeListAction,
    ) {
        match list_action {
            SymbolTreeListAction::Select(selection) => {
                SymbolTreeViewData::set_selected_entry(self.symbol_tree_view_data.clone(), Some(selection));
            }
            SymbolTreeListAction::ToggleExpansion(tree_node_key) => {
                SymbolTreeViewData::toggle_tree_node_expansion(self.symbol_tree_view_data.clone(), &tree_node_key);
            }
            SymbolTreeListAction::FocusStructViewer(symbol_tree_entry) => {
                self.focus_symbol_tree_entry_in_struct_viewer(project_symbol_catalog, &symbol_tree_entry);
            }
            SymbolTreeListAction::OpenMemoryViewer(locator) => {
                self.focus_memory_viewer_for_locator(&locator);
            }
            SymbolTreeListAction::OpenCodeViewer(locator) => {
                self.focus_code_viewer_for_locator(&locator);
            }
            SymbolTreeListAction::AddSymbolToProject(add_symbol_to_project_target) => {
                self.add_symbol_to_project(&add_symbol_to_project_target);
            }
            SymbolTreeListAction::EditSymbolLayout(struct_layout_id) => {
                self.edit_symbol_tree_entry_symbol_layout(project_symbol_catalog, &struct_layout_id);
            }
            SymbolTreeListAction::BeginInlineRename(tree_node_key) => {
                SymbolTreeViewData::begin_inline_rename(self.symbol_tree_view_data.clone(), tree_node_key);
            }
            SymbolTreeListAction::CommitRename(rename_target) => match rename_target {
                SymbolTreeRenameTarget::ModuleRoot { module_name, new_module_name } => self.rename_module_root(&module_name, new_module_name),
                SymbolTreeRenameTarget::SymbolClaim {
                    symbol_locator_key,
                    display_name,
                } => self.rename_symbol_claim(&symbol_locator_key, display_name),
            },
            SymbolTreeListAction::CancelInlineRename(tree_node_key) => {
                SymbolTreeListView::clear_inline_rename_state(self.symbol_tree_view_data.clone(), user_interface, &tree_node_key);
            }
            SymbolTreeListAction::ShowContextMenu(context_menu_target) => {
                SymbolTreeViewData::show_context_menu(self.symbol_tree_view_data.clone(), context_menu_target);
            }
            SymbolTreeListAction::HideContextMenu => {
                SymbolTreeViewData::hide_context_menu(self.symbol_tree_view_data.clone());
            }
            SymbolTreeListAction::RequestDelete(delete_target) => {
                self.request_delete_target(delete_target);
            }
            SymbolTreeListAction::ExecutePluginAction(menu_item, context) => {
                self.execute_symbol_tree_plugin_action(&menu_item, context);
            }
            SymbolTreeListAction::CreateModuleRoot => {
                SymbolTreeViewData::begin_create_module_root(self.symbol_tree_view_data.clone());
            }
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

    fn focus_symbol_tree_entry_in_struct_viewer(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_symbol_tree_entry: &SymbolTreeNode,
    ) {
        let focus_target = Self::build_struct_viewer_focus_target(Some(selected_symbol_tree_entry));

        let details_projection = SymbolTreeRuntimeDataController::new(self.app_context.clone())
            .build_symbol_details_projection_for_tree_entry(project_symbol_catalog, selected_symbol_tree_entry);
        let details_edit_callback = self.build_symbol_details_edit_callback(selected_symbol_tree_entry);

        StructViewerViewData::focus_details_projection_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_projection,
            details_edit_callback,
            focus_target,
        );
    }

    fn build_symbol_details_edit_callback(
        &self,
        selected_symbol_tree_entry: &SymbolTreeNode,
    ) -> Arc<dyn Fn(DetailsEdit) + Send + Sync> {
        let selected_symbol_tree_entry = selected_symbol_tree_entry.clone();
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();

        Arc::new(move |details_edit: DetailsEdit| match details_edit.get_value() {
            DetailsValue::Empty => {}
            details_value => {
                let DetailsFieldSource::ProjectSymbolRuntimeValue { field_path } = details_edit.get_source() else {
                    return;
                };
                let field_name = field_path
                    .last()
                    .cloned()
                    .unwrap_or_else(|| String::from("value"));
                let Some(anonymous_value_string) = Self::details_value_to_anonymous_value_string(engine_unprivileged_state.as_ref(), details_value) else {
                    return;
                };

                ProjectSymbolsWriteValueRequest {
                    address: selected_symbol_tree_entry.get_locator().get_focus_address(),
                    module_name: selected_symbol_tree_entry
                        .get_locator()
                        .get_focus_module_name()
                        .to_string(),
                    symbol_type_id: selected_symbol_tree_entry.get_symbol_type_id().to_string(),
                    container_type: selected_symbol_tree_entry.get_container_type(),
                    field_name,
                    anonymous_value_string,
                }
                .send(&engine_unprivileged_state, |project_symbols_write_value_response| {
                    if !project_symbols_write_value_response.success {
                        log::warn!(
                            "Symbol Tree details value write command failed: {}",
                            project_symbols_write_value_response
                                .error
                                .as_deref()
                                .unwrap_or("unknown error")
                        );
                    }
                });
            }
        })
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

    fn details_value_to_anonymous_value_string(
        engine_execution_context: &dyn EngineExecutionContext,
        details_value: &DetailsValue,
    ) -> Option<AnonymousValueString> {
        match details_value {
            DetailsValue::AnonymousValue(anonymous_value_string) => Some(anonymous_value_string.clone()),
            DetailsValue::DataValue(data_value) => Self::data_value_to_anonymous_value_string(engine_execution_context, data_value),
            DetailsValue::Text(text) => Some(AnonymousValueString::new(text.clone(), AnonymousValueStringFormat::String, ContainerType::None)),
            DetailsValue::Boolean(value) => Some(AnonymousValueString::new(
                value.to_string(),
                AnonymousValueStringFormat::Bool,
                ContainerType::None,
            )),
            DetailsValue::UnsignedInteger(value) => Some(AnonymousValueString::new(
                value.to_string(),
                AnonymousValueStringFormat::Decimal,
                ContainerType::None,
            )),
            DetailsValue::SignedInteger(value) => Some(AnonymousValueString::new(
                value.to_string(),
                AnonymousValueStringFormat::Decimal,
                ContainerType::None,
            )),
            DetailsValue::Empty => Some(AnonymousValueString::new(
                String::new(),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            )),
        }
    }

    fn data_value_to_anonymous_value_string(
        engine_execution_context: &dyn EngineExecutionContext,
        data_value: &DataValue,
    ) -> Option<AnonymousValueString> {
        let anonymous_value_string_format = engine_execution_context.get_default_anonymous_value_string_format(data_value.get_data_type_ref());

        engine_execution_context
            .anonymize_value(data_value, anonymous_value_string_format)
            .map_err(|error| {
                log::warn!("Failed to format Symbol Tree details edit: {}", error);
                error
            })
            .ok()
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
}

impl Widget for SymbolTreeView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let Some(project_symbol_catalog) = self.get_opened_project_symbol_catalog() else {
            SymbolTreeRuntimeDataController::new(self.app_context.clone()).clear_virtual_snapshots();

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
        let runtime_data = SymbolTreeRuntimeDataController::new(self.app_context.clone()).build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
        let symbol_tree_entries = runtime_data.symbol_tree_entries;
        let preview_values_by_node_key = runtime_data.preview_values_by_node_key;
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
                if let Some(selection) = SymbolTreeListView::build_selection_for_tree_entry(next_symbol_tree_entry) {
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
                if let Some(selection) = SymbolTreeListView::build_selection_for_tree_entry(next_symbol_tree_entry) {
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
                SymbolTreeListView::clear_inline_rename_state(self.symbol_tree_view_data.clone(), user_interface, active_inline_rename_tree_node_key);
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
                        let list_actions = SymbolTreeListView::new(
                            self.app_context.clone(),
                            &project_symbol_catalog,
                            &symbol_tree_entries,
                            &preview_values_by_node_key,
                            selected_entry.as_ref(),
                            inline_rename_tree_node_key.as_deref(),
                            context_menu_target.as_ref(),
                            shared_struct_viewer_focus_target.as_ref(),
                            !is_inline_rename_active,
                        )
                        .show(user_interface);

                        for list_action in list_actions {
                            self.apply_symbol_tree_list_action(user_interface, &project_symbol_catalog, list_action);
                        }
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
