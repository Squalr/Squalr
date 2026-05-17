use crate::{
    app_context::AppContext,
    ui::{
        list_navigation::ListNavigationDirection,
        widgets::controls::context_menu::context_menu::{ContextMenu, ContextMenuSizing},
    },
    views::code_viewer::{code_viewer_view::CodeViewerView, view_data::code_viewer_view_data::CodeViewerViewData},
    views::memory_viewer::{memory_viewer_view::MemoryViewerView, view_data::memory_viewer_view_data::MemoryViewerViewData},
    views::pointer_scanner::{pointer_scanner_view::PointerScannerView, view_data::pointer_scanner_view_data::PointerScannerViewData},
    views::project_explorer::{
        project_explorer_view::ProjectExplorerView,
        project_hierarchy::{
            project_hierarchy_create_item_menu_view::ProjectHierarchyCreateItemMenuView,
            project_hierarchy_details_focus::ProjectHierarchyDetailsFocus,
            project_hierarchy_list_view::{ProjectHierarchyListAction, ProjectHierarchyListView},
            project_hierarchy_runtime_preview_controller::ProjectHierarchyRuntimePreviewController,
            project_hierarchy_takeover_host_view::{ProjectHierarchyTakeoverHostAction, ProjectHierarchyTakeoverHostView},
            project_hierarchy_toolbar_view::ProjectHierarchyToolbarView,
            project_item_details::ProjectItemDetails,
            project_item_rename_request_builder::ProjectItemRenameRequestBuilder,
            view_data::{
                project_hierarchy_drop_target::ProjectHierarchyDropTarget, project_hierarchy_frame_action::ProjectHierarchyFrameAction,
                project_hierarchy_menu_target::ProjectHierarchyMenuTarget, project_hierarchy_pending_operation::ProjectHierarchyPendingOperation,
                project_hierarchy_take_over_state::ProjectHierarchyTakeOverState, project_hierarchy_view_data::ProjectHierarchyViewData,
            },
        },
    },
    views::struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData},
};
use eframe::egui::{Align, CursorIcon, Key, Layout, Pos2, Response, Ui, Widget};
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::commands::project_items::write_value::project_items_write_value_request::ProjectItemsWriteValueRequest;
use squalr_engine_api::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::memory::address_display::try_resolve_virtual_module_address;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct ProjectHierarchyView {
    app_context: Arc<AppContext>,
    project_hierarchy_toolbar_view: ProjectHierarchyToolbarView,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    code_viewer_view_data: Dependency<CodeViewerViewData>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
    pointer_scanner_view_data: Dependency<PointerScannerViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProjectItemChangeSignature {
    project_item_type_id: String,
    project_item_name: String,
    project_item_description: String,
    project_item_properties: Vec<(String, Vec<u8>)>,
}

impl ProjectHierarchyView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();
        let project_hierarchy_toolbar_view = ProjectHierarchyToolbarView::new(app_context.clone());
        let struct_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<StructViewerViewData>();
        let memory_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<MemoryViewerViewData>();
        let code_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<CodeViewerViewData>();
        let pointer_scanner_view_data = app_context
            .dependency_container
            .get_dependency::<PointerScannerViewData>();
        ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data.clone(), app_context.clone());

        Self {
            app_context,
            project_hierarchy_toolbar_view,
            project_hierarchy_view_data,
            code_viewer_view_data,
            memory_viewer_view_data,
            pointer_scanner_view_data,
            struct_viewer_view_data,
        }
    }

    fn details_focus(&self) -> ProjectHierarchyDetailsFocus {
        ProjectHierarchyDetailsFocus::new(
            self.app_context.clone(),
            self.project_hierarchy_view_data.clone(),
            self.struct_viewer_view_data.clone(),
        )
    }

    fn runtime_preview_controller(&self) -> ProjectHierarchyRuntimePreviewController {
        ProjectHierarchyRuntimePreviewController::new(self.app_context.clone(), self.project_hierarchy_view_data.clone(), self.details_focus())
    }
}
impl Widget for ProjectHierarchyView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        self.sync_scan_settings_if_needed();
        let runtime_preview_controller = self.runtime_preview_controller();
        let project_read_interval = runtime_preview_controller.get_project_read_interval();
        user_interface
            .ctx()
            .request_repaint_after(project_read_interval);

        self.refresh_if_project_changed();
        runtime_preview_controller.refresh_if_project_preview_values_stale(project_read_interval);

        let project_hierarchy_toolbar_view = self.project_hierarchy_toolbar_view.clone();
        let mut project_hierarchy_frame_action = ProjectHierarchyFrameAction::None;
        let mut drag_started_project_item_path: Option<PathBuf> = None;
        let mut hovered_drop_target_project_item_path: Option<ProjectHierarchyDropTarget> = None;
        let mut should_cancel_take_over = false;
        let mut delete_confirmation_project_item_paths: Option<Vec<std::path::PathBuf>> = None;
        let mut promote_symbol_overwrite_project_item_paths: Option<Vec<std::path::PathBuf>> = None;
        let mut rename_project_item_submission: Option<(PathBuf, String, String)> = None;
        let mut value_edit_project_item_submission: Option<(PathBuf, String, DataTypeRef, AnonymousValueString)> = None;
        let mut keyboard_activation_toggle_target: Option<(Vec<PathBuf>, bool)> = None;
        let mut is_delete_confirmation_active = false;
        let mut is_promote_symbol_conflict_active = false;
        let mut is_rename_take_over_active = false;
        let mut is_value_edit_take_over_active = false;
        let mut visible_preview_project_item_paths = Vec::new();
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let shared_struct_viewer_focus_target = self
                    .struct_viewer_view_data
                    .read("Project hierarchy shared struct viewer focus target")
                    .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned());
                let active_struct_viewer_project_item_paths: HashSet<PathBuf> = match shared_struct_viewer_focus_target.as_ref() {
                    Some(StructViewerFocusTarget::ProjectHierarchy { project_item_paths }) => project_item_paths.iter().cloned().collect(),
                    _ => HashSet::new(),
                };

                let project_hierarchy_view_data = match self.project_hierarchy_view_data.read("Project hierarchy view") {
                    Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                    None => return,
                };
                let take_over_state = project_hierarchy_view_data.take_over_state.clone();
                let tree_entries = project_hierarchy_view_data.tree_entries.clone();
                let selected_project_item_paths = project_hierarchy_view_data.selected_project_item_paths.clone();
                let dragged_project_item_paths = project_hierarchy_view_data.dragged_project_item_paths.clone();
                let menu_target = project_hierarchy_view_data.menu_target.clone();
                let menu_position = project_hierarchy_view_data.menu_position;
                let selected_project_item_paths_in_tree_order = project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order();
                let pending_operation = project_hierarchy_view_data.pending_operation.clone();

                user_interface.add(project_hierarchy_toolbar_view);
                self.show_toolbar_add_menu(&mut project_hierarchy_frame_action, user_interface, menu_target.as_ref(), menu_position);

                match pending_operation {
                    ProjectHierarchyPendingOperation::Deleting => {
                        user_interface.label("Deleting project item(s)...");
                    }
                    ProjectHierarchyPendingOperation::Promoting => {
                        user_interface.label("Promoting project item(s) to symbols...");
                    }
                    ProjectHierarchyPendingOperation::Reordering => {
                        user_interface.label("Reordering project item(s)...");
                    }
                    _ => {}
                }

                let active_inline_rename = match &take_over_state {
                    ProjectHierarchyTakeOverState::RenameProjectItem {
                        project_item_path,
                        project_item_type_id,
                    } => Some((project_item_path.clone(), project_item_type_id.clone())),
                    _ => None,
                };
                is_rename_take_over_active = active_inline_rename.is_some();
                let active_value_edit_project_item_path = match &take_over_state {
                    ProjectHierarchyTakeOverState::EditProjectItemValue { project_item_path } => Some(project_item_path.clone()),
                    _ => None,
                };
                is_value_edit_take_over_active = active_value_edit_project_item_path.is_some();
                is_delete_confirmation_active = matches!(take_over_state, ProjectHierarchyTakeOverState::DeleteConfirmation { .. });
                is_promote_symbol_conflict_active = matches!(take_over_state, ProjectHierarchyTakeOverState::PromoteSymbolConflict { .. });
                match &take_over_state {
                    ProjectHierarchyTakeOverState::None | ProjectHierarchyTakeOverState::RenameProjectItem { .. } => {
                        let list_response = ProjectHierarchyListView::new(
                            self.app_context.clone(),
                            self.project_hierarchy_view_data.clone(),
                            project_hierarchy_view_data.opened_project_info.as_ref(),
                            &tree_entries,
                            &selected_project_item_paths,
                            &selected_project_item_paths_in_tree_order,
                            dragged_project_item_paths.clone(),
                            &active_struct_viewer_project_item_paths,
                            active_inline_rename.clone(),
                            menu_target.as_ref(),
                            menu_position,
                        )
                        .show(user_interface);

                        visible_preview_project_item_paths.extend(list_response.visible_preview_project_item_paths);

                        for list_action in list_response.actions {
                            match list_action {
                                ProjectHierarchyListAction::Frame(frame_action) => {
                                    project_hierarchy_frame_action = frame_action;
                                }
                                ProjectHierarchyListAction::DragStarted(project_item_path) => {
                                    drag_started_project_item_path = Some(project_item_path);
                                }
                                ProjectHierarchyListAction::HoveredDropTarget(drop_target) => {
                                    hovered_drop_target_project_item_path = Some(drop_target);
                                }
                                ProjectHierarchyListAction::RenameSubmitted {
                                    project_item_path,
                                    project_item_type_id,
                                    edited_name,
                                } => {
                                    rename_project_item_submission = Some((project_item_path, project_item_type_id, edited_name));
                                }
                                ProjectHierarchyListAction::CancelTakeOver => {
                                    should_cancel_take_over = true;
                                }
                            }
                        }
                    }
                    ProjectHierarchyTakeOverState::EditProjectItemValue { .. }
                    | ProjectHierarchyTakeOverState::DeleteConfirmation { .. }
                    | ProjectHierarchyTakeOverState::PromoteSymbolConflict { .. } => {
                        match ProjectHierarchyTakeoverHostView::new(
                            self.app_context.clone(),
                            project_hierarchy_view_data.opened_project_info.as_ref(),
                            &tree_entries,
                            &take_over_state,
                        )
                        .show(user_interface)
                        {
                            ProjectHierarchyTakeoverHostAction::None => {}
                            ProjectHierarchyTakeoverHostAction::Cancel => {
                                should_cancel_take_over = true;
                            }
                            ProjectHierarchyTakeoverHostAction::DeleteProjectItems(project_item_paths) => {
                                delete_confirmation_project_item_paths = Some(project_item_paths);
                            }
                            ProjectHierarchyTakeoverHostAction::PromoteSymbolOverwrite(project_item_paths) => {
                                promote_symbol_overwrite_project_item_paths = Some(project_item_paths);
                            }
                            ProjectHierarchyTakeoverHostAction::SubmitValueEdit {
                                project_item_path,
                                value_field_name,
                                validation_data_type_ref,
                                value_edit,
                            } => {
                                value_edit_project_item_submission = Some((project_item_path, value_field_name, validation_data_type_ref, value_edit));
                            }
                        }
                    }
                }
            })
            .response;

        let is_window_focused = self
            .app_context
            .window_focus_manager
            .is_window_focused(ProjectExplorerView::WINDOW_ID);
        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), ProjectExplorerView::WINDOW_ID);

        if is_window_focused && (is_delete_confirmation_active || is_promote_symbol_conflict_active) {
            if user_interface.input(|input_state| input_state.key_pressed(Key::Escape))
                || user_interface.input(|input_state| input_state.key_pressed(Key::Backspace))
            {
                should_cancel_take_over = true;
            }
        }

        if is_window_focused && is_delete_confirmation_active {
            if user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
                delete_confirmation_project_item_paths = self
                    .project_hierarchy_view_data
                    .read("Project hierarchy confirm delete by keyboard")
                    .and_then(|project_hierarchy_view_data| match project_hierarchy_view_data.take_over_state.clone() {
                        ProjectHierarchyTakeOverState::DeleteConfirmation { project_item_paths } => Some(project_item_paths),
                        _ => None,
                    });
            }
        }

        if is_window_focused && is_promote_symbol_conflict_active && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            promote_symbol_overwrite_project_item_paths = self
                .project_hierarchy_view_data
                .read("Project hierarchy confirm promote overwrite by keyboard")
                .and_then(|project_hierarchy_view_data| match project_hierarchy_view_data.take_over_state.clone() {
                    ProjectHierarchyTakeOverState::PromoteSymbolConflict { project_item_paths, .. } => Some(project_item_paths),
                    _ => None,
                });
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowUp))
        {
            let extend_selection = user_interface.input(|input_state| input_state.modifiers.shift);
            if ProjectHierarchyViewData::navigate_project_item_selection(
                self.project_hierarchy_view_data.clone(),
                ListNavigationDirection::Up,
                extend_selection,
            )
            .is_some()
            {
                self.focus_selected_project_items_in_struct_viewer();
            }
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowDown))
        {
            let extend_selection = user_interface.input(|input_state| input_state.modifiers.shift);
            if ProjectHierarchyViewData::navigate_project_item_selection(
                self.project_hierarchy_view_data.clone(),
                ListNavigationDirection::Down,
                extend_selection,
            )
            .is_some()
            {
                self.focus_selected_project_items_in_struct_viewer();
            }
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::Delete))
        {
            ProjectHierarchyViewData::request_delete_confirmation_for_selected_project_item(self.project_hierarchy_view_data.clone());
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| (input_state.modifiers.command || input_state.modifiers.ctrl) && input_state.key_pressed(Key::X))
        {
            if let Some(project_item_paths) = self
                .project_hierarchy_view_data
                .read("Project hierarchy keyboard cut selection")
                .map(|project_hierarchy_view_data| project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order())
                .filter(|project_item_paths| !project_item_paths.is_empty())
            {
                project_hierarchy_frame_action = ProjectHierarchyFrameAction::CutProjectItems(project_item_paths);
            }
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| (input_state.modifiers.command || input_state.modifiers.ctrl) && input_state.key_pressed(Key::C))
        {
            if let Some(project_item_paths) = self
                .project_hierarchy_view_data
                .read("Project hierarchy keyboard copy selection")
                .map(|project_hierarchy_view_data| project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order())
                .filter(|project_item_paths| !project_item_paths.is_empty())
            {
                project_hierarchy_frame_action = ProjectHierarchyFrameAction::CopyProjectItems(project_item_paths);
            }
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| (input_state.modifiers.command || input_state.modifiers.ctrl) && input_state.key_pressed(Key::V))
        {
            if let Some(target_project_item_path) = ProjectHierarchyViewData::get_selected_or_root_directory_path(self.project_hierarchy_view_data.clone()) {
                project_hierarchy_frame_action = ProjectHierarchyFrameAction::PasteProjectItems { target_project_item_path };
            }
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::F2))
        {
            ProjectHierarchyViewData::request_rename_for_selected_project_item(self.project_hierarchy_view_data.clone());
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::Space))
        {
            keyboard_activation_toggle_target = self
                .project_hierarchy_view_data
                .read("Project hierarchy keyboard activation toggle")
                .and_then(|project_hierarchy_view_data| {
                    let selected_project_item_paths = project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order();
                    if selected_project_item_paths.is_empty() {
                        return None;
                    }

                    let selected_project_items = project_hierarchy_view_data
                        .project_items
                        .iter()
                        .filter(|(project_item_ref, _)| selected_project_item_paths.contains(project_item_ref.get_project_item_path()))
                        .map(|(_, project_item)| project_item)
                        .collect::<Vec<&ProjectItem>>();
                    let should_activate = selected_project_items
                        .iter()
                        .any(|project_item| !project_item.get_is_activated());

                    Some((selected_project_item_paths, should_activate))
                });
        }

        if !is_delete_confirmation_active
            && !is_value_edit_take_over_active
            && ProjectHierarchyViewData::set_visible_preview_project_item_paths(self.project_hierarchy_view_data.clone(), visible_preview_project_item_paths)
        {
            self.runtime_preview_controller()
                .sync_project_item_virtual_snapshot(project_read_interval);
        }

        if should_cancel_take_over {
            if let Some(project_item_path) = self
                .project_hierarchy_view_data
                .read("Project hierarchy clear inline rename state on cancel")
                .and_then(|project_hierarchy_view_data| match &project_hierarchy_view_data.take_over_state {
                    ProjectHierarchyTakeOverState::RenameProjectItem { project_item_path, .. } => Some(project_item_path.clone()),
                    _ => None,
                })
            {
                ProjectHierarchyListView::clear_project_item_rename_state(user_interface, &project_item_path);
            }
            if let Some(project_item_path) = self
                .project_hierarchy_view_data
                .read("Project hierarchy clear value edit state on cancel")
                .and_then(|project_hierarchy_view_data| match &project_hierarchy_view_data.take_over_state {
                    ProjectHierarchyTakeOverState::EditProjectItemValue { project_item_path } => Some(project_item_path.clone()),
                    _ => None,
                })
            {
                ProjectHierarchyTakeoverHostView::clear_project_item_value_edit_state(user_interface, &project_item_path);
            }
            ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
        }

        if let Some(project_item_paths) = delete_confirmation_project_item_paths {
            ProjectHierarchyViewData::delete_project_items(self.project_hierarchy_view_data.clone(), self.app_context.clone(), project_item_paths);
        }

        if let Some(project_item_paths) = promote_symbol_overwrite_project_item_paths {
            let details_refresh_callback = self
                .details_focus()
                .build_project_item_details_refresh_callback(project_item_paths.clone());
            ProjectHierarchyViewData::promote_project_items_to_symbols(
                self.project_hierarchy_view_data.clone(),
                self.app_context.clone(),
                project_item_paths,
                true,
                Some(details_refresh_callback),
            );
        }

        if let Some((project_item_path, project_item_type_id, edited_name)) = rename_project_item_submission {
            ProjectHierarchyListView::clear_project_item_rename_state(user_interface, &project_item_path);

            if let Some(project_item_rename_request) = ProjectItemRenameRequestBuilder::build(&project_item_path, &project_item_type_id, edited_name.trim()) {
                let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();
                let app_context = self.app_context.clone();
                let previous_project_item_path = project_item_path.clone();

                project_item_rename_request.send(&self.app_context.engine_unprivileged_state, move |project_items_rename_response| {
                    if !project_items_rename_response.success {
                        log::warn!("Project item rename command failed in hierarchy F2 rename flow.");
                        return;
                    }

                    ProjectHierarchyViewData::finish_project_item_rename(
                        project_hierarchy_view_data.clone(),
                        &previous_project_item_path,
                        &project_items_rename_response.renamed_project_item_path,
                    );
                    ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
                });
            }

            ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
        }

        if let Some((project_item_path, value_field_name, validation_data_type_ref, value_edit)) = value_edit_project_item_submission {
            match self
                .app_context
                .engine_unprivileged_state
                .deanonymize_value_string(&validation_data_type_ref, &value_edit)
            {
                Ok(_) => {
                    ProjectItemsWriteValueRequest {
                        project_item_path: project_item_path.clone(),
                        field_name: value_field_name,
                        anonymous_value_string: value_edit,
                    }
                    .send(&self.app_context.engine_unprivileged_state, |project_items_write_value_response| {
                        if !project_items_write_value_response.success {
                            log::warn!("Project item write-value command failed while committing value edit takeover.");
                        }
                    });
                    ProjectHierarchyTakeoverHostView::clear_project_item_value_edit_state(user_interface, &project_item_path);
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }
                Err(error) => {
                    log::warn!("Failed to commit project hierarchy runtime value edit: {}", error);
                }
            }
        }

        if let Some((project_item_paths, is_activated)) = keyboard_activation_toggle_target {
            ProjectHierarchyViewData::set_project_item_activation(
                self.project_hierarchy_view_data.clone(),
                self.app_context.clone(),
                project_item_paths,
                is_activated,
            );
        }

        if let Some(drag_started_project_item_path) = drag_started_project_item_path.clone() {
            ProjectHierarchyViewData::begin_reorder_drag(self.project_hierarchy_view_data.clone(), drag_started_project_item_path);
        }

        let persisted_dragged_project_item_paths = self
            .project_hierarchy_view_data
            .read("Project hierarchy check active drag")
            .and_then(|project_hierarchy_view_data| project_hierarchy_view_data.dragged_project_item_paths.clone());
        let active_dragged_project_item_paths = drag_started_project_item_path
            .map(|drag_started_project_item_path| vec![drag_started_project_item_path])
            .or(persisted_dragged_project_item_paths);

        if active_dragged_project_item_paths.is_some() {
            user_interface.output_mut(|platform_output| {
                platform_output.cursor_icon = CursorIcon::Move;
            });
        }

        if user_interface.input(|input_state| input_state.pointer.any_released()) {
            if active_dragged_project_item_paths.is_some() {
                if let Some(drop_target_project_item_path) = hovered_drop_target_project_item_path {
                    ProjectHierarchyViewData::commit_reorder_drop(
                        self.project_hierarchy_view_data.clone(),
                        self.app_context.clone(),
                        drop_target_project_item_path,
                    );
                } else {
                    ProjectHierarchyViewData::cancel_reorder_drag(self.project_hierarchy_view_data.clone());
                }
            }
        }

        let has_blocking_take_over = is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active;

        match project_hierarchy_frame_action {
            ProjectHierarchyFrameAction::None => {}
            ProjectHierarchyFrameAction::SelectProjectItem {
                project_item_path,
                additive_selection,
                range_selection,
            } => {
                if is_rename_take_over_active {
                    ProjectHierarchyListView::clear_active_project_item_rename_state(user_interface, self.project_hierarchy_view_data.clone());
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }
                if is_value_edit_take_over_active {
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }

                ProjectHierarchyViewData::select_project_item(self.project_hierarchy_view_data.clone(), project_item_path, additive_selection, range_selection);
                self.focus_selected_project_items_in_struct_viewer();
            }
            ProjectHierarchyFrameAction::ToggleDirectoryExpansion(project_item_path) => {
                if is_rename_take_over_active {
                    ProjectHierarchyListView::clear_active_project_item_rename_state(user_interface, self.project_hierarchy_view_data.clone());
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }
                if is_value_edit_take_over_active {
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }

                ProjectHierarchyViewData::toggle_directory_expansion(self.project_hierarchy_view_data.clone(), project_item_path);
            }
            ProjectHierarchyFrameAction::SetProjectItemActivation(project_item_path, is_activated) => {
                if has_blocking_take_over {
                    return response;
                }

                let project_item_paths = self
                    .project_hierarchy_view_data
                    .read("Project hierarchy checkbox activation selection")
                    .map(|project_hierarchy_view_data| {
                        if project_hierarchy_view_data
                            .selected_project_item_paths
                            .contains(&project_item_path)
                        {
                            project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order()
                        } else {
                            vec![project_item_path.clone()]
                        }
                    })
                    .unwrap_or_else(|| vec![project_item_path.clone()]);
                ProjectHierarchyViewData::set_project_item_activation(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    project_item_paths,
                    is_activated,
                );
            }
            ProjectHierarchyFrameAction::CreateProjectItem {
                target_project_item_path,
                create_item_kind,
            } => {
                if has_blocking_take_over {
                    return response;
                }

                ProjectHierarchyViewData::create_project_item(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    target_project_item_path,
                    create_item_kind,
                );
            }
            ProjectHierarchyFrameAction::CopyProjectItems(project_item_paths) => {
                if has_blocking_take_over {
                    return response;
                }

                ProjectHierarchyViewData::copy_project_items(self.project_hierarchy_view_data.clone(), project_item_paths);
            }
            ProjectHierarchyFrameAction::CutProjectItems(project_item_paths) => {
                if has_blocking_take_over {
                    return response;
                }

                ProjectHierarchyViewData::cut_project_items(self.project_hierarchy_view_data.clone(), project_item_paths);
            }
            ProjectHierarchyFrameAction::PasteProjectItems { target_project_item_path } => {
                if has_blocking_take_over {
                    return response;
                }

                ProjectHierarchyViewData::paste_project_item_clipboard(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    target_project_item_path,
                );
            }
            ProjectHierarchyFrameAction::OpenPointerScannerForAddress {
                address,
                module_name,
                data_type_id,
            } => {
                if has_blocking_take_over {
                    return response;
                }

                self.focus_pointer_scanner_for_address(address, &module_name, &data_type_id);
            }
            ProjectHierarchyFrameAction::OpenMemoryViewerForAddress {
                address,
                module_name,
                selection_byte_count,
            } => {
                if has_blocking_take_over {
                    return response;
                }

                self.focus_memory_viewer_for_address(address, &module_name, selection_byte_count);
            }
            ProjectHierarchyFrameAction::OpenCodeViewerForAddress { address, module_name } => {
                if has_blocking_take_over {
                    return response;
                }

                self.focus_code_viewer_for_address(address, &module_name);
            }
            ProjectHierarchyFrameAction::PromoteToSymbol {
                project_item_paths,
                overwrite_conflicting_symbols,
            } => {
                if is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                let details_refresh_callback = self
                    .details_focus()
                    .build_project_item_details_refresh_callback(project_item_paths.clone());
                ProjectHierarchyViewData::promote_project_items_to_symbols(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    project_item_paths,
                    overwrite_conflicting_symbols,
                    Some(details_refresh_callback),
                );
            }
            ProjectHierarchyFrameAction::StripSymbolInformation { project_item_paths } => {
                if is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                let details_refresh_callback = self
                    .details_focus()
                    .build_project_item_details_refresh_callback(project_item_paths.clone());

                Self::strip_symbol_information_from_project_items(
                    self.app_context.clone(),
                    self.project_hierarchy_view_data.clone(),
                    project_item_paths,
                    Some(details_refresh_callback),
                );
            }
            ProjectHierarchyFrameAction::RequestRename(project_item_path) => {
                if is_promote_symbol_conflict_active || is_value_edit_take_over_active {
                    return response;
                }

                if is_rename_take_over_active {
                    ProjectHierarchyListView::clear_active_project_item_rename_state(user_interface, self.project_hierarchy_view_data.clone());
                }

                ProjectHierarchyViewData::request_rename_for_project_item(self.project_hierarchy_view_data.clone(), project_item_path);
            }
            ProjectHierarchyFrameAction::RequestValueEdit(project_item_path) => {
                if is_promote_symbol_conflict_active {
                    return response;
                }

                if is_rename_take_over_active {
                    ProjectHierarchyListView::clear_active_project_item_rename_state(user_interface, self.project_hierarchy_view_data.clone());
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }

                ProjectHierarchyViewData::request_value_edit_for_project_item(self.project_hierarchy_view_data.clone(), project_item_path);
            }
            ProjectHierarchyFrameAction::RequestDeleteConfirmation(project_item_paths) => {
                if has_blocking_take_over {
                    return response;
                }

                ProjectHierarchyViewData::request_delete_confirmation(self.project_hierarchy_view_data.clone(), project_item_paths);
            }
        }

        response
    }
}

