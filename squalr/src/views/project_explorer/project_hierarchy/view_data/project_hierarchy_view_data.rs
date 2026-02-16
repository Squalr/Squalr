use crate::app_context::AppContext;
use crate::views::project_explorer::project_hierarchy::view_data::{
    project_hierarchy_pending_operation::ProjectHierarchyPendingOperation, project_hierarchy_take_over_state::ProjectHierarchyTakeOverState,
    project_hierarchy_tree_entry::ProjectHierarchyTreeEntry,
};
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest;
use squalr_engine_api::commands::project_items::delete::project_items_delete_request::ProjectItemsDeleteRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::move_item::project_items_move_request::ProjectItemsMoveRequest;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_request::ProjectItemsReorderRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

enum ProjectHierarchyDropOperation {
    Reorder {
        project_item_paths: Vec<PathBuf>,
    },
    Move {
        project_item_paths: Vec<PathBuf>,
        target_directory_path: PathBuf,
    },
}

#[derive(Clone)]
pub struct ProjectHierarchyViewData {
    pub opened_project_info: Option<ProjectInfo>,
    pub opened_project_root: Option<ProjectItem>,
    pub project_items: Vec<(ProjectItemRef, ProjectItem)>,
    pub tree_entries: Vec<ProjectHierarchyTreeEntry>,
    pub selected_project_item_path: Option<PathBuf>,
    pub selected_project_item_paths: HashSet<PathBuf>,
    pub selection_anchor_project_item_path: Option<PathBuf>,
    pub expanded_directory_paths: HashSet<PathBuf>,
    pub context_menu_project_item_path: Option<PathBuf>,
    pub dragged_project_item_paths: Option<Vec<PathBuf>>,
    pub take_over_state: ProjectHierarchyTakeOverState,
    pub pending_operation: ProjectHierarchyPendingOperation,
}

impl ProjectHierarchyViewData {
    pub fn new() -> Self {
        Self {
            opened_project_info: None,
            opened_project_root: None,
            project_items: Vec::new(),
            tree_entries: Vec::new(),
            selected_project_item_path: None,
            selected_project_item_paths: HashSet::new(),
            selection_anchor_project_item_path: None,
            expanded_directory_paths: HashSet::new(),
            context_menu_project_item_path: None,
            dragged_project_item_paths: None,
            take_over_state: ProjectHierarchyTakeOverState::None,
            pending_operation: ProjectHierarchyPendingOperation::None,
        }
    }

    pub fn refresh_project_items(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
    ) {
        if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy view data refresh request") {
            if project_hierarchy_view_data.pending_operation == ProjectHierarchyPendingOperation::Refreshing {
                return;
            }

            project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Refreshing;
        }

        let project_items_list_request = ProjectItemsListRequest {};

