use crate::{
    app_context::AppContext,
    ui::widgets::controls::button::Button,
    views::project_explorer::project_hierarchy::{
        project_hierarchy_toolbar_view::ProjectHierarchyToolbarView,
        project_item_entry_view::ProjectItemEntryView,
        view_data::{
            project_hierarchy_frame_action::ProjectHierarchyFrameAction, project_hierarchy_pending_operation::ProjectHierarchyPendingOperation,
            project_hierarchy_take_over_state::ProjectHierarchyTakeOverState, project_hierarchy_view_data::ProjectHierarchyViewData,
        },
    },
    views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData,
};
use eframe::egui::{Align, CursorIcon, Layout, Response, ScrollArea, TextureHandle, Ui, Widget, vec2};
use epaint::{Color32, CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use squalr_engine_api::structures::structs::valued_struct_field::ValuedStructField;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectHierarchyView {
    app_context: Arc<AppContext>,
    project_hierarchy_toolbar_view: ProjectHierarchyToolbarView,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
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
        ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data.clone(), app_context.clone());

        Self {
            app_context,
            project_hierarchy_toolbar_view,
            project_hierarchy_view_data,
            struct_viewer_view_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectHierarchyView;
    use squalr_engine_api::structures::data_types::built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64};
    use squalr_engine_api::structures::projects::project_items::built_in_types::{
        project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
    };
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use squalr_engine_api::structures::structs::valued_struct_field::{ValuedStructField, ValuedStructFieldData};
    use std::path::{Path, PathBuf};

    #[test]
    fn build_memory_write_request_for_address_item_address_edit_returns_request() {
        let mut project_item = ProjectItemTypeAddress::new_project_item("player_health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        let expected_module_name = ProjectItemTypeAddress::get_field_module(&mut project_item);
        let edited_field = ValuedStructField::new(
            ProjectItemTypeAddress::PROPERTY_ADDRESS.to_string(),
            ValuedStructFieldData::Value(DataTypeU64::get_value_from_primitive(0xABCD)),
            false,
        );

        let memory_write_request = ProjectHierarchyView::build_memory_write_request_for_project_item_edit(&mut project_item, &edited_field);

        assert!(memory_write_request.is_some());
        let memory_write_request = memory_write_request.unwrap_or_else(|| panic!("Expected memory write request for address edit."));
        assert_eq!(memory_write_request.address, 0x1234);
        assert_eq!(memory_write_request.module_name, expected_module_name);
        assert_eq!(memory_write_request.value, 0xABCDu64.to_le_bytes().to_vec());
    }

    #[test]
    fn build_memory_write_request_for_address_item_non_address_edit_returns_none() {
        let mut project_item = ProjectItemTypeAddress::new_project_item("player_health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        let edited_field = ValuedStructField::new(
            ProjectItemTypeAddress::PROPERTY_MODULE.to_string(),
            ValuedStructFieldData::Value(DataTypeStringUtf8::get_value_from_primitive_string("new_module.exe")),
            false,
        );

        let memory_write_request = ProjectHierarchyView::build_memory_write_request_for_project_item_edit(&mut project_item, &edited_field);

        assert!(memory_write_request.is_none());
    }

    #[test]
    fn build_memory_write_request_for_non_address_item_address_edit_returns_none() {
        let project_item_ref = ProjectItemRef::new(PathBuf::from("project/folder"));
        let mut project_item = ProjectItemTypeDirectory::new_project_item(&project_item_ref);
        let edited_field = ValuedStructField::new(
            ProjectItemTypeAddress::PROPERTY_ADDRESS.to_string(),
            ValuedStructFieldData::Value(DataTypeU64::get_value_from_primitive(0xABCD)),
            false,
        );

        let memory_write_request = ProjectHierarchyView::build_memory_write_request_for_project_item_edit(&mut project_item, &edited_field);

        assert!(memory_write_request.is_none());
    }

    #[test]
    fn build_project_item_rename_request_for_directory_uses_edited_name_without_extension() {
        let project_item_path = Path::new("C:/Projects/TestProject/project_items/Folder");
        let rename_request =
            ProjectHierarchyView::build_project_item_rename_request(project_item_path, ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID, "Renamed Folder");

        assert!(rename_request.is_some());
        let rename_request = rename_request.unwrap_or_else(|| panic!("Expected rename request for directory item."));
        assert_eq!(rename_request.project_item_name, "Renamed Folder".to_string());
    }

    #[test]
    fn build_project_item_rename_request_for_file_appends_json_extension() {
        let project_item_path = Path::new("C:/Projects/TestProject/project_items/health.json");
        let rename_request =
            ProjectHierarchyView::build_project_item_rename_request(project_item_path, ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID, "player_health");

        assert!(rename_request.is_some());
        let rename_request = rename_request.unwrap_or_else(|| panic!("Expected rename request for file item."));
        assert_eq!(rename_request.project_item_name, "player_health.json".to_string());
    }

    #[test]
    fn build_project_item_rename_request_returns_none_when_name_is_unchanged() {
        let project_item_path = Path::new("C:/Projects/TestProject/project_items/health.json");
        let rename_request =
            ProjectHierarchyView::build_project_item_rename_request(project_item_path, ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID, "health.json");

        assert!(rename_request.is_none());
    }

    #[test]
    fn should_apply_struct_field_edit_to_project_item_returns_false_for_directory_name_edits() {
        let should_apply_struct_field_edit = ProjectHierarchyView::should_apply_struct_field_edit_to_project_item(
            ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID,
            squalr_engine_api::structures::projects::project_items::project_item::ProjectItem::PROPERTY_NAME,
        );

        assert!(!should_apply_struct_field_edit);
    }

    #[test]
    fn should_apply_struct_field_edit_to_project_item_returns_true_for_file_name_edits() {
        let should_apply_struct_field_edit = ProjectHierarchyView::should_apply_struct_field_edit_to_project_item(
            ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID,
            squalr_engine_api::structures::projects::project_items::project_item::ProjectItem::PROPERTY_NAME,
        );

        assert!(should_apply_struct_field_edit);
    }

    #[test]
    fn should_apply_struct_field_edit_to_project_item_returns_true_for_non_name_edits() {
        let should_apply_struct_field_edit = ProjectHierarchyView::should_apply_struct_field_edit_to_project_item(
            ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID,
            ProjectItemTypeAddress::PROPERTY_MODULE,
        );

        assert!(should_apply_struct_field_edit);
    }
}

impl Widget for ProjectHierarchyView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        self.refresh_if_project_changed();

        let project_hierarchy_toolbar_view = self.project_hierarchy_toolbar_view.clone();
        let mut project_hierarchy_frame_action = ProjectHierarchyFrameAction::None;
        let mut drag_started_project_item_path: Option<PathBuf> = None;
        let mut hovered_drop_target_project_item_path: Option<PathBuf> = None;
        let mut should_cancel_take_over = false;
        let mut delete_confirmation_project_item_paths: Option<Vec<std::path::PathBuf>> = None;
        let mut keyboard_activation_toggle_target: Option<(Vec<PathBuf>, bool)> = None;
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let project_hierarchy_view_data = match self.project_hierarchy_view_data.read("Project hierarchy view") {
                    Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                    None => return,
                };
                let take_over_state = project_hierarchy_view_data.take_over_state.clone();
                let tree_entries = project_hierarchy_view_data.tree_entries.clone();
                let selected_project_item_paths = project_hierarchy_view_data.selected_project_item_paths.clone();
                let dragged_project_item_paths = project_hierarchy_view_data.dragged_project_item_paths.clone();
                let pending_operation = project_hierarchy_view_data.pending_operation.clone();

                user_interface.add(project_hierarchy_toolbar_view);

                match pending_operation {
                    ProjectHierarchyPendingOperation::Deleting => {
                        user_interface.label("Deleting project item(s)...");
                    }
                    ProjectHierarchyPendingOperation::Reordering => {
                        user_interface.label("Reordering project item(s)...");
                    }
                    _ => {}
                }

                match take_over_state {
                    ProjectHierarchyTakeOverState::None => {
                        ScrollArea::vertical()
                            .id_salt("project_hierarchy")
                            .auto_shrink([false, false])
                            .show(user_interface, |user_interface| {
                                for tree_entry in &tree_entries {
                                    let is_selected = selected_project_item_paths.contains(&tree_entry.project_item_path);
                                    let icon = Self::resolve_tree_entry_icon(
                                        self.app_context.clone(),
                                        tree_entry
                                            .project_item
                                            .get_item_type()
                                            .get_project_item_type_id(),
                                    );

                                    let row_response = user_interface.add(ProjectItemEntryView::new(
                                        self.app_context.clone(),
                                        &tree_entry.project_item_path,
                                        &tree_entry.display_name,
                                        &tree_entry.preview_value,
                                        tree_entry.is_activated,
                                        tree_entry.depth,
                                        icon,
                                        is_selected,
                                        tree_entry.is_directory,
                                        tree_entry.has_children,
                                        tree_entry.is_expanded,
                                        &mut project_hierarchy_frame_action,
                                    ));

                                    if row_response.drag_started() {
                                        drag_started_project_item_path = Some(tree_entry.project_item_path.clone());
                                    }

                                    let tree_entry_project_item_path = tree_entry.project_item_path.clone();
                                    row_response.context_menu(|user_interface| {
                                        if user_interface.button("New Folder").clicked() {
                                            project_hierarchy_frame_action = ProjectHierarchyFrameAction::CreateDirectory(tree_entry_project_item_path.clone());
                                            user_interface.close();
                                        }

                                        if user_interface.button("Delete").clicked() {
                                            let selected_project_item_paths_in_order = self
                                                .project_hierarchy_view_data
                                                .read("Project hierarchy selected project items for context menu delete")
                                                .map(|project_hierarchy_view_data| {
                                                    project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order()
                                                })
                                                .unwrap_or_default();
                                            let project_item_paths_for_delete = if selected_project_item_paths_in_order.contains(&tree_entry_project_item_path)
                                                && selected_project_item_paths_in_order.len() > 1
                                            {
                                                selected_project_item_paths_in_order
                                            } else {
                                                vec![tree_entry_project_item_path.clone()]
                                            };
                                            project_hierarchy_frame_action =
                                                ProjectHierarchyFrameAction::RequestDeleteConfirmation(project_item_paths_for_delete);
                                            user_interface.close();
                                        }
                                    });

                                    let active_dragged_project_item_paths = drag_started_project_item_path
                                        .as_ref()
                                        .map(|drag_started_project_item_path| vec![drag_started_project_item_path.clone()])
                                        .or(dragged_project_item_paths.clone());

                                    if let Some(active_dragged_project_item_paths) = active_dragged_project_item_paths {
                                        if !active_dragged_project_item_paths.contains(&tree_entry.project_item_path) && row_response.contains_pointer() {
                                            hovered_drop_target_project_item_path = Some(tree_entry.project_item_path.clone());
                                            user_interface
                                                .painter()
                                                .rect_filled(row_response.rect, CornerRadius::ZERO, self.app_context.theme.hover_tint);
                                            user_interface.painter().rect_stroke(
                                                row_response.rect,
                                                CornerRadius::ZERO,
                                                Stroke::new(1.0, self.app_context.theme.selected_border),
                                                StrokeKind::Inside,
                                            );
                                        }
                                    }
                                }
                            });
                    }
                    ProjectHierarchyTakeOverState::DeleteConfirmation { project_item_paths } => {
                        user_interface.label("Confirm deletion of selected project item(s).");

                        ScrollArea::vertical()
                            .id_salt("project_hierarchy_delete_confirmation")
                            .max_height(160.0)
                            .auto_shrink([false, false])
                            .show(user_interface, |user_interface| {
                                for project_item_path in &project_item_paths {
                                    let project_item_name = project_item_path
                                        .file_name()
                                        .and_then(|value| value.to_str())
                                        .unwrap_or_default();
                                    user_interface.label(project_item_name);
                                }
                            });

                        user_interface.horizontal(|user_interface| {
                            let button_size = vec2(120.0, 28.0);
                            let button_cancel = user_interface.add_sized(
                                button_size,
                                Button::new_from_theme(&self.app_context.theme)
                                    .with_tooltip_text("Cancel project item deletion.")
                                    .background_color(Color32::TRANSPARENT),
                            );

                            if button_cancel.clicked() {
                                should_cancel_take_over = true;
                            }

                            let button_confirm_delete = user_interface.add_sized(
                                button_size,
                                Button::new_from_theme(&self.app_context.theme).with_tooltip_text("Permanently delete selected project item(s)."),
                            );

                            if button_confirm_delete.clicked() {
                                delete_confirmation_project_item_paths = Some(project_item_paths);
                            }
                        });
                    }
                }
            })
            .response;

        if user_interface.input(|input_state| input_state.key_pressed(eframe::egui::Key::Delete)) {
            ProjectHierarchyViewData::request_delete_confirmation_for_selected_project_item(self.project_hierarchy_view_data.clone());
        }

        if user_interface.input(|input_state| input_state.key_pressed(eframe::egui::Key::Space)) {
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

        if should_cancel_take_over {
            ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
        }

        if let Some(project_item_paths) = delete_confirmation_project_item_paths {
            ProjectHierarchyViewData::delete_project_items(self.project_hierarchy_view_data.clone(), self.app_context.clone(), project_item_paths);
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

        match project_hierarchy_frame_action {
            ProjectHierarchyFrameAction::None => {}
            ProjectHierarchyFrameAction::SelectProjectItem {
                project_item_path,
                additive_selection,
                range_selection,
            } => {
                ProjectHierarchyViewData::select_project_item(self.project_hierarchy_view_data.clone(), project_item_path, additive_selection, range_selection);
                self.focus_selected_project_items_in_struct_viewer();
            }
            ProjectHierarchyFrameAction::ToggleDirectoryExpansion(project_item_path) => {
                ProjectHierarchyViewData::toggle_directory_expansion(self.project_hierarchy_view_data.clone(), project_item_path);
            }
            ProjectHierarchyFrameAction::SetProjectItemActivation(project_item_path, is_activated) => {
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
            ProjectHierarchyFrameAction::CreateDirectory(target_project_item_path) => {
                ProjectHierarchyViewData::create_directory(self.project_hierarchy_view_data.clone(), self.app_context.clone(), target_project_item_path);
            }
            ProjectHierarchyFrameAction::RequestDeleteConfirmation(project_item_paths) => {
                ProjectHierarchyViewData::request_delete_confirmation(self.project_hierarchy_view_data.clone(), project_item_paths);
            }
        }

        response
    }
}

impl ProjectHierarchyView {
    fn focus_selected_project_items_in_struct_viewer(&self) {
        let selected_project_item_paths = self
            .project_hierarchy_view_data
            .read("Project hierarchy selected project items for struct viewer focus")
            .map(|project_hierarchy_view_data| project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order())
            .unwrap_or_default();
        let selected_project_items = self
            .project_hierarchy_view_data
            .read("Project hierarchy selected project item data for struct viewer focus")
            .map(|project_hierarchy_view_data| {
                project_hierarchy_view_data
                    .project_items
                    .iter()
                    .filter(|(project_item_ref, _)| selected_project_item_paths.contains(project_item_ref.get_project_item_path()))
                    .map(|(_, project_item)| project_item.clone())
                    .collect::<Vec<ProjectItem>>()
            })
            .unwrap_or_default();

        if selected_project_item_paths.is_empty() || selected_project_items.is_empty() {
            StructViewerViewData::clear_focus(self.struct_viewer_view_data.clone());
            return;
        }

        let app_context = self.app_context.clone();
        let selected_project_item_paths_for_edit = selected_project_item_paths.clone();
        let callback = Arc::new(move |edited_field: ValuedStructField| {
            Self::apply_project_item_edits(app_context.clone(), selected_project_item_paths_for_edit.clone(), edited_field);
        });

        if selected_project_items.len() == 1 {
            if let Some(selected_project_item) = selected_project_items.into_iter().next() {
                StructViewerViewData::focus_valued_struct(self.struct_viewer_view_data.clone(), selected_project_item.get_properties().clone(), callback);
            }
        } else {
            let selected_project_item_properties = selected_project_items
                .into_iter()
                .map(|selected_project_item| selected_project_item.get_properties().clone())
                .collect::<Vec<_>>();
            StructViewerViewData::focus_valued_structs(self.struct_viewer_view_data.clone(), selected_project_item_properties, callback);
        }
    }

    fn apply_project_item_edits(
        app_context: Arc<AppContext>,
        project_item_paths: Vec<PathBuf>,
        edited_field: ValuedStructField,
    ) {
        if project_item_paths.is_empty() {
            return;
        }

        let project_manager = app_context.engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let mut memory_write_requests = Vec::new();
        let mut rename_requests = Vec::new();
        let mut has_persisted_property_edits = false;
        let edited_field_name = edited_field.get_name().to_string();
        let edited_name = if edited_field_name == ProjectItem::PROPERTY_NAME {
            Self::extract_string_value_from_edited_field(&edited_field)
        } else {
            None
        };

        let mut opened_project_guard = match opened_project_lock.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for struct viewer edit: {}", error);
                return;
            }
        };
        let opened_project = match opened_project_guard.as_mut() {
            Some(opened_project) => opened_project,
            None => {
                log::warn!("Cannot apply struct viewer edit without an opened project.");
                return;
            }
        };
        let root_project_item_path = opened_project
            .get_project_root_ref()
            .get_project_item_path()
            .clone();

        for project_item_path in &project_item_paths {
            if edited_field.get_name() == ProjectItem::PROPERTY_NAME && project_item_path == &root_project_item_path {
                log::debug!("Ignoring root project directory name edit in project hierarchy.");
                continue;
            }

            let project_item_ref = ProjectItemRef::new(project_item_path.clone());
            let project_item = match opened_project.get_project_item_mut(&project_item_ref) {
                Some(project_item) => project_item,
                None => {
                    log::warn!("Cannot apply struct viewer edit, project item was not found: {:?}", project_item_path);
                    continue;
                }
            };
            let project_item_type_id = project_item
                .get_item_type()
                .get_project_item_type_id()
                .to_string();
            let should_apply_field_edit = Self::should_apply_struct_field_edit_to_project_item(&project_item_type_id, &edited_field_name);

            if should_apply_field_edit {
                project_item.get_properties_mut().set_field_data(
                    edited_field.get_name(),
                    edited_field.get_field_data().clone(),
                    edited_field.get_is_read_only(),
                );
                project_item.set_has_unsaved_changes(true);
                has_persisted_property_edits = true;
            }

            if let Some(edited_name) = &edited_name {
                if let Some(project_items_rename_request) = Self::build_project_item_rename_request(project_item_path, &project_item_type_id, edited_name) {
                    rename_requests.push(project_items_rename_request);
                }
            }

            if let Some(memory_write_request) = Self::build_memory_write_request_for_project_item_edit(project_item, &edited_field) {
                memory_write_requests.push(memory_write_request);
            }
        }

        if !has_persisted_property_edits && rename_requests.is_empty() && memory_write_requests.is_empty() {
            return;
        }

        drop(opened_project_guard);

        if has_persisted_property_edits {
            if let Ok(mut opened_project_guard) = opened_project_lock.write() {
                if let Some(opened_project) = opened_project_guard.as_mut() {
                    opened_project
                        .get_project_info_mut()
                        .set_has_unsaved_changes(true);
                }
            }

            let project_save_request = ProjectSaveRequest {};

            project_save_request.send(&app_context.engine_unprivileged_state, |project_save_response| {
                if !project_save_response.success {
                    log::error!("Failed to persist project item edit through project save command.");
                }
            });
            project_manager.notify_project_items_changed();
        }

        for rename_request in rename_requests {
            rename_request.send(&app_context.engine_unprivileged_state, |project_items_rename_response| {
                if !project_items_rename_response.success {
                    log::warn!("Project item rename command failed while committing name edit.");
                }
            });
        }

        for memory_write_request in memory_write_requests {
            memory_write_request.send(&app_context.engine_unprivileged_state, |memory_write_response| {
                if !memory_write_response.success {
                    log::warn!("Project item address edit memory write command failed.");
                }
            });
        }
    }

    fn build_memory_write_request_for_project_item_edit(
        project_item: &mut ProjectItem,
        edited_field: &ValuedStructField,
    ) -> Option<MemoryWriteRequest> {
        if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            return None;
        }

        if edited_field.get_name() != ProjectItemTypeAddress::PROPERTY_ADDRESS {
            return None;
        }

        let edited_data_value = edited_field.get_data_value()?;
        let address = ProjectItemTypeAddress::get_field_address(project_item);
        let module_name = ProjectItemTypeAddress::get_field_module(project_item);

        Some(MemoryWriteRequest {
            address,
            module_name,
            value: edited_data_value.get_value_bytes().clone(),
        })
    }

    fn resolve_tree_entry_icon(
        app_context: Arc<AppContext>,
        project_item_type_id: &str,
    ) -> Option<TextureHandle> {
        let icon_library = &app_context.theme.icon_library;

        if project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
            Some(icon_library.icon_handle_file_system_open_folder.clone())
        } else if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            Some(icon_library.icon_handle_data_type_blue_blocks_8.clone())
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            Some(icon_library.icon_handle_project_pointer_type.clone())
        } else {
            Some(icon_library.icon_handle_data_type_unknown.clone())
        }
    }

    fn refresh_if_project_changed(&self) {
        let (opened_project_directory_path, opened_project_item_paths, opened_project_sort_order) = match self
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
                    let opened_project_item_paths = opened_project
                        .get_project_items()
                        .keys()
                        .map(|project_item_ref| project_item_ref.get_project_item_path().clone())
                        .collect::<HashSet<PathBuf>>();
                    let opened_project_sort_order = opened_project
                        .get_project_info()
                        .get_project_manifest()
                        .get_project_item_sort_order()
                        .iter()
                        .cloned()
                        .collect::<Vec<PathBuf>>();

                    (opened_project_directory_path, opened_project_item_paths, opened_project_sort_order)
                })
                .unwrap_or((None, HashSet::new(), Vec::new())),
            Err(error) => {
                log::error!("Failed to acquire opened project lock for hierarchy refresh check: {}", error);
                (None, HashSet::new(), Vec::new())
            }
        };

        let (loaded_project_directory_path, loaded_project_item_paths, loaded_project_sort_order) = self
            .project_hierarchy_view_data
            .read("Project hierarchy refresh check")
            .map(|project_hierarchy_view_data| {
                let loaded_project_directory_path = project_hierarchy_view_data
                    .opened_project_info
                    .as_ref()
                    .and_then(|project_info| project_info.get_project_directory());
                let loaded_project_item_paths = project_hierarchy_view_data
                    .project_items
                    .iter()
                    .map(|(project_item_ref, _)| project_item_ref.get_project_item_path().clone())
                    .collect::<HashSet<PathBuf>>();
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

                (loaded_project_directory_path, loaded_project_item_paths, loaded_project_sort_order)
            })
            .unwrap_or((None, HashSet::new(), Vec::new()));

        let project_directory_changed = opened_project_directory_path != loaded_project_directory_path;
        let project_items_changed = opened_project_item_paths != loaded_project_item_paths;
        let sort_order_changed = opened_project_sort_order != loaded_project_sort_order;

        if project_directory_changed || project_items_changed || sort_order_changed {
            ProjectHierarchyViewData::refresh_project_items(self.project_hierarchy_view_data.clone(), self.app_context.clone());
        }
    }

    fn extract_string_value_from_edited_field(edited_field: &ValuedStructField) -> Option<String> {
        let data_value = edited_field.get_data_value()?;
        let edited_name = String::from_utf8(data_value.get_value_bytes().clone()).ok()?;
        let edited_name = edited_name.trim();

        if edited_name.is_empty() { None } else { Some(edited_name.to_string()) }
    }

    fn build_project_item_rename_request(
        project_item_path: &Path,
        project_item_type_id: &str,
        edited_name: &str,
    ) -> Option<ProjectItemsRenameRequest> {
        let sanitized_file_name = Path::new(edited_name)
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .map(str::trim)
            .filter(|file_name| !file_name.is_empty())?
            .to_string();
        let is_directory_project_item = project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID;
        let renamed_project_item_name = if is_directory_project_item {
            sanitized_file_name
        } else {
            let mut file_name_with_extension = sanitized_file_name.clone();
            let expected_extension = Project::PROJECT_ITEM_EXTENSION.trim_start_matches('.');
            let has_expected_extension = Path::new(&sanitized_file_name)
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| extension.eq_ignore_ascii_case(expected_extension))
                .unwrap_or(false);

            if !has_expected_extension {
                file_name_with_extension.push('.');
                file_name_with_extension.push_str(expected_extension);
            }

            file_name_with_extension
        };
        let current_file_name = project_item_path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or_default();

        if current_file_name == renamed_project_item_name {
            return None;
        }

        Some(ProjectItemsRenameRequest {
            project_item_path: project_item_path.to_path_buf(),
            project_item_name: renamed_project_item_name,
        })
    }

    fn should_apply_struct_field_edit_to_project_item(
        project_item_type_id: &str,
        edited_field_name: &str,
    ) -> bool {
        !(edited_field_name == ProjectItem::PROPERTY_NAME && project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID)
    }
}