impl ProjectHierarchyView {
    const SCAN_SETTINGS_SYNC_INTERVAL_MS: u64 = 1_000;

    fn show_toolbar_add_menu(
        &self,
        project_hierarchy_frame_action: &mut ProjectHierarchyFrameAction,
        user_interface: &mut Ui,
        menu_target: Option<&ProjectHierarchyMenuTarget>,
        menu_position: Option<Pos2>,
    ) {
        let Some(ProjectHierarchyMenuTarget::ToolbarAdd { target_project_item_path }) = menu_target else {
            return;
        };

        let Some(menu_position) = menu_position else {
            return;
        };
        let create_project_item_menu_labels = ProjectHierarchyCreateItemMenuView::labels();
        let project_item_menu_width =
            ContextMenuSizing::width_for_labels(self.app_context.as_ref(), user_interface, create_project_item_menu_labels.iter().copied());
        let mut open = true;
        ContextMenu::new(
            self.app_context.clone(),
            "project_hierarchy_toolbar_add_menu",
            menu_position,
            |user_interface, should_close| {
                let create_project_item_action = ProjectHierarchyCreateItemMenuView::show_items(
                    self.app_context.clone(),
                    user_interface,
                    target_project_item_path,
                    project_item_menu_width,
                    should_close,
                );

                if create_project_item_action != ProjectHierarchyFrameAction::None {
                    *project_hierarchy_frame_action = create_project_item_action;
                }
            },
        )
        .width(project_item_menu_width)
        .corner_radius(8)
        .show(user_interface, &mut open);

        if !open {
            ProjectHierarchyViewData::hide_menu(self.project_hierarchy_view_data.clone());
        }
    }