        project_items_list_request.send(&app_context.engine_unprivileged_state, move |project_items_list_response| {
            let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy view data refresh response") {
                Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                None => return,
            };

            project_hierarchy_view_data.opened_project_info = project_items_list_response.opened_project_info;
            project_hierarchy_view_data.opened_project_root = project_items_list_response.opened_project_root;
            project_hierarchy_view_data.project_items = project_items_list_response.opened_project_items;

            if let Some(project_root_directory_path) = Self::resolve_project_root_path(
                project_hierarchy_view_data.opened_project_info.as_ref(),
                &project_hierarchy_view_data.project_items,
            ) {
                project_hierarchy_view_data
                    .expanded_directory_paths
                    .insert(project_root_directory_path);
            }

            project_hierarchy_view_data.tree_entries = Self::build_tree_entries(
                project_hierarchy_view_data.opened_project_info.as_ref(),
                &project_hierarchy_view_data.project_items,
                &project_hierarchy_view_data.expanded_directory_paths,
            );

            project_hierarchy_view_data.retain_valid_selection();

            if let Some(dragged_project_item_paths) = &project_hierarchy_view_data.dragged_project_item_paths {
                let visible_project_item_paths: HashSet<PathBuf> = project_hierarchy_view_data
                    .tree_entries
                    .iter()
                    .map(|tree_entry| tree_entry.project_item_path.clone())
                    .collect();
                let has_invalid_dragged_project_item = dragged_project_item_paths
                    .iter()
                    .any(|dragged_project_item_path| !visible_project_item_paths.contains(dragged_project_item_path));

                if has_invalid_dragged_project_item {
                    project_hierarchy_view_data.dragged_project_item_paths = None;
                }
            }

            project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
        });
    }

    pub fn select_project_item(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_path: PathBuf,
        additive_selection: bool,
        range_selection: bool,
    ) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy select project item") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };
        project_hierarchy_view_data.apply_selection(project_item_path, additive_selection, range_selection);
    }

    fn apply_selection(
        &mut self,
        project_item_path: PathBuf,
        additive_selection: bool,
        range_selection: bool,
    ) {
        let selected_project_item_paths_in_order = self.collect_selected_project_item_paths_in_tree_order();

        if range_selection {
            let selection_anchor_project_item_path = self
                .selection_anchor_project_item_path
                .clone()
                .or_else(|| self.selected_project_item_path.clone())
                .unwrap_or_else(|| project_item_path.clone());
            let selected_project_item_paths_in_range = self.collect_project_item_paths_in_range(&selection_anchor_project_item_path, &project_item_path);

            if additive_selection {
                for selected_project_item_path in selected_project_item_paths_in_range {
                    self.selected_project_item_paths
                        .insert(selected_project_item_path);
                }
            } else {
                self.selected_project_item_paths = selected_project_item_paths_in_range.into_iter().collect();
            }

            self.selected_project_item_path = Some(project_item_path.clone());
            self.selection_anchor_project_item_path = Some(selection_anchor_project_item_path);
            self.retain_valid_selection();
            return;
        }

        if additive_selection {
            if self.selected_project_item_paths.contains(&project_item_path) {
                self.selected_project_item_paths.remove(&project_item_path);

                if self.selected_project_item_path.as_ref() == Some(&project_item_path) {
                    self.selected_project_item_path = selected_project_item_paths_in_order
                        .into_iter()
                        .find(|selected_project_item_path| {
                            self.selected_project_item_paths
                                .contains(selected_project_item_path)
                        });
                }
            } else {
                self.selected_project_item_paths
                    .insert(project_item_path.clone());
                self.selected_project_item_path = Some(project_item_path.clone());
            }

            self.selection_anchor_project_item_path = Some(project_item_path);
            self.retain_valid_selection();
            return;
        }

        self.selected_project_item_paths.clear();
        self.selected_project_item_paths
            .insert(project_item_path.clone());
        self.selected_project_item_path = Some(project_item_path.clone());
        self.selection_anchor_project_item_path = Some(project_item_path);
        self.retain_valid_selection();
    }

    pub fn get_selected_directory_path(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) -> Option<PathBuf> {
        let project_hierarchy_view_data = project_hierarchy_view_data.read("Project hierarchy selected directory path")?;
        let selected_project_item_path = project_hierarchy_view_data
            .selected_project_item_path
            .as_ref()?;
        let selected_project_item = project_hierarchy_view_data
            .project_items
            .iter()
            .find(|(project_item_ref, _)| project_item_ref.get_project_item_path() == selected_project_item_path)
            .map(|(_, project_item)| project_item);

        if selected_project_item
            .map(Self::is_directory_project_item)
            .unwrap_or(false)
        {
            Some(selected_project_item_path.clone())
        } else {
            selected_project_item_path.parent().map(Path::to_path_buf)
        }
    }

    pub fn toggle_directory_expansion(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_path: PathBuf,
    ) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy toggle directory expansion") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        if project_hierarchy_view_data
            .expanded_directory_paths
            .contains(&project_item_path)
        {
            project_hierarchy_view_data
                .expanded_directory_paths
                .remove(&project_item_path);
        } else {
            project_hierarchy_view_data
                .expanded_directory_paths
                .insert(project_item_path);
        }

        project_hierarchy_view_data.tree_entries = Self::build_tree_entries(
            project_hierarchy_view_data.opened_project_info.as_ref(),
            &project_hierarchy_view_data.project_items,
            &project_hierarchy_view_data.expanded_directory_paths,
        );
    }

    pub fn request_delete_confirmation_for_selected_project_item(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) {
        let selected_project_item_paths = project_hierarchy_view_data
            .read("Project hierarchy selected project item for delete request")
            .map(|project_hierarchy_view_data| project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order())
            .unwrap_or_default();

        if !selected_project_item_paths.is_empty() {
            Self::request_delete_confirmation(project_hierarchy_view_data, selected_project_item_paths);
        }
    }

    pub fn request_delete_confirmation(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_paths: Vec<PathBuf>,
    ) {
        if project_item_paths.is_empty() {
            return;
        }

        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy request delete confirmation") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::DeleteConfirmation { project_item_paths };
    }

    pub fn cancel_take_over(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy cancel take over") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::None;
    }

    pub fn begin_reorder_drag(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_path: PathBuf,
    ) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy begin reorder drag") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        if project_hierarchy_view_data.pending_operation != ProjectHierarchyPendingOperation::None {
            return;
        }

        let dragged_project_item_paths = project_hierarchy_view_data.collect_dragged_project_item_paths(&project_item_path);
        project_hierarchy_view_data.dragged_project_item_paths = Some(dragged_project_item_paths);
    }

    pub fn cancel_reorder_drag(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy cancel reorder drag") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        project_hierarchy_view_data.dragged_project_item_paths = None;
    }

    pub fn commit_reorder_drop(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
        target_project_item_path: PathBuf,
    ) {
        let drop_operation = {
            let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy commit reorder drop") {
                Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                None => return,
            };
            let dragged_project_item_paths = match project_hierarchy_view_data.dragged_project_item_paths.clone() {
                Some(dragged_project_item_paths) if !dragged_project_item_paths.is_empty() => dragged_project_item_paths,
                _ => return,
            };

            if project_hierarchy_view_data.pending_operation != ProjectHierarchyPendingOperation::None {
                project_hierarchy_view_data.dragged_project_item_paths = None;
                return;
            }

            let drop_operation = Self::build_drop_operation(
                project_hierarchy_view_data.opened_project_info.as_ref(),
                &project_hierarchy_view_data.project_items,
                &dragged_project_item_paths,
                &target_project_item_path,
            );

            match drop_operation {
                Some(drop_operation) => {
                    project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Reordering;
                    project_hierarchy_view_data.dragged_project_item_paths = None;
                    drop_operation
                }
                None => {
                    project_hierarchy_view_data.dragged_project_item_paths = None;
                    return;
                }
            }
        };

        let app_context_clone = app_context.clone();
        let project_hierarchy_view_data_clone = project_hierarchy_view_data.clone();

        match drop_operation {
            ProjectHierarchyDropOperation::Reorder { project_item_paths } => {
                let project_items_reorder_request = ProjectItemsReorderRequest { project_item_paths };

                project_items_reorder_request.send(&app_context.engine_unprivileged_state, move |project_items_reorder_response| {
                    if !project_items_reorder_response.success {
                        log::error!(
                            "Failed to reorder project items. Reordered count: {}.",
                            project_items_reorder_response.reordered_project_item_count
                        );
                    }

                    if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data_clone.write("Project hierarchy reorder project items response") {
                        project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                    }

                    Self::refresh_project_items(project_hierarchy_view_data_clone, app_context_clone);
                });
            }
            ProjectHierarchyDropOperation::Move {
                project_item_paths,
                target_directory_path,
            } => {
                let project_items_move_request = ProjectItemsMoveRequest {
                    project_item_paths,
                    target_directory_path,
                };

                project_items_move_request.send(&app_context.engine_unprivileged_state, move |project_items_move_response| {
                    if !project_items_move_response.success {
                        log::error!(
                            "Failed to move project items. Moved count: {}.",
                            project_items_move_response.moved_project_item_count
                        );
                    }

                    if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data_clone.write("Project hierarchy move project items response") {
                        project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                    }

                    Self::refresh_project_items(project_hierarchy_view_data_clone, app_context_clone);
                });
            }
        }
    }

    pub fn delete_project_items(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
        project_item_paths: Vec<PathBuf>,
    ) {
        if project_item_paths.is_empty() {
            Self::cancel_take_over(project_hierarchy_view_data);

            return;
        }

        if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy begin delete project items") {
            project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Deleting;
            project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::None;
        }

        let project_items_delete_request = ProjectItemsDeleteRequest { project_item_paths };
        let app_context_clone = app_context.clone();
        let project_hierarchy_view_data_clone = project_hierarchy_view_data.clone();

        project_items_delete_request.send(&app_context.engine_unprivileged_state, move |project_items_delete_response| {
            if !project_items_delete_response.success {
                log::error!(
                    "Failed to delete one or more project items. Deleted count: {}.",
                    project_items_delete_response.deleted_project_item_count
                );
            }

            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data_clone.write("Project hierarchy delete project items response") {
                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
            }

            Self::refresh_project_items(project_hierarchy_view_data_clone, app_context_clone);
        });
    }

    pub fn create_directory(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
        target_project_item_path: PathBuf,
    ) {
        let (parent_directory_path, directory_name) = match project_hierarchy_view_data.write("Project hierarchy resolve create directory target") {
            Some(project_hierarchy_view_data) => {
                let parent_directory_path = Self::resolve_directory_create_parent_path(&project_hierarchy_view_data.project_items, &target_project_item_path);
                let directory_name = Self::build_unique_directory_name(&project_hierarchy_view_data.project_items, &parent_directory_path);

                (parent_directory_path, directory_name)
            }
            None => return,
        };

        let project_items_create_request = ProjectItemsCreateRequest {
            parent_directory_path,
            project_item_name: directory_name,
            project_item_type: ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID.to_string(),
        };
        let app_context_clone = app_context.clone();
        let project_hierarchy_view_data_clone = project_hierarchy_view_data.clone();

        project_items_create_request.send(&app_context.engine_unprivileged_state, move |project_items_create_response| {
            if !project_items_create_response.success {
                log::error!("Failed to create project directory item.");
                return;
            }

            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data_clone.write("Project hierarchy select created directory") {
                Self::expand_project_item_ancestor_directories(
                    &mut project_hierarchy_view_data.expanded_directory_paths,
                    &project_items_create_response.created_project_item_path,
                );
                project_hierarchy_view_data.selected_project_item_path = Some(project_items_create_response.created_project_item_path.clone());
                project_hierarchy_view_data.selected_project_item_paths.clear();
                project_hierarchy_view_data
                    .selected_project_item_paths
                    .insert(project_items_create_response.created_project_item_path.clone());
                project_hierarchy_view_data.selection_anchor_project_item_path = Some(project_items_create_response.created_project_item_path.clone());
            }

            Self::refresh_project_items(project_hierarchy_view_data_clone, app_context_clone);
        });
    }

    pub fn set_project_item_activation(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
        project_item_paths: Vec<PathBuf>,
        is_activated: bool,
    ) {
        if project_item_paths.is_empty() {
            return;
        }

        let project_items_activate_request = ProjectItemsActivateRequest {
            project_item_paths: project_item_paths
                .into_iter()
                .map(|project_item_path| project_item_path.to_string_lossy().into_owned())
                .collect(),
            is_activated,
        };
        let app_context_clone = app_context.clone();
        let project_hierarchy_view_data_clone = project_hierarchy_view_data.clone();

        project_items_activate_request.send(&app_context.engine_unprivileged_state, move |_project_items_activate_response| {
            Self::refresh_project_items(project_hierarchy_view_data_clone, app_context_clone);
        });
    }

    fn build_tree_entries(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
        expanded_directory_paths: &HashSet<PathBuf>,
    ) -> Vec<ProjectHierarchyTreeEntry> {
        let (project_root_directory_path, project_item_map, child_paths_by_parent_path) =
            match Self::build_project_hierarchy_maps(opened_project_info, project_items) {
                Some(project_hierarchy_maps) => project_hierarchy_maps,
                None => return Vec::new(),
            };

        let mut visible_tree_entries = Vec::new();
        let root_is_expanded = expanded_directory_paths.contains(&project_root_directory_path);

        if let Some((project_item_ref, project_item)) = project_item_map.get(&project_root_directory_path) {
            let has_children = child_paths_by_parent_path
                .get(&project_root_directory_path)
                .map(|entries| !entries.is_empty())
                .unwrap_or(false);
            let display_name = opened_project_info
                .map(|project_info| project_info.get_name().to_string())
                .filter(|project_name| !project_name.is_empty())
                .unwrap_or_else(|| {
                    let root_display_name = project_item.get_field_name();
                    if root_display_name.is_empty() {
                        project_root_directory_path
                            .file_name()
                            .and_then(|value| value.to_str())
                            .unwrap_or_default()
                            .to_string()
                    } else {
                        root_display_name
                    }
                });

            visible_tree_entries.push(ProjectHierarchyTreeEntry {
                project_item_ref: project_item_ref.clone(),
                project_item: project_item.clone(),
                project_item_path: project_root_directory_path.clone(),
                display_name,
                preview_value: Self::build_preview_value(project_item),
                is_activated: project_item.get_is_activated(),
                depth: 0,
                is_directory: true,
                has_children,
                is_expanded: root_is_expanded,
            });
        }

        if root_is_expanded {
            Self::append_visible_entries(
                &mut visible_tree_entries,
                &project_root_directory_path,
                &child_paths_by_parent_path,
                &project_item_map,
                1,
                expanded_directory_paths,
            );
        }

        visible_tree_entries
    }

    fn build_drop_operation(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
        dragged_project_item_paths: &[PathBuf],
        target_project_item_path: &Path,
    ) -> Option<ProjectHierarchyDropOperation> {
        if dragged_project_item_paths.is_empty() {
            return None;
        }

        let (_project_root_directory_path, project_item_map, mut child_paths_by_parent_path) =
            Self::build_project_hierarchy_maps(opened_project_info, project_items)?;
        let target_is_directory = Self::is_directory_path(target_project_item_path, &project_item_map);
        let target_directory_path = if target_is_directory {
            target_project_item_path.to_path_buf()
        } else {
            target_project_item_path.parent()?.to_path_buf()
        };
        let dragged_project_item_path_set = dragged_project_item_paths
            .iter()
            .cloned()
            .collect::<HashSet<PathBuf>>();

        if dragged_project_item_path_set.contains(target_project_item_path) {
            return None;
        }

        if dragged_project_item_paths
            .iter()
            .any(|dragged_project_item_path| target_directory_path.starts_with(dragged_project_item_path))
        {
            return None;
        }

        let all_dragged_items_share_target_parent = dragged_project_item_paths
            .iter()
            .all(|dragged_project_item_path| dragged_project_item_path.parent() == Some(target_directory_path.as_path()));

        if !all_dragged_items_share_target_parent {
            return Some(ProjectHierarchyDropOperation::Move {
                project_item_paths: dragged_project_item_paths.to_vec(),
                target_directory_path,
            });
        }

        let sibling_paths = child_paths_by_parent_path.get_mut(&target_directory_path)?;
        let dragged_paths_in_sibling_order: Vec<PathBuf> = sibling_paths
            .iter()
            .filter(|sibling_project_item_path| dragged_project_item_path_set.contains(*sibling_project_item_path))
            .cloned()
            .collect();

        if dragged_paths_in_sibling_order.len() != dragged_project_item_path_set.len() {
            return None;
        }

        sibling_paths.retain(|sibling_project_item_path| !dragged_project_item_path_set.contains(sibling_project_item_path));
        let target_sibling_index = sibling_paths
            .iter()
            .position(|project_item_path| project_item_path == target_project_item_path)?;

        for (dragged_path_insert_index, dragged_project_item_path) in dragged_paths_in_sibling_order.into_iter().enumerate() {
            sibling_paths.insert(target_sibling_index + dragged_path_insert_index, dragged_project_item_path);
        }

        let mut reordered_project_item_paths = Vec::new();
        Self::append_project_item_paths_in_order(&target_directory_path, &child_paths_by_parent_path, &mut reordered_project_item_paths);

        Some(ProjectHierarchyDropOperation::Reorder {
            project_item_paths: reordered_project_item_paths,
        })
    }

    fn append_visible_entries(
        visible_tree_entries: &mut Vec<ProjectHierarchyTreeEntry>,
        parent_path: &PathBuf,
        child_paths_by_parent_path: &HashMap<PathBuf, Vec<PathBuf>>,
        project_item_map: &HashMap<PathBuf, (ProjectItemRef, ProjectItem)>,
        depth: usize,
        expanded_directory_paths: &HashSet<PathBuf>,
    ) {
        let child_paths = match child_paths_by_parent_path.get(parent_path) {
            Some(child_paths) => child_paths,
            None => return,
        };

        for child_path in child_paths {
            let (project_item_ref, project_item) = match project_item_map.get(child_path) {
                Some(project_item_pair) => project_item_pair,
                None => continue,
            };
            let is_directory = Self::is_directory_project_item(project_item);
            let has_children = child_paths_by_parent_path
                .get(child_path)
                .map(|entries| !entries.is_empty())
                .unwrap_or(false);
            let is_expanded = expanded_directory_paths.contains(child_path);
            let display_name = child_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            let display_name_from_property = project_item.get_field_name();
            let display_name = if display_name_from_property.is_empty() {
                display_name.to_string()
            } else {
                display_name_from_property
            };
            let preview_value = Self::build_preview_value(project_item);

            visible_tree_entries.push(ProjectHierarchyTreeEntry {
                project_item_ref: project_item_ref.clone(),
                project_item: project_item.clone(),
                project_item_path: child_path.clone(),
                display_name,
                preview_value,
                is_activated: project_item.get_is_activated(),
                depth,
                is_directory,
                has_children,
                is_expanded,
            });

            if is_directory && is_expanded {
                Self::append_visible_entries(
                    visible_tree_entries,
                    child_path,
                    child_paths_by_parent_path,
                    project_item_map,
                    depth + 1,
                    expanded_directory_paths,
                );
            }
        }
    }

    fn build_sort_order_lookup(
        project_info: &ProjectInfo,
        project_directory_path: &Path,
    ) -> HashMap<PathBuf, usize> {
        project_info
            .get_project_manifest()
            .get_project_item_sort_order()
            .iter()
            .enumerate()
            .map(|(sort_order_index, relative_project_item_path)| (project_directory_path.join(relative_project_item_path), sort_order_index))
            .collect()
    }

    fn build_project_hierarchy_maps(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
    ) -> Option<(PathBuf, HashMap<PathBuf, (ProjectItemRef, ProjectItem)>, HashMap<PathBuf, Vec<PathBuf>>)> {
        let project_root_directory_path = Self::resolve_project_root_path(opened_project_info, project_items)?;
        let project_info = opened_project_info?;
        let project_directory_path = project_info.get_project_directory()?;
        let project_item_map: HashMap<PathBuf, (ProjectItemRef, ProjectItem)> = project_items
            .iter()
            .map(|(project_item_ref, project_item)| {
                (
                    project_item_ref.get_project_item_path().clone(),
                    (project_item_ref.clone(), project_item.clone()),
                )
            })
            .collect();
        let sort_order_lookup = Self::build_sort_order_lookup(project_info, &project_directory_path);
        let mut child_paths_by_parent_path: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

        for project_item_path in project_item_map.keys() {
            if project_item_path == &project_root_directory_path {
                continue;
            }

            let parent_path = project_item_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| project_root_directory_path.clone());

            child_paths_by_parent_path
                .entry(parent_path)
                .or_default()
                .push(project_item_path.clone());
        }

        for child_paths in child_paths_by_parent_path.values_mut() {
            child_paths.sort_by(|left_path, right_path| {
                let left_order = sort_order_lookup.get(left_path).copied().unwrap_or(usize::MAX);
                let right_order = sort_order_lookup.get(right_path).copied().unwrap_or(usize::MAX);

                if left_order != right_order {
                    return left_order.cmp(&right_order);
                }

                let left_is_directory = Self::is_directory_path(left_path, &project_item_map);
                let right_is_directory = Self::is_directory_path(right_path, &project_item_map);

                if left_is_directory != right_is_directory {
                    return right_is_directory.cmp(&left_is_directory);
                }

                let left_name = left_path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();
                let right_name = right_path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();

                left_name.cmp(right_name)
            });
        }

        Some((project_root_directory_path, project_item_map, child_paths_by_parent_path))
    }

    fn resolve_project_root_path(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
    ) -> Option<PathBuf> {
        let project_info = opened_project_info?;
        let project_directory_path = project_info.get_project_directory()?;
        let hidden_project_root_path = project_directory_path.join(Project::PROJECT_DIR);
        let contains_hidden_project_root = project_items
            .iter()
            .any(|(project_item_ref, _)| project_item_ref.get_project_item_path() == &hidden_project_root_path);

        if contains_hidden_project_root {
            Some(hidden_project_root_path)
        } else {
            Some(project_directory_path)
        }
    }

    fn expand_project_item_ancestor_directories(
        expanded_directory_paths: &mut HashSet<PathBuf>,
        project_item_path: &Path,
    ) {
        let mut current_path = project_item_path.parent();

        while let Some(directory_path) = current_path {
            expanded_directory_paths.insert(directory_path.to_path_buf());
            current_path = directory_path.parent();
        }
    }

    fn append_project_item_paths_in_order(
        parent_path: &Path,
        child_paths_by_parent_path: &HashMap<PathBuf, Vec<PathBuf>>,
        reordered_project_item_paths: &mut Vec<PathBuf>,
    ) {
        let child_paths = match child_paths_by_parent_path.get(parent_path) {
            Some(child_paths) => child_paths,
            None => return,
        };

        for child_path in child_paths {
            reordered_project_item_paths.push(child_path.clone());
            Self::append_project_item_paths_in_order(child_path, child_paths_by_parent_path, reordered_project_item_paths);
        }
    }

    fn is_directory_path(
        project_item_path: &Path,
        project_item_map: &HashMap<PathBuf, (ProjectItemRef, ProjectItem)>,
    ) -> bool {
        project_item_map
            .get(project_item_path)
            .map(|(_, project_item)| Self::is_directory_project_item(project_item))
            .unwrap_or(false)
    }

    fn is_directory_project_item(project_item: &ProjectItem) -> bool {
        project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID
    }

    fn build_preview_value(project_item: &ProjectItem) -> String {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let preview_value = Self::read_string_field(project_item, ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE);

            if preview_value.is_empty() { "??".to_string() } else { preview_value }
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            let preview_value = Self::read_string_field(project_item, ProjectItemTypePointer::PROPERTY_FREEZE_DISPLAY_VALUE);

            if preview_value.is_empty() { "??".to_string() } else { preview_value }
        } else {
            String::new()
        }
    }

    fn read_string_field(
        project_item: &ProjectItem,
        field_name: &str,
    ) -> String {
        let data_value = match project_item
            .get_properties()
            .get_field(field_name)
            .and_then(|field| field.get_data_value())
        {
            Some(data_value) => data_value,
            None => return String::new(),
        };

        String::from_utf8(data_value.get_value_bytes().clone()).unwrap_or_default()
    }

    fn resolve_directory_create_parent_path(
        project_items: &[(ProjectItemRef, ProjectItem)],
        target_project_item_path: &Path,
    ) -> PathBuf {
        let is_target_directory = project_items
            .iter()
            .find(|(project_item_ref, _)| project_item_ref.get_project_item_path() == target_project_item_path)
            .map(|(_, project_item)| Self::is_directory_project_item(project_item))
            .unwrap_or(false);

        if is_target_directory {
            target_project_item_path.to_path_buf()
        } else {
            target_project_item_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| target_project_item_path.to_path_buf())
        }
    }

    fn build_unique_directory_name(
        project_items: &[(ProjectItemRef, ProjectItem)],
        parent_directory_path: &Path,
    ) -> String {
        const BASE_DIRECTORY_NAME: &str = "New Folder";
        let existing_children: HashSet<String> = project_items
            .iter()
            .map(|(project_item_ref, _)| project_item_ref.get_project_item_path())
            .filter(|project_item_path| project_item_path.parent() == Some(parent_directory_path))
            .filter_map(|project_item_path| {
                project_item_path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .map(str::to_string)
            })
            .collect();

        if !existing_children.contains(BASE_DIRECTORY_NAME) {
            return BASE_DIRECTORY_NAME.to_string();
        }

        let mut directory_suffix_index = 2usize;
        loop {
            let candidate_name = format!("{} {}", BASE_DIRECTORY_NAME, directory_suffix_index);
            if !existing_children.contains(&candidate_name) {
                return candidate_name;
            }

            directory_suffix_index += 1;
        }
    }

    fn retain_valid_selection(&mut self) {
        let valid_project_item_paths: HashSet<PathBuf> = self
            .tree_entries
            .iter()
            .map(|tree_entry| tree_entry.project_item_path.clone())
            .collect();
        self.selected_project_item_paths
            .retain(|selected_project_item_path| valid_project_item_paths.contains(selected_project_item_path));

        if let Some(selected_project_item_path) = &self.selected_project_item_path {
            if !self
                .selected_project_item_paths
                .contains(selected_project_item_path)
            {
                self.selected_project_item_path = None;
            }
        }

        if self.selected_project_item_path.is_none() {
            self.selected_project_item_path = self
                .collect_selected_project_item_paths_in_tree_order()
                .into_iter()
                .next();
        }

        if let Some(selection_anchor_project_item_path) = &self.selection_anchor_project_item_path {
            if !valid_project_item_paths.contains(selection_anchor_project_item_path) {
                self.selection_anchor_project_item_path = None;
            }
        }

        if self.selected_project_item_paths.is_empty() {
            self.selected_project_item_path = None;
            self.selection_anchor_project_item_path = None;
        }
    }

    pub fn collect_selected_project_item_paths_in_tree_order(&self) -> Vec<PathBuf> {
        self.tree_entries
            .iter()
            .map(|tree_entry| tree_entry.project_item_path.clone())
            .filter(|project_item_path| self.selected_project_item_paths.contains(project_item_path))
            .collect()
    }

    fn collect_dragged_project_item_paths(
        &self,
        drag_start_project_item_path: &Path,
    ) -> Vec<PathBuf> {
        let mut dragged_project_item_paths = if self
            .selected_project_item_paths
            .contains(drag_start_project_item_path)
        {
            self.collect_selected_project_item_paths_in_tree_order()
        } else {
            Vec::new()
        };

        if dragged_project_item_paths.is_empty() {
            dragged_project_item_paths.push(drag_start_project_item_path.to_path_buf());
        }

        dragged_project_item_paths
    }

    fn collect_project_item_paths_in_range(
        &self,
        selection_start_project_item_path: &Path,
        selection_end_project_item_path: &Path,
    ) -> Vec<PathBuf> {
        let selection_start_index = self
            .tree_entries
            .iter()
            .position(|tree_entry| tree_entry.project_item_path == selection_start_project_item_path);
        let selection_end_index = self
            .tree_entries
            .iter()
            .position(|tree_entry| tree_entry.project_item_path == selection_end_project_item_path);

        let (selection_start_index, selection_end_index) = match (selection_start_index, selection_end_index) {
            (Some(selection_start_index), Some(selection_end_index)) => (selection_start_index, selection_end_index),
            _ => return vec![selection_end_project_item_path.to_path_buf()],
        };
        let (range_start_index, range_end_index) = if selection_start_index <= selection_end_index {
            (selection_start_index, selection_end_index)
        } else {
            (selection_end_index, selection_start_index)
        };

        self.tree_entries[range_start_index..=range_end_index]
            .iter()
            .map(|tree_entry| tree_entry.project_item_path.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectHierarchyViewData;
    use crate::views::project_explorer::project_hierarchy::view_data::project_hierarchy_tree_entry::ProjectHierarchyTreeEntry;
    use squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
    use squalr_engine_api::structures::projects::project_items::built_in_types::{
        project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
        project_item_type_pointer::ProjectItemTypePointer,
    };
    use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
    use std::path::{Path, PathBuf};

    fn create_directory_project_item(project_item_path: &Path) -> (ProjectItemRef, ProjectItem) {
        let project_item_ref = ProjectItemRef::new(project_item_path.to_path_buf());
        let project_item = ProjectItemTypeDirectory::new_project_item(&project_item_ref);

        (project_item_ref, project_item)
    }

    fn create_directory_tree_entry(
        project_item_path: &Path,
        depth: usize,
    ) -> ProjectHierarchyTreeEntry {
        let (project_item_ref, project_item) = create_directory_project_item(project_item_path);

        ProjectHierarchyTreeEntry {
            project_item_ref,
            project_item,
            project_item_path: project_item_path.to_path_buf(),
            display_name: project_item_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string(),
            preview_value: String::new(),
            is_activated: false,
            depth,
            is_directory: true,
            has_children: false,
            is_expanded: false,
        }
    }

    #[test]
    fn resolve_directory_create_parent_path_for_directory_target_returns_target_path() {
        let project_directory_path = PathBuf::from("C:/Projects/TestProject/project");
        let target_directory_path = project_directory_path.join("Cheats");
        let project_items = vec![create_directory_project_item(&target_directory_path)];

        let resolved_parent_path = ProjectHierarchyViewData::resolve_directory_create_parent_path(&project_items, &target_directory_path);

        assert_eq!(resolved_parent_path, target_directory_path);
    }

    #[test]
    fn resolve_directory_create_parent_path_for_file_target_returns_parent_directory() {
        let project_directory_path = PathBuf::from("C:/Projects/TestProject/project");
        let target_directory_path = project_directory_path.join("Cheats");
        let target_file_path = target_directory_path.join("health.json");
        let project_items = vec![
            create_directory_project_item(&target_directory_path),
            (
                ProjectItemRef::new(target_file_path.clone()),
                ProjectItemTypeAddress::new_project_item(
                    "Health",
                    0x1234,
                    "game.exe",
                    "",
                    squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8::get_value_from_primitive(0),
                ),
            ),
        ];

        let resolved_parent_path = ProjectHierarchyViewData::resolve_directory_create_parent_path(&project_items, &target_file_path);

        assert_eq!(resolved_parent_path, target_directory_path);
    }

    #[test]
    fn build_unique_directory_name_returns_incremented_suffix_when_name_conflicts() {
        let parent_directory_path = PathBuf::from("C:/Projects/TestProject/project");
        let project_items = vec![
            create_directory_project_item(&parent_directory_path.join("New Folder")),
            create_directory_project_item(&parent_directory_path.join("New Folder 2")),
        ];

        let next_directory_name = ProjectHierarchyViewData::build_unique_directory_name(&project_items, &parent_directory_path);

        assert_eq!(next_directory_name, "New Folder 3");
    }

    #[test]
    fn apply_selection_with_additive_selection_toggles_entries() {
        let root_path = PathBuf::from("C:/Projects/TestProject/project");
        let first_child_path = root_path.join("First");
        let second_child_path = root_path.join("Second");
        let mut project_hierarchy_view_data = ProjectHierarchyViewData::new();
        project_hierarchy_view_data.tree_entries = vec![
            create_directory_tree_entry(&root_path, 0),
            create_directory_tree_entry(&first_child_path, 1),
            create_directory_tree_entry(&second_child_path, 1),
        ];

        project_hierarchy_view_data.apply_selection(first_child_path.clone(), false, false);
        project_hierarchy_view_data.apply_selection(second_child_path.clone(), true, false);
        let selected_project_item_paths = project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order();
        assert_eq!(selected_project_item_paths, vec![first_child_path.clone(), second_child_path.clone()]);

        project_hierarchy_view_data.apply_selection(first_child_path.clone(), true, false);
        let selected_project_item_paths = project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order();
        assert_eq!(selected_project_item_paths, vec![second_child_path.clone()]);
    }

    #[test]
    fn apply_selection_with_range_selection_selects_contiguous_entries() {
        let root_path = PathBuf::from("C:/Projects/TestProject/project");
        let child_one_path = root_path.join("One");
        let child_two_path = root_path.join("Two");
        let child_three_path = root_path.join("Three");
        let mut project_hierarchy_view_data = ProjectHierarchyViewData::new();
        project_hierarchy_view_data.tree_entries = vec![
            create_directory_tree_entry(&root_path, 0),
            create_directory_tree_entry(&child_one_path, 1),
            create_directory_tree_entry(&child_two_path, 1),
            create_directory_tree_entry(&child_three_path, 1),
        ];

        project_hierarchy_view_data.apply_selection(child_one_path.clone(), false, false);
        project_hierarchy_view_data.apply_selection(child_three_path.clone(), false, true);
        let selected_project_item_paths = project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order();

        assert_eq!(
            selected_project_item_paths,
            vec![
                child_one_path.clone(),
                child_two_path.clone(),
                child_three_path.clone()
            ]
        );
    }

    #[test]
    fn build_preview_value_for_pointer_without_display_value_returns_unknown() {
        let pointer_project_item = ProjectItemTypePointer::new_project_item("Pointer", "", "");

        let preview_value = ProjectHierarchyViewData::build_preview_value(&pointer_project_item);

        assert_eq!(preview_value, "??");
    }

    #[test]
    fn build_preview_value_for_pointer_with_display_value_returns_display_value() {
        let pointer_project_item = ProjectItemTypePointer::new_project_item("Pointer", "", "0x1234 -> 0x5678");

        let preview_value = ProjectHierarchyViewData::build_preview_value(&pointer_project_item);

        assert_eq!(preview_value, "0x1234 -> 0x5678");
    }

    #[test]
    fn build_preview_value_for_address_without_display_value_returns_unknown() {
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU8::get_value_from_primitive(0));

        let preview_value = ProjectHierarchyViewData::build_preview_value(&address_project_item);

        assert_eq!(preview_value, "??");
    }

    #[test]
    fn expand_project_item_ancestor_directories_expands_full_parent_chain() {
        let project_root_path = PathBuf::from("C:/Projects/TestProject/project_items");
        let nested_directory_path = project_root_path
            .join("Player")
            .join("Stats")
            .join("New Folder");
        let mut expanded_directory_paths = std::collections::HashSet::new();

        ProjectHierarchyViewData::expand_project_item_ancestor_directories(&mut expanded_directory_paths, &nested_directory_path);

        assert!(expanded_directory_paths.contains(&project_root_path));
        assert!(expanded_directory_paths.contains(&project_root_path.join("Player")));
        assert!(expanded_directory_paths.contains(&project_root_path.join("Player").join("Stats")));
    }

    #[test]
    fn collect_dragged_project_item_paths_uses_selected_items_when_dragging_selected_row() {
        let root_path = PathBuf::from("C:/Projects/TestProject/project_items");
        let first_child_path = root_path.join("First");
        let second_child_path = root_path.join("Second");
        let third_child_path = root_path.join("Third");
        let mut project_hierarchy_view_data = ProjectHierarchyViewData::new();
        project_hierarchy_view_data.tree_entries = vec![
            create_directory_tree_entry(&root_path, 0),
            create_directory_tree_entry(&first_child_path, 1),
            create_directory_tree_entry(&second_child_path, 1),
            create_directory_tree_entry(&third_child_path, 1),
        ];
        project_hierarchy_view_data
            .selected_project_item_paths
            .insert(first_child_path.clone());
        project_hierarchy_view_data
            .selected_project_item_paths
            .insert(third_child_path.clone());

        let dragged_project_item_paths = project_hierarchy_view_data.collect_dragged_project_item_paths(&third_child_path);

        assert_eq!(dragged_project_item_paths, vec![first_child_path, third_child_path]);
    }

    #[test]
    fn collect_dragged_project_item_paths_uses_only_anchor_when_dragging_unselected_row() {
        let root_path = PathBuf::from("C:/Projects/TestProject/project_items");
        let first_child_path = root_path.join("First");
        let second_child_path = root_path.join("Second");
        let mut project_hierarchy_view_data = ProjectHierarchyViewData::new();
        project_hierarchy_view_data.tree_entries = vec![
            create_directory_tree_entry(&root_path, 0),
            create_directory_tree_entry(&first_child_path, 1),
            create_directory_tree_entry(&second_child_path, 1),
        ];
        project_hierarchy_view_data
            .selected_project_item_paths
            .insert(first_child_path);

        let dragged_project_item_paths = project_hierarchy_view_data.collect_dragged_project_item_paths(&second_child_path);

        assert_eq!(dragged_project_item_paths, vec![second_child_path]);
    }
}
