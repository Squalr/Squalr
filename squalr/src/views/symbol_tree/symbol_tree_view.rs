use crate::app_context::AppContext;
use crate::ui::list_navigation::{ListNavigationDirection, resolve_next_index};
use crate::views::{
    code_viewer::view_data::code_viewer_view_data::CodeViewerViewData,
    memory_viewer::view_data::memory_viewer_view_data::MemoryViewerViewData,
    struct_viewer::view_data::struct_viewer_focus_target::StructViewerFocusTarget,
    struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData,
    symbol_layout_editor::view_data::symbol_layout_editor_view_data::SymbolLayoutEditorViewData,
    symbol_tree::symbol_tree_command_dispatcher::SymbolTreeCommandDispatcher,
    symbol_tree::symbol_tree_details_focus::SymbolTreeDetailsFocus,
    symbol_tree::symbol_tree_list_view::{SymbolTreeListAction, SymbolTreeListView, SymbolTreeRenameTarget},
    symbol_tree::symbol_tree_module_create_view::SymbolTreeModuleCreateView,
    symbol_tree::symbol_tree_runtime_data_controller::SymbolTreeRuntimeDataController,
    symbol_tree::symbol_tree_takeover_host_view::{SymbolTreeTakeoverHostAction, SymbolTreeTakeoverHostView},
    symbol_tree::symbol_tree_toolbar_view::{SymbolTreeToolbarAction, SymbolTreeToolbarView},
    symbol_tree::view_data::symbol_tree_view_data::{SymbolTreeSelection, SymbolTreeTakeOverState, SymbolTreeViewData},
};
use eframe::egui::{Align, Direction, Key, Layout, Response, RichText, ScrollArea, Ui, UiBuilder, Widget};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog,
    symbol_tree::operations::delete_symbol::build_module_child_range_target,
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

    fn command_dispatcher(&self) -> SymbolTreeCommandDispatcher {
        SymbolTreeCommandDispatcher::new(
            self.app_context.clone(),
            self.symbol_tree_view_data.clone(),
            self.symbol_layout_editor_view_data.clone(),
            self.memory_viewer_view_data.clone(),
            self.code_viewer_view_data.clone(),
        )
    }

    fn details_focus(&self) -> SymbolTreeDetailsFocus {
        SymbolTreeDetailsFocus::new(self.app_context.clone(), self.struct_viewer_view_data.clone())
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
                self.details_focus()
                    .focus_symbol_tree_entry_in_struct_viewer(project_symbol_catalog, &symbol_tree_entry);
            }
            SymbolTreeListAction::OpenMemoryViewer(locator) => {
                self.command_dispatcher()
                    .focus_memory_viewer_for_locator(&locator);
            }
            SymbolTreeListAction::OpenCodeViewer(locator) => {
                self.command_dispatcher()
                    .focus_code_viewer_for_locator(&locator);
            }
            SymbolTreeListAction::AddSymbolToProject(add_symbol_to_project_target) => {
                self.command_dispatcher()
                    .add_symbol_to_project(&add_symbol_to_project_target);
            }
            SymbolTreeListAction::EditSymbolLayout(struct_layout_id) => {
                self.command_dispatcher()
                    .edit_symbol_tree_entry_symbol_layout(project_symbol_catalog, &struct_layout_id);
            }
            SymbolTreeListAction::BeginInlineRename(tree_node_key) => {
                SymbolTreeViewData::begin_inline_rename(self.symbol_tree_view_data.clone(), tree_node_key);
            }
            SymbolTreeListAction::CommitRename(rename_target) => match rename_target {
                SymbolTreeRenameTarget::ModuleRoot { module_name, new_module_name } => {
                    self.command_dispatcher()
                        .rename_module_root(&module_name, new_module_name);
                }
                SymbolTreeRenameTarget::SymbolClaim {
                    symbol_locator_key,
                    display_name,
                } => {
                    self.command_dispatcher()
                        .rename_symbol_claim(&symbol_locator_key, display_name);
                }
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
                self.command_dispatcher().request_delete_target(delete_target);
            }
            SymbolTreeListAction::ExecutePluginAction(menu_item, context) => {
                self.command_dispatcher()
                    .execute_symbol_tree_plugin_action(&menu_item, context);
            }
            SymbolTreeListAction::CreateModuleRoot => {
                SymbolTreeViewData::begin_create_module_root(self.symbol_tree_view_data.clone());
            }
        }
    }

    fn apply_symbol_tree_takeover_host_action(
        &self,
        takeover_host_action: SymbolTreeTakeoverHostAction,
    ) {
        match takeover_host_action {
            SymbolTreeTakeoverHostAction::None => {}
            SymbolTreeTakeoverHostAction::CancelTakeover => {
                SymbolTreeViewData::cancel_take_over_state(self.symbol_tree_view_data.clone());
            }
            SymbolTreeTakeoverHostAction::CancelModuleRootCreate => {
                SymbolTreeViewData::set_selected_entry(self.symbol_tree_view_data.clone(), None);
            }
            SymbolTreeTakeoverHostAction::DeleteSymbolClaim { symbol_locator_key } => {
                self.command_dispatcher()
                    .delete_symbol_claim(&symbol_locator_key);
            }
            SymbolTreeTakeoverHostAction::DeleteModuleRoot { module_name } => {
                self.command_dispatcher().delete_module_root(&module_name);
            }
            SymbolTreeTakeoverHostAction::DeleteModuleRange {
                module_name,
                offset,
                length,
                mode,
            } => {
                self.command_dispatcher()
                    .delete_module_range(&module_name, offset, length, mode);
            }
            SymbolTreeTakeoverHostAction::CreateFieldFromUnassignedSegment {
                module_name,
                define_field_plan,
            } => {
                SymbolTreeViewData::cancel_take_over_state(self.symbol_tree_view_data.clone());
                self.command_dispatcher()
                    .create_define_field_from_unassigned_span_edit_target(&module_name, define_field_plan);
            }
            SymbolTreeTakeoverHostAction::DefineFieldDraftChanged(define_field_draft) => {
                SymbolTreeViewData::set_define_field_draft(self.symbol_tree_view_data.clone(), define_field_draft);
            }
            SymbolTreeTakeoverHostAction::CreateModuleRoot(project_symbols_create_module_request) => {
                self.command_dispatcher()
                    .create_module_root(project_symbols_create_module_request);
            }
            SymbolTreeTakeoverHostAction::ModuleRootCreateDraftChanged(module_root_create_draft) => {
                SymbolTreeViewData::set_module_root_create_draft(self.symbol_tree_view_data.clone(), module_root_create_draft);
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
        self.details_focus()
            .sync_selected_symbol_into_struct_viewer(&project_symbol_catalog, selected_symbol_tree_entry);
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
                Some(SymbolTreeTakeOverState::DeleteSymbolClaimConfirmation { symbol_locator_key, .. }) => {
                    self.command_dispatcher()
                        .delete_symbol_claim(symbol_locator_key);
                }
                Some(SymbolTreeTakeOverState::DeleteModuleRootConfirmation { module_name }) => {
                    self.command_dispatcher().delete_module_root(module_name);
                }
                Some(SymbolTreeTakeOverState::DeleteModuleRangeConfirmation {
                    module_name,
                    offset,
                    length,
                    mode,
                    ..
                }) => {
                    self.command_dispatcher()
                        .delete_module_range(module_name, *offset, *length, *mode);
                }
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
                self.command_dispatcher()
                    .create_module_root(project_symbols_create_module_request);
            }
        }

        if can_use_standard_toolbar_actions && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowUp)) {
            if let Some(next_symbol_tree_entry) =
                Self::resolve_adjacent_symbol_tree_entry(&symbol_tree_entries, selected_entry.as_ref(), ListNavigationDirection::Up)
            {
                if let Some(selection) = SymbolTreeListView::build_selection_for_tree_entry(next_symbol_tree_entry) {
                    SymbolTreeViewData::set_selected_entry(self.symbol_tree_view_data.clone(), Some(selection));

                    if !matches!(next_symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. }) {
                        self.details_focus()
                            .focus_symbol_tree_entry_in_struct_viewer(&project_symbol_catalog, next_symbol_tree_entry);
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
                        self.details_focus()
                            .focus_symbol_tree_entry_in_struct_viewer(&project_symbol_catalog, next_symbol_tree_entry);
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
            self.command_dispatcher().request_delete_for_selection(
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

                let takeover_host_response = SymbolTreeTakeoverHostView::new(
                    self.app_context.clone(),
                    &project_symbol_catalog,
                    selected_entry.as_ref(),
                    take_over_state.as_ref(),
                    &current_module_root_create_draft,
                    &current_define_field_draft,
                )
                .show(&mut list_user_interface);
                self.apply_symbol_tree_takeover_host_action(takeover_host_response.action);

                if takeover_host_response.is_active {
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