    fn sync_scan_settings_if_needed(&self) {
        let should_request_scan_settings = self
            .project_hierarchy_view_data
            .write("Project hierarchy scan settings sync check")
            .map(|mut project_hierarchy_view_data| {
                let now = Instant::now();
                let has_sync_interval_elapsed = project_hierarchy_view_data
                    .last_scan_settings_sync_timestamp
                    .map(|last_scan_settings_sync_timestamp| {
                        now.duration_since(last_scan_settings_sync_timestamp) >= Duration::from_millis(Self::SCAN_SETTINGS_SYNC_INTERVAL_MS)
                    })
                    .unwrap_or(true);

                if project_hierarchy_view_data.is_querying_scan_settings || !has_sync_interval_elapsed {
                    return false;
                }

                project_hierarchy_view_data.is_querying_scan_settings = true;
                project_hierarchy_view_data.last_scan_settings_sync_timestamp = Some(now);

                true
            })
            .unwrap_or(false);

        if !should_request_scan_settings {
            return;
        }

        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();
        let scan_settings_list_request = ScanSettingsListRequest {};
        scan_settings_list_request.send(&self.app_context.engine_unprivileged_state, move |scan_settings_list_response| {
            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy scan settings sync response") {
                if let Ok(scan_settings) = scan_settings_list_response.scan_settings {
                    project_hierarchy_view_data.project_read_interval_ms = scan_settings.project_read_interval_ms;
                }

                project_hierarchy_view_data.is_querying_scan_settings = false;
            }
        });
    }

    fn focus_selected_project_items_in_struct_viewer(&self) {
        self.details_focus().focus_selected_project_items();
    }

    fn strip_symbol_information_from_project_items(
        app_context: Arc<AppContext>,
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_paths: Vec<PathBuf>,
        details_refresh_callback: Option<Arc<dyn Fn() + Send + Sync>>,
    ) {
        let project_item_paths = ProjectHierarchyViewData::filter_strippable_symbol_project_item_paths(project_hierarchy_view_data, project_item_paths);

        if project_item_paths.is_empty() {
            return;
        }

        let project_manager = app_context.engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project_lock.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!(
                    "Failed to acquire opened project lock while stripping project item symbol information: {}",
                    error
                );
                return;
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            log::warn!("Cannot strip project item symbol information without an opened project.");
            return;
        };
        let project_symbol_catalog = opened_project
            .get_project_info()
            .get_project_symbol_catalog()
            .clone();
        let mut has_persisted_property_edits = false;

        for project_item_path in &project_item_paths {
            let project_item_ref = ProjectItemRef::new(project_item_path.clone());
            let Some(project_item) = opened_project.get_project_item_mut(&project_item_ref) else {
                log::warn!("Cannot strip symbol information, project item was not found: {:?}", project_item_path);
                continue;
            };

            if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
                continue;
            }

            let address_target = ProjectItemTypeAddress::get_address_target(project_item);

            if !address_target.has_symbolic_offsets() {
                continue;
            }

            let Some(stripped_address_target) = address_target.strip_symbolic_offsets(&project_symbol_catalog) else {
                log::warn!("Cannot strip unresolved symbol information from project item: {:?}", project_item_path);
                continue;
            };

            if stripped_address_target != address_target {
                ProjectItemTypeAddress::set_address_target(project_item, stripped_address_target);
                project_item.set_has_unsaved_changes(true);
                has_persisted_property_edits = true;
            }
        }

        if !has_persisted_property_edits {
            return;
        }

        opened_project
            .get_project_info_mut()
            .set_has_unsaved_changes(true);
        drop(opened_project_guard);

        let project_save_request = ProjectSaveRequest {};

        project_save_request.send(&app_context.engine_unprivileged_state, |project_save_response| {
            if !project_save_response.success {
                log::error!("Failed to persist stripped project item symbol information through project save command.");
            }
        });
        project_manager.notify_project_items_changed();

        if let Some(details_refresh_callback) = details_refresh_callback {
            details_refresh_callback();
        }
    }

    fn focus_pointer_scanner_for_address(
        &self,
        address: u64,
        module_name: &str,
        data_type_id: &str,
    ) {
        let (resolved_target_address, resolved_target_module_name) = if module_name.trim().is_empty() {
            (address, String::new())
        } else if try_resolve_virtual_module_address(module_name, address).is_some() {
            (address, module_name.to_string())
        } else if let Some(resolved_absolute_address) = ProjectItemDetails::dispatch_memory_query_request(&self.app_context.engine_unprivileged_state)
            .and_then(|memory_query_response| ProjectItemDetails::resolve_module_relative_address(&memory_query_response.modules, address, module_name))
        {
            (resolved_absolute_address, String::new())
        } else {
            log::warn!(
                "Failed to resolve pointer scanner target for module-relative address {}+0x{:X}; falling back to unresolved offset.",
                module_name,
                address
            );
            (address, module_name.to_string())
        };

        PointerScannerViewData::set_scan_target_from_project_address(
            self.pointer_scanner_view_data.clone(),
            resolved_target_address,
            &resolved_target_module_name,
            data_type_id,
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(PointerScannerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(PointerScannerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening the pointer scanner: {}", error);
            }
        }
    }

    fn focus_memory_viewer_for_address(
        &self,
        address: u64,
        module_name: &str,
        selection_byte_count: u64,
    ) {
        MemoryViewerViewData::request_focus_address_range(
            self.memory_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            address,
            module_name.to_string(),
            selection_byte_count,
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(MemoryViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(MemoryViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening the memory viewer: {}", error);
            }
        }
    }

    fn focus_code_viewer_for_address(
        &self,
        address: u64,
        module_name: &str,
    ) {
        CodeViewerViewData::request_focus_address(
            self.code_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            address,
            module_name.to_string(),
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(CodeViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(CodeViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening the code viewer: {}", error);
            }
        }
    }

    fn refresh_if_project_changed(&self) {
        let (opened_project_directory_path, opened_project_item_signatures, opened_project_sort_order) = match self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .read()
        {
            Ok(opened_project_guard) => opened_project_guard
                .as_ref()
                .map(|opened_project| {
                    let opened_project_directory_path = opened_project.get_project_info().get_project_directory();
                    let opened_project_item_signatures = Self::collect_project_item_change_signatures(opened_project.get_project_items().iter());
                    let opened_project_sort_order = opened_project
                        .get_project_info()
                        .get_project_manifest()
                        .get_project_item_sort_order()
                        .iter()
                        .cloned()
                        .collect::<Vec<PathBuf>>();

                    (opened_project_directory_path, opened_project_item_signatures, opened_project_sort_order)
                })
                .unwrap_or((None, HashMap::new(), Vec::new())),
            Err(error) => {
                log::error!("Failed to acquire opened project lock for hierarchy refresh check: {}", error);
                (None, HashMap::new(), Vec::new())
            }
        };

        let (loaded_project_directory_path, loaded_project_item_signatures, loaded_project_sort_order) = self
            .project_hierarchy_view_data
            .read("Project hierarchy refresh check")
            .map(|project_hierarchy_view_data| {
                let loaded_project_directory_path = project_hierarchy_view_data
                    .opened_project_info
                    .as_ref()
                    .and_then(|project_info| project_info.get_project_directory());
                let loaded_project_item_signatures = Self::collect_project_item_change_signatures(
                    project_hierarchy_view_data
                        .project_items
                        .iter()
                        .map(|(project_item_ref, project_item)| (project_item_ref, project_item)),
                );
                let loaded_project_sort_order = project_hierarchy_view_data
                    .opened_project_info
                    .as_ref()
                    .map(|project_info| {
                        project_info
                            .get_project_manifest()
                            .get_project_item_sort_order()
                            .iter()
                            .cloned()
                            .collect::<Vec<PathBuf>>()
                    })
                    .unwrap_or_default();

                (loaded_project_directory_path, loaded_project_item_signatures, loaded_project_sort_order)
            })
            .unwrap_or((None, HashMap::new(), Vec::new()));

        let project_directory_changed = opened_project_directory_path != loaded_project_directory_path;
        let project_items_changed = opened_project_item_signatures != loaded_project_item_signatures;
        let sort_order_changed = opened_project_sort_order != loaded_project_sort_order;

        if project_directory_changed || project_items_changed || sort_order_changed {
            ProjectHierarchyViewData::refresh_project_items(self.project_hierarchy_view_data.clone(), self.app_context.clone());
        }
    }

    fn collect_project_item_change_signatures<'a>(
        project_items: impl IntoIterator<Item = (&'a ProjectItemRef, &'a ProjectItem)>
    ) -> HashMap<PathBuf, ProjectItemChangeSignature> {
        project_items
            .into_iter()
            .map(|(project_item_ref, project_item)| {
                (
                    project_item_ref.get_project_item_path().clone(),
                    ProjectItemChangeSignature {
                        project_item_type_id: project_item
                            .get_item_type()
                            .get_project_item_type_id()
                            .to_string(),
                        project_item_name: project_item.get_field_name(),
                        project_item_description: project_item.get_field_description(),
                        project_item_properties: project_item
                            .get_properties()
                            .get_fields()
                            .iter()
                            .map(|field| (field.get_name().to_string(), field.get_bytes()))
                            .collect(),
                    },
                )
            })
            .collect()
    }
}
