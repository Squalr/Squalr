use crate::app_context::AppContext;
use crate::ui::list_navigation::{ListNavigationDirection, resolve_next_index};
use crate::views::project_explorer::project_hierarchy::project_hierarchy_clipboard_controller::ProjectHierarchyClipboardController;
use crate::views::project_explorer::project_hierarchy::project_hierarchy_drop_operation_planner::{
    ProjectHierarchyDropOperation, ProjectHierarchyDropOperationPlanner,
};
use crate::views::project_explorer::project_hierarchy::project_item_create_request_builder::ProjectItemCreateRequestBuilder;
use crate::views::project_explorer::project_hierarchy::view_data::{
    project_hierarchy_clipboard::{ProjectHierarchyClipboard, ProjectHierarchyClipboardMode},
    project_hierarchy_create_item_kind::ProjectHierarchyCreateItemKind,
    project_hierarchy_drop_target::ProjectHierarchyDropTarget,
    project_hierarchy_menu_target::ProjectHierarchyMenuTarget,
    project_hierarchy_pending_operation::ProjectHierarchyPendingOperation,
    project_hierarchy_take_over_state::ProjectHierarchyTakeOverState,
    project_hierarchy_tree_entry::ProjectHierarchyTreeEntry,
    project_hierarchy_tree_model::ProjectHierarchyTreeModel,
};
use eframe::egui::Pos2;
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::delete::project_items_delete_request::ProjectItemsDeleteRequest;
use squalr_engine_api::commands::project_items::duplicate::project_items_duplicate_request::ProjectItemsDuplicateRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::move_item::project_items_move_request::ProjectItemsMoveRequest;
use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_request::ProjectItemsPromoteSymbolRequest;
use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_response::ProjectItemsPromoteSymbolResponse;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_request::ProjectItemsReorderRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use squalr_engine_api::structures::settings::scan_settings::ScanSettings;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

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
    pub menu_target: Option<ProjectHierarchyMenuTarget>,
    pub menu_position: Option<Pos2>,
    pub dragged_project_item_paths: Option<Vec<PathBuf>>,
    pub project_item_clipboard: ProjectHierarchyClipboard,
    pub visible_preview_project_item_paths: Vec<PathBuf>,
    pub take_over_state: ProjectHierarchyTakeOverState,
    pub pending_operation: ProjectHierarchyPendingOperation,
    pub project_read_interval_ms: u64,
    pub is_querying_scan_settings: bool,
    pub last_scan_settings_sync_timestamp: Option<Instant>,
    pub last_project_read_timestamp: Option<Instant>,
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
            menu_target: None,
            menu_position: None,
            dragged_project_item_paths: None,
            project_item_clipboard: ProjectHierarchyClipboard::default(),
            visible_preview_project_item_paths: Vec::new(),
            take_over_state: ProjectHierarchyTakeOverState::None,
            pending_operation: ProjectHierarchyPendingOperation::None,
            project_read_interval_ms: ScanSettings::default().project_read_interval_ms,
            is_querying_scan_settings: false,
            last_scan_settings_sync_timestamp: None,
            last_project_read_timestamp: Some(Instant::now()),
        }
    }

    pub fn refresh_project_items(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
    ) {
        Self::refresh_project_items_with_after_refresh(project_hierarchy_view_data, app_context, None);
    }

    fn refresh_project_items_with_after_refresh(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
        after_refresh_callback: Option<Arc<dyn Fn() + Send + Sync>>,
    ) {
        let requested_preview_project_item_paths =
            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy view data refresh request") {
                if project_hierarchy_view_data.pending_operation == ProjectHierarchyPendingOperation::Refreshing {
                    return;
                }

                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Refreshing;

                Some(Vec::new())
            } else {
                None
            };
        let project_items_list_request = ProjectItemsListRequest {
            preview_project_item_paths: requested_preview_project_item_paths.clone(),
        };

        project_items_list_request.send(&app_context.engine_unprivileged_state, move |project_items_list_response| {
            let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy view data refresh response") {
                Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                None => return,
            };
            let previous_project_file_path = project_hierarchy_view_data
                .opened_project_info
                .as_ref()
                .map(|opened_project_info| opened_project_info.get_project_file_path().clone());

            project_hierarchy_view_data.opened_project_info = project_items_list_response.opened_project_info;
            project_hierarchy_view_data.opened_project_root = project_items_list_response.opened_project_root;
            project_hierarchy_view_data.project_items = Self::merge_project_item_preview_fields(
                &project_hierarchy_view_data.project_items,
                project_items_list_response.opened_project_items,
                requested_preview_project_item_paths.as_deref(),
            );

            if let Some(project_root_directory_path) = ProjectHierarchyTreeModel::resolve_project_root_path(
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
            project_hierarchy_view_data.retain_valid_take_over_state();
            let valid_project_item_paths = project_hierarchy_view_data
                .project_items
                .iter()
                .map(|(project_item_ref, _)| project_item_ref.get_project_item_path().clone())
                .collect::<HashSet<PathBuf>>();
            let current_project_file_path = project_hierarchy_view_data
                .opened_project_info
                .as_ref()
                .map(|opened_project_info| opened_project_info.get_project_file_path().clone());

            if previous_project_file_path != current_project_file_path {
                project_hierarchy_view_data.project_item_clipboard.clear();
            } else {
                let mut valid_project_item_paths_in_order = valid_project_item_paths
                    .iter()
                    .cloned()
                    .collect::<Vec<PathBuf>>();
                valid_project_item_paths_in_order.sort();
                project_hierarchy_view_data
                    .project_item_clipboard
                    .retain_valid_paths(&valid_project_item_paths_in_order);
            }
            project_hierarchy_view_data
                .visible_preview_project_item_paths
                .retain(|project_item_path| valid_project_item_paths.contains(project_item_path));

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
            drop(project_hierarchy_view_data);

            if let Some(after_refresh_callback) = &after_refresh_callback {
                after_refresh_callback();
            }
        });
    }

    pub fn set_visible_preview_project_item_paths(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        visible_preview_project_item_paths: Vec<PathBuf>,
    ) -> bool {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy set visible preview project item paths") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return false,
        };

        if project_hierarchy_view_data.visible_preview_project_item_paths == visible_preview_project_item_paths {
            return false;
        }

        project_hierarchy_view_data.visible_preview_project_item_paths = visible_preview_project_item_paths;

        true
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

    pub fn navigate_project_item_selection(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        direction: ListNavigationDirection,
        extend_selection: bool,
    ) -> Option<PathBuf> {
        let mut project_hierarchy_view_data = project_hierarchy_view_data.write("Project hierarchy navigate project item selection")?;
        let selected_project_item_index = project_hierarchy_view_data
            .selected_project_item_path
            .as_ref()
            .and_then(|selected_project_item_path| {
                project_hierarchy_view_data
                    .tree_entries
                    .iter()
                    .position(|tree_entry| &tree_entry.project_item_path == selected_project_item_path)
            });
        let next_selection_index = resolve_next_index(selected_project_item_index, project_hierarchy_view_data.tree_entries.len(), direction)?;
        let next_project_item_path = project_hierarchy_view_data
            .tree_entries
            .get(next_selection_index)?
            .project_item_path
            .clone();

        project_hierarchy_view_data.apply_selection(next_project_item_path.clone(), false, extend_selection);

        Some(next_project_item_path)
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

    fn retain_valid_take_over_state(&mut self) {
        let visible_project_item_paths: HashSet<PathBuf> = self
            .project_items
            .iter()
            .map(|(project_item_ref, _)| project_item_ref.get_project_item_path().clone())
            .collect();

        match &mut self.take_over_state {
            ProjectHierarchyTakeOverState::None => {}
            ProjectHierarchyTakeOverState::RenameProjectItem { project_item_path, .. }
            | ProjectHierarchyTakeOverState::EditProjectItemValue { project_item_path } => {
                if !visible_project_item_paths.contains(project_item_path) {
                    self.take_over_state = ProjectHierarchyTakeOverState::None;
                }
            }
            ProjectHierarchyTakeOverState::DeleteConfirmation { project_item_paths } => {
                project_item_paths.retain(|project_item_path| visible_project_item_paths.contains(project_item_path));

                if project_item_paths.is_empty() {
                    self.take_over_state = ProjectHierarchyTakeOverState::None;
                }
            }
            ProjectHierarchyTakeOverState::PromoteSymbolConflict { project_item_paths, conflicts } => {
                project_item_paths.retain(|project_item_path| visible_project_item_paths.contains(project_item_path));
                conflicts.retain(|conflict| visible_project_item_paths.contains(&conflict.project_item_path));

                if project_item_paths.is_empty() || conflicts.is_empty() {
                    self.take_over_state = ProjectHierarchyTakeOverState::None;
                }
            }
        }
    }

    pub fn collect_requested_preview_project_item_paths(&self) -> Vec<PathBuf> {
        let mut requested_preview_project_item_paths = self.visible_preview_project_item_paths.clone();

        if let Some(selected_project_item_path) = self.selected_project_item_path.as_ref() {
            requested_preview_project_item_paths.push(selected_project_item_path.clone());
        }

        if let ProjectHierarchyTakeOverState::EditProjectItemValue { project_item_path } = &self.take_over_state {
            requested_preview_project_item_paths.push(project_item_path.clone());
        }

        requested_preview_project_item_paths.sort();
        requested_preview_project_item_paths.dedup();

        requested_preview_project_item_paths
    }

    pub fn set_project_item_preview_fields(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        preview_fields_by_project_item_path: &HashMap<PathBuf, (String, String)>,
    ) -> bool {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy set project item preview fields") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return false,
        };
        let mut did_change = false;

        for (project_item_ref, project_item) in &mut project_hierarchy_view_data.project_items {
            let Some((preview_value, preview_path)) = preview_fields_by_project_item_path.get(project_item_ref.get_project_item_path()) else {
                continue;
            };
            let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

            if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
                let mut comparison_project_item = project_item.clone();
                let existing_preview_value = ProjectItemTypeAddress::get_field_freeze_data_value_interpreter(&mut comparison_project_item);

                if existing_preview_value != *preview_value {
                    ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(project_item, preview_value);
                    did_change = true;
                }
            } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
                let existing_preview_value = ProjectItemTypePointer::get_field_freeze_data_value_interpreter(project_item);
                let existing_preview_path = ProjectItemTypePointer::get_field_evaluated_pointer_path(project_item);

                if existing_preview_value != *preview_value {
                    ProjectItemTypePointer::set_field_freeze_data_value_interpreter(project_item, preview_value);
                    did_change = true;
                }

                if existing_preview_path != *preview_path {
                    ProjectItemTypePointer::set_field_evaluated_pointer_path(project_item, preview_path);
                    did_change = true;
                }
            }
        }

        if did_change {
            project_hierarchy_view_data.tree_entries = Self::build_tree_entries(
                project_hierarchy_view_data.opened_project_info.as_ref(),
                &project_hierarchy_view_data.project_items,
                &project_hierarchy_view_data.expanded_directory_paths,
            );
        }

        did_change
    }

    fn merge_project_item_preview_fields(
        previous_project_items: &[(ProjectItemRef, ProjectItem)],
        next_project_items: Vec<(ProjectItemRef, ProjectItem)>,
        refreshed_project_item_paths: Option<&[PathBuf]>,
    ) -> Vec<(ProjectItemRef, ProjectItem)> {
        let Some(refreshed_project_item_paths) = refreshed_project_item_paths else {
            return next_project_items;
        };
        let refreshed_project_item_path_set: HashSet<&PathBuf> = refreshed_project_item_paths.iter().collect();
        let previous_project_item_map: HashMap<&PathBuf, &ProjectItem> = previous_project_items
            .iter()
            .map(|(project_item_ref, project_item)| (project_item_ref.get_project_item_path(), project_item))
            .collect();

        next_project_items
            .into_iter()
            .map(|(project_item_ref, mut project_item)| {
                let project_item_path = project_item_ref.get_project_item_path();

                if !refreshed_project_item_path_set.contains(project_item_path) {
                    if let Some(previous_project_item) = previous_project_item_map.get(project_item_path) {
                        Self::copy_project_item_preview_fields(previous_project_item, &mut project_item);
                    }
                }

                (project_item_ref, project_item)
            })
            .collect()
    }

    fn copy_project_item_preview_fields(
        source_project_item: &ProjectItem,
        target_project_item: &mut ProjectItem,
    ) {
        let project_item_type_id = target_project_item.get_item_type().get_project_item_type_id();
        let preview_value = Self::read_project_item_preview_value(source_project_item);

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(target_project_item, &preview_value);
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            let preview_path = ProjectItemTypePointer::get_field_evaluated_pointer_path(source_project_item);

            ProjectItemTypePointer::set_field_freeze_data_value_interpreter(target_project_item, &preview_value);
            ProjectItemTypePointer::set_field_evaluated_pointer_path(target_project_item, &preview_path);
        }
    }

    fn read_project_item_preview_value(project_item: &ProjectItem) -> String {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();

            ProjectItemTypeAddress::get_field_freeze_data_value_interpreter(&mut project_item)
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypePointer::get_field_freeze_data_value_interpreter(project_item)
        } else {
            String::new()
        }
    }

    fn contains_promotable_project_item_paths(
        &self,
        project_item_paths: &[PathBuf],
    ) -> bool {
        project_item_paths
            .iter()
            .any(|project_item_path| self.is_promotable_project_item_path(project_item_path))
    }

    fn filter_promotable_project_item_paths(
        &self,
        project_item_paths: Vec<PathBuf>,
    ) -> Vec<PathBuf> {
        project_item_paths
            .into_iter()
            .filter(|project_item_path| self.is_promotable_project_item_path(project_item_path))
            .collect()
    }

    fn contains_strippable_symbol_project_item_paths(
        &self,
        project_item_paths: &[PathBuf],
    ) -> bool {
        project_item_paths
            .iter()
            .any(|project_item_path| self.is_strippable_symbol_project_item_path(project_item_path))
    }

    fn contains_symbolic_address_project_item_paths(
        &self,
        project_item_paths: &[PathBuf],
    ) -> bool {
        project_item_paths
            .iter()
            .any(|project_item_path| self.is_symbolic_address_project_item_path(project_item_path))
    }

    fn collect_strippable_symbol_project_item_paths(
        &self,
        project_item_paths: Vec<PathBuf>,
    ) -> Vec<PathBuf> {
        project_item_paths
            .into_iter()
            .filter(|project_item_path| self.is_strippable_symbol_project_item_path(project_item_path))
            .collect()
    }

    fn is_promotable_project_item_path(
        &self,
        project_item_path: &Path,
    ) -> bool {
        self.project_items
            .iter()
            .find(|(project_item_ref, _)| project_item_ref.get_project_item_path() == project_item_path)
            .map(|(_, project_item)| {
                let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

                project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID || project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID
            })
            .unwrap_or(false)
    }

    fn is_strippable_symbol_project_item_path(
        &self,
        project_item_path: &Path,
    ) -> bool {
        let Some(project_symbol_catalog) = self
            .opened_project_info
            .as_ref()
            .map(ProjectInfo::get_project_symbol_catalog)
        else {
            return false;
        };

        self.project_items
            .iter()
            .find(|(project_item_ref, _)| project_item_ref.get_project_item_path() == project_item_path)
            .map(|(_, project_item)| {
                if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
                    return false;
                }

                let mut project_item = project_item.clone();
                let address_target = ProjectItemTypeAddress::get_address_target(&mut project_item);

                address_target.has_symbolic_offsets()
                    && address_target
                        .strip_symbolic_offsets(project_symbol_catalog)
                        .is_some()
            })
            .unwrap_or(false)
    }

    fn is_symbolic_address_project_item_path(
        &self,
        project_item_path: &Path,
    ) -> bool {
        self.project_items
            .iter()
            .find(|(project_item_ref, _)| project_item_ref.get_project_item_path() == project_item_path)
            .map(|(_, project_item)| {
                if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
                    return false;
                }

                let mut project_item = project_item.clone();
                let address_target = ProjectItemTypeAddress::get_address_target(&mut project_item);

                address_target.has_symbolic_offsets()
            })
            .unwrap_or(false)
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
            .map(ProjectHierarchyTreeModel::is_directory_project_item)
            .unwrap_or(false)
        {
            Some(selected_project_item_path.clone())
        } else {
            selected_project_item_path.parent().map(Path::to_path_buf)
        }
    }

    pub fn show_project_item_menu(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_path: PathBuf,
        position: Pos2,
    ) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy show project item menu") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        project_hierarchy_view_data.menu_target = Some(ProjectHierarchyMenuTarget::ProjectItem(project_item_path));
        project_hierarchy_view_data.menu_position = Some(position);
    }

    pub fn show_add_menu(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        target_project_item_path: PathBuf,
        position: Pos2,
    ) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy show add menu") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        project_hierarchy_view_data.menu_target = Some(ProjectHierarchyMenuTarget::ToolbarAdd { target_project_item_path });
        project_hierarchy_view_data.menu_position = Some(position);
    }

    pub fn hide_menu(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy hide menu") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        project_hierarchy_view_data.menu_target = None;
        project_hierarchy_view_data.menu_position = None;
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

    pub fn request_rename_for_selected_project_item(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) {
        let selected_project_item_path = project_hierarchy_view_data
            .read("Project hierarchy selected item path for rename request")
            .and_then(|project_hierarchy_view_data| project_hierarchy_view_data.selected_project_item_path.clone());

        let Some(selected_project_item_path) = selected_project_item_path else {
            return;
        };

        Self::request_rename_for_project_item(project_hierarchy_view_data, selected_project_item_path);
    }

    pub fn request_rename_for_project_item(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_path: PathBuf,
    ) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy request selected item rename") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        if project_hierarchy_view_data.pending_operation != ProjectHierarchyPendingOperation::None {
            return;
        }

        let Some((_, selected_project_item)) = project_hierarchy_view_data
            .project_items
            .iter()
            .find(|(project_item_ref, _)| project_item_ref.get_project_item_path() == &project_item_path)
        else {
            return;
        };
        let selected_project_item_type_id = selected_project_item
            .get_item_type()
            .get_project_item_type_id()
            .to_string();
        let Some(project_root_path) = ProjectHierarchyTreeModel::resolve_project_root_path(
            project_hierarchy_view_data.opened_project_info.as_ref(),
            &project_hierarchy_view_data.project_items,
        ) else {
            return;
        };

        if project_item_path == project_root_path {
            return;
        }

        project_hierarchy_view_data.menu_target = None;
        project_hierarchy_view_data.menu_position = None;
        project_hierarchy_view_data.selected_project_item_paths.clear();
        project_hierarchy_view_data
            .selected_project_item_paths
            .insert(project_item_path.clone());
        project_hierarchy_view_data.selection_anchor_project_item_path = Some(project_item_path.clone());
        project_hierarchy_view_data.selected_project_item_path = Some(project_item_path.clone());
        project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::RenameProjectItem {
            project_item_path,
            project_item_type_id: selected_project_item_type_id,
        };
    }

    pub fn request_value_edit_for_selected_project_item(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) {
        let selected_project_item_path = project_hierarchy_view_data
            .read("Project hierarchy selected item path for value edit request")
            .and_then(|project_hierarchy_view_data| project_hierarchy_view_data.selected_project_item_path.clone());

        let Some(selected_project_item_path) = selected_project_item_path else {
            return;
        };

        Self::request_value_edit_for_project_item(project_hierarchy_view_data, selected_project_item_path);
    }

    pub fn request_value_edit_for_project_item(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_path: PathBuf,
    ) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy request value edit") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        if project_hierarchy_view_data.pending_operation != ProjectHierarchyPendingOperation::None {
            return;
        }

        let is_editable_value_project_item = project_hierarchy_view_data
            .project_items
            .iter()
            .find(|(project_item_ref, _)| project_item_ref.get_project_item_path() == &project_item_path)
            .map(|(_, project_item)| {
                let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

                project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID || project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID
            })
            .unwrap_or(false);

        if !is_editable_value_project_item {
            return;
        }

        project_hierarchy_view_data.menu_target = None;
        project_hierarchy_view_data.menu_position = None;
        project_hierarchy_view_data.selected_project_item_paths.clear();
        project_hierarchy_view_data
            .selected_project_item_paths
            .insert(project_item_path.clone());
        project_hierarchy_view_data.selection_anchor_project_item_path = Some(project_item_path.clone());
        project_hierarchy_view_data.selected_project_item_path = Some(project_item_path.clone());
        project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::EditProjectItemValue { project_item_path };
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
        let filtered_project_item_paths = project_hierarchy_view_data.filter_deletable_project_item_paths(project_item_paths);

        if filtered_project_item_paths.is_empty() {
            return;
        }

        project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::DeleteConfirmation {
            project_item_paths: filtered_project_item_paths,
        };
    }

    pub fn cancel_take_over(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy cancel take over") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::None;
    }

    pub fn finish_project_item_rename(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        previous_project_item_path: &Path,
        renamed_project_item_path: &Path,
    ) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy finish project item rename") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::None;

        if project_hierarchy_view_data
            .selected_project_item_path
            .as_deref()
            == Some(previous_project_item_path)
        {
            project_hierarchy_view_data.selected_project_item_path = Some(renamed_project_item_path.to_path_buf());
        }

        if project_hierarchy_view_data
            .selection_anchor_project_item_path
            .as_deref()
            == Some(previous_project_item_path)
        {
            project_hierarchy_view_data.selection_anchor_project_item_path = Some(renamed_project_item_path.to_path_buf());
        }

        if project_hierarchy_view_data
            .selected_project_item_paths
            .remove(previous_project_item_path)
        {
            project_hierarchy_view_data
                .selected_project_item_paths
                .insert(renamed_project_item_path.to_path_buf());
        }

        project_hierarchy_view_data.expanded_directory_paths = project_hierarchy_view_data
            .expanded_directory_paths
            .iter()
            .map(|expanded_directory_path| {
                Self::replace_project_item_path_prefix(expanded_directory_path, previous_project_item_path, renamed_project_item_path)
            })
            .collect();

        if let Some(dragged_project_item_paths) = project_hierarchy_view_data.dragged_project_item_paths.as_mut() {
            for dragged_project_item_path in dragged_project_item_paths.iter_mut() {
                *dragged_project_item_path =
                    Self::replace_project_item_path_prefix(dragged_project_item_path, previous_project_item_path, renamed_project_item_path);
            }
        }

        project_hierarchy_view_data
            .project_item_clipboard
            .update_path_prefix(previous_project_item_path, renamed_project_item_path);
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
        drop_target: ProjectHierarchyDropTarget,
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

            let drop_operation = ProjectHierarchyDropOperationPlanner::build(
                project_hierarchy_view_data.opened_project_info.as_ref(),
                &project_hierarchy_view_data.project_items,
                &dragged_project_item_paths,
                &drop_target,
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
            ProjectHierarchyDropOperation::MoveAndReorder {
                project_item_paths,
                target_directory_path,
                reordered_project_item_paths,
            } => {
                let project_items_move_request = ProjectItemsMoveRequest {
                    project_item_paths,
                    target_directory_path,
                };
                let app_context_for_reorder = app_context.clone();
                let project_hierarchy_view_data_for_reorder = project_hierarchy_view_data.clone();

                project_items_move_request.send(&app_context.engine_unprivileged_state, move |project_items_move_response| {
                    if !project_items_move_response.success {
                        log::error!(
                            "Failed to move project items before reorder. Moved count: {}.",
                            project_items_move_response.moved_project_item_count
                        );

                        if let Some(mut project_hierarchy_view_data) =
                            project_hierarchy_view_data_for_reorder.write("Project hierarchy move and reorder move response")
                        {
                            project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                        }

                        Self::refresh_project_items(project_hierarchy_view_data_for_reorder, app_context_for_reorder);
                        return;
                    }

                    let project_items_reorder_request = ProjectItemsReorderRequest {
                        project_item_paths: reordered_project_item_paths.clone(),
                    };
                    let app_context_after_reorder = app_context_for_reorder.clone();
                    let project_hierarchy_view_data_after_reorder = project_hierarchy_view_data_for_reorder.clone();

                    project_items_reorder_request.send(&app_context_for_reorder.engine_unprivileged_state, move |project_items_reorder_response| {
                        if !project_items_reorder_response.success {
                            log::error!(
                                "Failed to reorder project items after move. Reordered count: {}.",
                                project_items_reorder_response.reordered_project_item_count
                            );
                        }

                        if let Some(mut project_hierarchy_view_data) =
                            project_hierarchy_view_data_after_reorder.write("Project hierarchy move and reorder reorder response")
                        {
                            project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                        }

                        Self::refresh_project_items(project_hierarchy_view_data_after_reorder, app_context_after_reorder);
                    });
                });
            }
        }
    }

    pub fn delete_project_items(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
        project_item_paths: Vec<PathBuf>,
    ) {
        let filtered_project_item_paths = match project_hierarchy_view_data.write("Project hierarchy filter delete project items") {
            Some(mut project_hierarchy_view_data) => {
                let filtered_project_item_paths = project_hierarchy_view_data.filter_deletable_project_item_paths(project_item_paths);

                if filtered_project_item_paths.is_empty() {
                    project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::None;
                    return;
                }

                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Deleting;
                project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::None;

                filtered_project_item_paths
            }
            None => return,
        };

        let project_items_delete_request = ProjectItemsDeleteRequest {
            project_item_paths: filtered_project_item_paths,
        };
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

    pub fn copy_project_items(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_paths: Vec<PathBuf>,
    ) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy copy project items") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };
        let clipboard_project_item_paths = ProjectHierarchyClipboardController::filter_copy_paths(
            project_hierarchy_view_data.opened_project_info.as_ref(),
            &project_hierarchy_view_data.project_items,
            &project_hierarchy_view_data.tree_entries,
            project_item_paths,
        );

        if clipboard_project_item_paths.is_empty() {
            project_hierarchy_view_data.project_item_clipboard.clear();
            return;
        }

        let project_file_path = project_hierarchy_view_data
            .opened_project_info
            .as_ref()
            .map(|opened_project_info| opened_project_info.get_project_file_path().clone());
        project_hierarchy_view_data
            .project_item_clipboard
            .set(project_file_path, clipboard_project_item_paths, ProjectHierarchyClipboardMode::Copy);
        project_hierarchy_view_data.menu_target = None;
        project_hierarchy_view_data.menu_position = None;
    }

    pub fn cut_project_items(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_paths: Vec<PathBuf>,
    ) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy cut project items") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };
        let clipboard_project_item_paths = ProjectHierarchyClipboardController::filter_cut_paths(
            project_hierarchy_view_data.opened_project_info.as_ref(),
            &project_hierarchy_view_data.project_items,
            &project_hierarchy_view_data.tree_entries,
            project_item_paths,
        );

        if clipboard_project_item_paths.is_empty() {
            project_hierarchy_view_data.project_item_clipboard.clear();
            return;
        }

        let project_file_path = project_hierarchy_view_data
            .opened_project_info
            .as_ref()
            .map(|opened_project_info| opened_project_info.get_project_file_path().clone());
        project_hierarchy_view_data
            .project_item_clipboard
            .set(project_file_path, clipboard_project_item_paths, ProjectHierarchyClipboardMode::Cut);
        project_hierarchy_view_data.menu_target = None;
        project_hierarchy_view_data.menu_position = None;
    }

    pub fn clear_project_item_clipboard(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy clear project item clipboard") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        project_hierarchy_view_data.project_item_clipboard.clear();
    }

    pub fn has_project_item_clipboard(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) -> bool {
        project_hierarchy_view_data
            .read("Project hierarchy has project item clipboard")
            .map(|project_hierarchy_view_data| !project_hierarchy_view_data.project_item_clipboard.is_empty())
            .unwrap_or(false)
    }

    pub fn can_paste_project_item_clipboard(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        target_project_item_path: &Path,
    ) -> bool {
        project_hierarchy_view_data
            .read("Project hierarchy can paste project item clipboard")
            .map(|project_hierarchy_view_data| {
                ProjectHierarchyClipboardController::can_paste(
                    project_hierarchy_view_data.opened_project_info.as_ref(),
                    &project_hierarchy_view_data.project_items,
                    &project_hierarchy_view_data.project_item_clipboard,
                    target_project_item_path,
                )
            })
            .unwrap_or(false)
    }

    pub fn is_cut_project_item_path(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_path: &Path,
    ) -> bool {
        project_hierarchy_view_data
            .read("Project hierarchy is cut project item path")
            .map(|project_hierarchy_view_data| {
                ProjectHierarchyClipboardController::is_cut_project_item_path(&project_hierarchy_view_data.project_item_clipboard, project_item_path)
            })
            .unwrap_or(false)
    }

    pub fn paste_project_item_clipboard(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
        target_project_item_path: PathBuf,
    ) {
        enum PasteOperation {
            Copy {
                duplicate_request: ProjectItemsDuplicateRequest,
                insert_after_project_item_path: Option<PathBuf>,
            },
            CutMove {
                move_request: ProjectItemsMoveRequest,
                pasted_project_item_paths: Vec<PathBuf>,
            },
            CutMoveAndReorder {
                move_request: ProjectItemsMoveRequest,
                pasted_project_item_paths: Vec<PathBuf>,
                reordered_project_item_paths: Vec<PathBuf>,
            },
            CutReorder {
                pasted_project_item_paths: Vec<PathBuf>,
                reordered_project_item_paths: Vec<PathBuf>,
            },
        }

        let paste_operation = match project_hierarchy_view_data.write("Project hierarchy paste project item clipboard") {
            Some(mut project_hierarchy_view_data) => {
                if project_hierarchy_view_data.pending_operation != ProjectHierarchyPendingOperation::None {
                    return;
                }

                let paste_target =
                    ProjectHierarchyClipboardController::resolve_paste_target(&project_hierarchy_view_data.project_items, &target_project_item_path);
                let Some(current_project_file_path) = project_hierarchy_view_data
                    .opened_project_info
                    .as_ref()
                    .map(|opened_project_info| opened_project_info.get_project_file_path().clone())
                else {
                    return;
                };

                if project_hierarchy_view_data
                    .project_item_clipboard
                    .get_project_file_path()
                    != Some(&current_project_file_path)
                {
                    project_hierarchy_view_data.project_item_clipboard.clear();
                    return;
                }

                let clipboard_project_item_paths = project_hierarchy_view_data
                    .project_item_clipboard
                    .get_project_item_paths()
                    .to_vec();

                if clipboard_project_item_paths.is_empty() {
                    project_hierarchy_view_data.project_item_clipboard.clear();
                    return;
                }

                let clipboard_mode = project_hierarchy_view_data
                    .project_item_clipboard
                    .get_mode()
                    .cloned();
                let filtered_project_item_paths = ProjectHierarchyClipboardController::filter_pasteable_paths(
                    project_hierarchy_view_data.opened_project_info.as_ref(),
                    &project_hierarchy_view_data.project_items,
                    &clipboard_project_item_paths,
                    &paste_target,
                    clipboard_mode.as_ref(),
                );

                if filtered_project_item_paths.is_empty() {
                    if clipboard_mode == Some(ProjectHierarchyClipboardMode::Cut) {
                        project_hierarchy_view_data.project_item_clipboard.clear();
                    }
                    return;
                }

                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Pasting;
                project_hierarchy_view_data.menu_target = None;
                project_hierarchy_view_data.menu_position = None;

                match clipboard_mode {
                    Some(ProjectHierarchyClipboardMode::Copy) => PasteOperation::Copy {
                        duplicate_request: ProjectItemsDuplicateRequest {
                            project_item_paths: filtered_project_item_paths,
                            target_directory_path: paste_target.target_directory_path,
                        },
                        insert_after_project_item_path: paste_target.insert_after_project_item_path,
                    },
                    Some(ProjectHierarchyClipboardMode::Cut) => {
                        let pasted_project_item_paths = filtered_project_item_paths
                            .iter()
                            .map(|project_item_path| {
                                if project_item_path.parent() == Some(paste_target.target_directory_path.as_path()) {
                                    project_item_path.clone()
                                } else {
                                    paste_target
                                        .target_directory_path
                                        .join(project_item_path.file_name().unwrap_or_default())
                                }
                            })
                            .collect::<Vec<_>>();
                        let project_item_paths_to_move = filtered_project_item_paths
                            .iter()
                            .filter(|project_item_path| project_item_path.parent() != Some(paste_target.target_directory_path.as_path()))
                            .cloned()
                            .collect::<Vec<_>>();

                        if let Some(insert_after_project_item_path) = paste_target.insert_after_project_item_path.clone() {
                            let reordered_project_item_paths = ProjectHierarchyTreeModel::build_reorder_paths_after_target(
                                project_hierarchy_view_data.opened_project_info.as_ref(),
                                &project_hierarchy_view_data.project_items,
                                &insert_after_project_item_path,
                                &pasted_project_item_paths,
                                &filtered_project_item_paths,
                            );

                            match (project_item_paths_to_move.is_empty(), reordered_project_item_paths) {
                                (true, Some(reordered_project_item_paths)) => PasteOperation::CutReorder {
                                    pasted_project_item_paths,
                                    reordered_project_item_paths,
                                },
                                (false, Some(reordered_project_item_paths)) => PasteOperation::CutMoveAndReorder {
                                    move_request: ProjectItemsMoveRequest {
                                        project_item_paths: project_item_paths_to_move,
                                        target_directory_path: paste_target.target_directory_path,
                                    },
                                    pasted_project_item_paths,
                                    reordered_project_item_paths,
                                },
                                (false, None) => PasteOperation::CutMove {
                                    move_request: ProjectItemsMoveRequest {
                                        project_item_paths: project_item_paths_to_move,
                                        target_directory_path: paste_target.target_directory_path,
                                    },
                                    pasted_project_item_paths,
                                },
                                (true, None) => {
                                    project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                                    return;
                                }
                            }
                        } else {
                            PasteOperation::CutMove {
                                move_request: ProjectItemsMoveRequest {
                                    project_item_paths: project_item_paths_to_move,
                                    target_directory_path: paste_target.target_directory_path,
                                },
                                pasted_project_item_paths,
                            }
                        }
                    }
                    None => {
                        project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                        return;
                    }
                }
            }
            None => return,
        };
        let app_context_clone = app_context.clone();
        let project_hierarchy_view_data_clone = project_hierarchy_view_data.clone();
        let engine_unprivileged_state = app_context.engine_unprivileged_state.clone();

        match paste_operation {
            PasteOperation::Copy {
                duplicate_request,
                insert_after_project_item_path,
            } => {
                duplicate_request.send(&engine_unprivileged_state, move |project_items_duplicate_response| {
                    if !project_items_duplicate_response.success {
                        log::error!(
                            "Failed to duplicate one or more project items. Duplicated count: {}.",
                            project_items_duplicate_response.duplicated_project_item_count
                        );
                    }

                    let duplicated_project_item_paths = project_items_duplicate_response
                        .duplicated_project_item_paths
                        .clone();
                    let reordered_project_item_paths = insert_after_project_item_path
                        .as_ref()
                        .and_then(|insert_after_project_item_path| {
                            project_hierarchy_view_data_clone
                                .read("Project hierarchy duplicate project items reorder plan")
                                .and_then(|project_hierarchy_view_data| {
                                    ProjectHierarchyTreeModel::build_reorder_paths_after_target(
                                        project_hierarchy_view_data.opened_project_info.as_ref(),
                                        &project_hierarchy_view_data.project_items,
                                        insert_after_project_item_path,
                                        &duplicated_project_item_paths,
                                        &[],
                                    )
                                })
                        });

                    if let Some(reordered_project_item_paths) = reordered_project_item_paths {
                        let app_context_after_reorder = app_context_clone.clone();
                        let project_hierarchy_view_data_after_reorder = project_hierarchy_view_data_clone.clone();
                        let duplicated_project_item_paths_after_reorder = duplicated_project_item_paths.clone();
                        let project_items_reorder_request = ProjectItemsReorderRequest {
                            project_item_paths: reordered_project_item_paths,
                        };
                        let engine_unprivileged_state_after_reorder = app_context_clone.engine_unprivileged_state.clone();

                        project_items_reorder_request.send(&engine_unprivileged_state_after_reorder, move |project_items_reorder_response| {
                            if !project_items_reorder_response.success {
                                log::error!(
                                    "Failed to reorder duplicated project items. Reordered count: {}.",
                                    project_items_reorder_response.reordered_project_item_count
                                );
                            }

                            if let Some(mut project_hierarchy_view_data) =
                                project_hierarchy_view_data_after_reorder.write("Project hierarchy duplicate project items reorder response")
                            {
                                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                                Self::apply_pasted_project_item_selection(&mut project_hierarchy_view_data, &duplicated_project_item_paths_after_reorder);
                            }

                            Self::refresh_project_items(project_hierarchy_view_data_after_reorder, app_context_after_reorder);
                        });

                        return;
                    }

                    if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data_clone.write("Project hierarchy duplicate project items response")
                    {
                        project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                        Self::apply_pasted_project_item_selection(&mut project_hierarchy_view_data, &duplicated_project_item_paths);
                    }

                    Self::refresh_project_items(project_hierarchy_view_data_clone, app_context_clone);
                });
            }
            PasteOperation::CutMove {
                move_request,
                pasted_project_item_paths,
            } => {
                move_request.send(&engine_unprivileged_state, move |project_items_move_response| {
                    if !project_items_move_response.success {
                        log::error!(
                            "Failed to paste cut project items. Moved count: {}.",
                            project_items_move_response.moved_project_item_count
                        );
                    }

                    if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data_clone.write("Project hierarchy move cut project items response")
                    {
                        project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;

                        if project_items_move_response.success {
                            project_hierarchy_view_data.project_item_clipboard.clear();
                            Self::apply_pasted_project_item_selection(&mut project_hierarchy_view_data, &pasted_project_item_paths);
                        }
                    }

                    Self::refresh_project_items(project_hierarchy_view_data_clone, app_context_clone);
                });
            }
            PasteOperation::CutMoveAndReorder {
                move_request,
                pasted_project_item_paths,
                reordered_project_item_paths,
            } => {
                move_request.send(&engine_unprivileged_state, move |project_items_move_response| {
                    if !project_items_move_response.success {
                        log::error!(
                            "Failed to move cut project items before reorder. Moved count: {}.",
                            project_items_move_response.moved_project_item_count
                        );

                        if let Some(mut project_hierarchy_view_data) =
                            project_hierarchy_view_data_clone.write("Project hierarchy move and reorder cut project items move response")
                        {
                            project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                        }

                        Self::refresh_project_items(project_hierarchy_view_data_clone, app_context_clone);
                        return;
                    }

                    if let Some(mut project_hierarchy_view_data) =
                        project_hierarchy_view_data_clone.write("Project hierarchy move and reorder cut project items move success")
                    {
                        project_hierarchy_view_data.project_item_clipboard.clear();
                    }

                    let app_context_after_reorder = app_context_clone.clone();
                    let project_hierarchy_view_data_after_reorder = project_hierarchy_view_data_clone.clone();
                    let pasted_project_item_paths_after_reorder = pasted_project_item_paths.clone();
                    let project_items_reorder_request = ProjectItemsReorderRequest {
                        project_item_paths: reordered_project_item_paths.clone(),
                    };
                    let engine_unprivileged_state_after_reorder = app_context_clone.engine_unprivileged_state.clone();

                    project_items_reorder_request.send(&engine_unprivileged_state_after_reorder, move |project_items_reorder_response| {
                        if !project_items_reorder_response.success {
                            log::error!(
                                "Failed to reorder cut project items after move. Reordered count: {}.",
                                project_items_reorder_response.reordered_project_item_count
                            );
                        }

                        if let Some(mut project_hierarchy_view_data) =
                            project_hierarchy_view_data_after_reorder.write("Project hierarchy move and reorder cut project items reorder response")
                        {
                            project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                            Self::apply_pasted_project_item_selection(&mut project_hierarchy_view_data, &pasted_project_item_paths_after_reorder);
                        }

                        Self::refresh_project_items(project_hierarchy_view_data_after_reorder, app_context_after_reorder);
                    });
                });
            }
            PasteOperation::CutReorder {
                pasted_project_item_paths,
                reordered_project_item_paths,
            } => {
                let project_items_reorder_request = ProjectItemsReorderRequest {
                    project_item_paths: reordered_project_item_paths,
                };

                project_items_reorder_request.send(&engine_unprivileged_state, move |project_items_reorder_response| {
                    if !project_items_reorder_response.success {
                        log::error!(
                            "Failed to reorder cut project items. Reordered count: {}.",
                            project_items_reorder_response.reordered_project_item_count
                        );
                    }

                    if let Some(mut project_hierarchy_view_data) =
                        project_hierarchy_view_data_clone.write("Project hierarchy reorder cut project items response")
                    {
                        project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;

                        if project_items_reorder_response.success {
                            project_hierarchy_view_data.project_item_clipboard.clear();
                            Self::apply_pasted_project_item_selection(&mut project_hierarchy_view_data, &pasted_project_item_paths);
                        }
                    }

                    Self::refresh_project_items(project_hierarchy_view_data_clone, app_context_clone);
                });
            }
        }
    }

    pub fn has_deletable_selected_project_item(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) -> bool {
        project_hierarchy_view_data
            .read("Project hierarchy has deletable selected project item")
            .map(|project_hierarchy_view_data| {
                let selected_project_item_paths = project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order();

                project_hierarchy_view_data.contains_deletable_project_item_paths(&selected_project_item_paths)
            })
            .unwrap_or(false)
    }

    pub fn has_deletable_project_item_paths(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_paths: &[PathBuf],
    ) -> bool {
        project_hierarchy_view_data
            .read("Project hierarchy has deletable project item paths")
            .map(|project_hierarchy_view_data| project_hierarchy_view_data.contains_deletable_project_item_paths(project_item_paths))
            .unwrap_or(false)
    }

    pub fn has_promotable_project_item_paths(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_paths: &[PathBuf],
    ) -> bool {
        project_hierarchy_view_data
            .read("Project hierarchy has promotable project item paths")
            .map(|project_hierarchy_view_data| project_hierarchy_view_data.contains_promotable_project_item_paths(project_item_paths))
            .unwrap_or(false)
    }

    pub fn has_strippable_symbol_project_item_paths(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_paths: &[PathBuf],
    ) -> bool {
        project_hierarchy_view_data
            .read("Project hierarchy has strippable symbol project item paths")
            .map(|project_hierarchy_view_data| project_hierarchy_view_data.contains_strippable_symbol_project_item_paths(project_item_paths))
            .unwrap_or(false)
    }

    pub fn has_symbolic_address_project_item_paths(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_paths: &[PathBuf],
    ) -> bool {
        project_hierarchy_view_data
            .read("Project hierarchy has symbolic address project item paths")
            .map(|project_hierarchy_view_data| project_hierarchy_view_data.contains_symbolic_address_project_item_paths(project_item_paths))
            .unwrap_or(false)
    }

    pub fn filter_strippable_symbol_project_item_paths(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_paths: Vec<PathBuf>,
    ) -> Vec<PathBuf> {
        project_hierarchy_view_data
            .read("Project hierarchy filter strippable symbol project item paths")
            .map(|project_hierarchy_view_data| project_hierarchy_view_data.collect_strippable_symbol_project_item_paths(project_item_paths))
            .unwrap_or_default()
    }

    pub fn promote_project_items_to_symbols(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
        project_item_paths: Vec<PathBuf>,
        overwrite_conflicting_symbols: bool,
        after_successful_refresh_callback: Option<Arc<dyn Fn() + Send + Sync>>,
    ) {
        let filtered_project_item_paths = match project_hierarchy_view_data.write("Project hierarchy filter promote project items") {
            Some(mut project_hierarchy_view_data) => {
                let filtered_project_item_paths = project_hierarchy_view_data.filter_promotable_project_item_paths(project_item_paths);

                if filtered_project_item_paths.is_empty() {
                    return;
                }

                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Promoting;

                filtered_project_item_paths
            }
            None => return,
        };
        let promote_conflict_project_item_paths = filtered_project_item_paths.clone();
        let project_items_promote_symbol_request = ProjectItemsPromoteSymbolRequest {
            project_item_paths: filtered_project_item_paths,
            overwrite_conflicting_symbols,
        };
        let app_context_clone = app_context.clone();
        let project_hierarchy_view_data_clone = project_hierarchy_view_data.clone();

        project_items_promote_symbol_request.send(&app_context.engine_unprivileged_state, move |project_items_promote_symbol_response| {
            if !project_items_promote_symbol_response.success {
                if project_items_promote_symbol_response.status_message.is_empty() {
                    log::error!(
                        "Failed to promote one or more project items to symbols. Promoted count before failure: {}.",
                        project_items_promote_symbol_response.promoted_symbol_count
                    );
                } else {
                    log::warn!("{}", project_items_promote_symbol_response.status_message);
                }
            }

            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data_clone.write("Project hierarchy promote project items response") {
                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;

                if !project_items_promote_symbol_response.conflicts.is_empty() {
                    project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::PromoteSymbolConflict {
                        project_item_paths: promote_conflict_project_item_paths.clone(),
                        conflicts: project_items_promote_symbol_response.conflicts.clone(),
                    };
                } else {
                    project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::None;
                }
            }

            let after_refresh_callback = if Self::should_refocus_details_after_promote_response(&project_items_promote_symbol_response) {
                after_successful_refresh_callback.clone()
            } else {
                None
            };

            Self::refresh_project_items_with_after_refresh(project_hierarchy_view_data_clone, app_context_clone, after_refresh_callback);
        });
    }

    fn should_refocus_details_after_promote_response(project_items_promote_symbol_response: &ProjectItemsPromoteSymbolResponse) -> bool {
        project_items_promote_symbol_response.success
            && project_items_promote_symbol_response.conflicts.is_empty()
            && project_items_promote_symbol_response
                .promoted_symbol_count
                .saturating_add(project_items_promote_symbol_response.reused_symbol_count)
                > 0
    }

    pub fn create_project_item(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
        target_project_item_path: PathBuf,
        create_item_kind: ProjectHierarchyCreateItemKind,
    ) {
        let project_items_create_request = match project_hierarchy_view_data.write("Project hierarchy resolve create project item target") {
            Some(project_hierarchy_view_data) => {
                ProjectItemCreateRequestBuilder::build(&project_hierarchy_view_data.project_items, &target_project_item_path, create_item_kind)
            }
            None => return,
        };
        let app_context_clone = app_context.clone();
        let project_hierarchy_view_data_clone = project_hierarchy_view_data.clone();

        project_items_create_request.send(&app_context.engine_unprivileged_state, move |project_items_create_response| {
            if !project_items_create_response.success {
                log::error!("Failed to create project item.");
                return;
            }

            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data_clone.write("Project hierarchy select created project item") {
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

    pub fn get_selected_or_root_directory_path(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) -> Option<PathBuf> {
        project_hierarchy_view_data
            .read("Project hierarchy selected or root directory path")
            .and_then(|project_hierarchy_view_data| {
                project_hierarchy_view_data
                    .selected_project_item_path
                    .as_ref()
                    .map(|selected_project_item_path| {
                        ProjectItemCreateRequestBuilder::resolve_parent_directory_path(&project_hierarchy_view_data.project_items, selected_project_item_path)
                    })
                    .or_else(|| {
                        ProjectHierarchyTreeModel::resolve_project_root_path(
                            project_hierarchy_view_data.opened_project_info.as_ref(),
                            &project_hierarchy_view_data.project_items,
                        )
                    })
            })
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
        let project_hierarchy_tree_model = match ProjectHierarchyTreeModel::build(opened_project_info, project_items) {
            Some(project_hierarchy_tree_model) => project_hierarchy_tree_model,
            None => return Vec::new(),
        };
        let project_root_directory_path = project_hierarchy_tree_model.root_directory_path;
        let project_item_map = project_hierarchy_tree_model.project_item_map;
        let child_paths_by_parent_path = project_hierarchy_tree_model.child_paths_by_parent_path;

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
                preview_path: Self::build_preview_path(project_item),
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
            let is_directory = ProjectHierarchyTreeModel::is_directory_project_item(project_item);
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
                preview_path: Self::build_preview_path(project_item),
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

    fn apply_pasted_project_item_selection(
        project_hierarchy_view_data: &mut ProjectHierarchyViewData,
        pasted_project_item_paths: &[PathBuf],
    ) {
        if pasted_project_item_paths.is_empty() {
            return;
        }

        project_hierarchy_view_data.selected_project_item_path = pasted_project_item_paths.first().cloned();
        project_hierarchy_view_data.selected_project_item_paths = pasted_project_item_paths.iter().cloned().collect();
        project_hierarchy_view_data.selection_anchor_project_item_path = pasted_project_item_paths.first().cloned();

        for pasted_project_item_path in pasted_project_item_paths {
            Self::expand_project_item_ancestor_directories(&mut project_hierarchy_view_data.expanded_directory_paths, pasted_project_item_path);
        }
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

    fn build_preview_path(project_item: &ProjectItem) -> String {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            Self::read_string_field(project_item, ProjectItemTypePointer::PROPERTY_EVALUATED_POINTER_PATH)
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

    fn filter_deletable_project_item_paths(
        &self,
        project_item_paths: Vec<PathBuf>,
    ) -> Vec<PathBuf> {
        project_item_paths
            .into_iter()
            .filter(|project_item_path| !self.is_protected_project_item_path(project_item_path))
            .collect()
    }

    fn contains_deletable_project_item_paths(
        &self,
        project_item_paths: &[PathBuf],
    ) -> bool {
        project_item_paths
            .iter()
            .any(|project_item_path| !self.is_protected_project_item_path(project_item_path))
    }

    fn is_protected_project_item_path(
        &self,
        project_item_path: &Path,
    ) -> bool {
        ProjectHierarchyTreeModel::resolve_project_root_path(self.opened_project_info.as_ref(), &self.project_items)
            .as_ref()
            .map(|root_project_item_path| root_project_item_path == project_item_path)
            .unwrap_or(false)
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

    fn replace_project_item_path_prefix(
        project_item_path: &Path,
        previous_project_item_path: &Path,
        renamed_project_item_path: &Path,
    ) -> PathBuf {
        if !project_item_path.starts_with(previous_project_item_path) {
            return project_item_path.to_path_buf();
        }

        let renamed_child_suffix = project_item_path
            .strip_prefix(previous_project_item_path)
            .unwrap_or_else(|_| Path::new(""));

        renamed_project_item_path.join(renamed_child_suffix)
    }
}
