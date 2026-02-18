use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::project_explorer::entry_rows::{build_visible_project_entry_rows, build_visible_project_item_entry_rows};
use crate::views::project_explorer::hierarchy_graph::{build_project_item_hierarchy_graph, is_directory_project_item};
use crate::views::project_explorer::hierarchy_visibility::build_visible_hierarchy_entries;
use crate::views::project_explorer::hierarchy_walk::build_project_item_paths_preorder;
use crate::views::project_explorer::summary::build_project_explorer_summary_lines;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Stores text input mode for project selector operations.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProjectSelectorInputMode {
    #[default]
    None,
    Search,
    CreatingProject,
    RenamingProject,
    CreatingProjectDirectory,
}

/// Stores current focus target for the project explorer pane.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProjectExplorerFocusTarget {
    #[default]
    ProjectList,
    ProjectHierarchy,
}

/// Stores a visible hierarchy entry for a project item.
#[derive(Clone, Debug)]
pub struct ProjectHierarchyEntry {
    pub project_item_path: PathBuf,
    pub display_name: String,
    pub depth: usize,
    pub is_directory: bool,
    pub is_expanded: bool,
    pub is_activated: bool,
}

/// Stores state for browsing projects and project items.
#[derive(Clone, Debug)]
pub struct ProjectExplorerPaneState {
    pub all_project_entries: Vec<ProjectInfo>,
    pub project_entries: Vec<ProjectInfo>,
    pub selected_project_list_index: Option<usize>,
    pub selected_project_name: Option<String>,
    pub selected_project_directory_path: Option<PathBuf>,
    pub active_project_name: Option<String>,
    pub active_project_directory_path: Option<PathBuf>,
    pub selected_item_path: Option<String>,
    pub is_hierarchy_expanded: bool,
    pub focus_target: ProjectExplorerFocusTarget,
    pub input_mode: ProjectSelectorInputMode,
    pub pending_search_name_input: String,
    pub pending_project_name_input: String,
    pub has_loaded_project_list_once: bool,
    pub is_awaiting_project_list_response: bool,
    pub is_creating_project: bool,
    pub is_opening_project: bool,
    pub is_renaming_project: bool,
    pub is_deleting_project: bool,
    pub is_closing_project: bool,
    pub has_loaded_project_item_list_once: bool,
    pub is_awaiting_project_item_list_response: bool,
    pub is_creating_project_item: bool,
    pub is_deleting_project_item: bool,
    pub is_moving_project_item: bool,
    pub is_reordering_project_item: bool,
    pub is_toggling_project_item_activation: bool,
    pub project_item_visible_entries: Vec<ProjectHierarchyEntry>,
    pub selected_project_item_visible_index: Option<usize>,
    pub pending_move_source_paths: Vec<PathBuf>,
    pub pending_delete_confirmation_paths: Vec<PathBuf>,
    pub status_message: String,
    opened_project_item_map: HashMap<PathBuf, ProjectItem>,
    child_paths_by_parent_path: HashMap<PathBuf, Vec<PathBuf>>,
    root_project_item_paths: Vec<PathBuf>,
    expanded_directory_paths: HashSet<PathBuf>,
}

impl ProjectExplorerPaneState {
    /// Synchronizes explorer focus target based on whether a project is currently open.
    fn sync_focus_target_to_project_context(&mut self) {
        self.focus_target = if self.active_project_directory_path.is_some() {
            ProjectExplorerFocusTarget::ProjectHierarchy
        } else {
            ProjectExplorerFocusTarget::ProjectList
        };
    }

    pub fn apply_project_list(
        &mut self,
        project_entries: Vec<ProjectInfo>,
    ) {
        self.all_project_entries = project_entries;
        self.apply_search_filter_to_projects();
    }

    pub fn apply_search_filter_to_projects(&mut self) {
        let selected_project_directory_path_before_refresh = self.selected_project_directory_path.clone();
        let search_name_filter = self
            .pending_search_name_trimmed()
            .map(|search_name| search_name.to_ascii_lowercase());
        self.project_entries = match search_name_filter {
            Some(search_name_filter) => self
                .all_project_entries
                .iter()
                .filter(|project_entry| {
                    project_entry
                        .get_name()
                        .to_ascii_lowercase()
                        .contains(&search_name_filter)
                })
                .cloned()
                .collect(),
            None => self.all_project_entries.clone(),
        };
        self.selected_project_list_index = selected_project_directory_path_before_refresh
            .as_ref()
            .and_then(|selected_project_directory_path| {
                self.project_entries.iter().position(|project_entry| {
                    project_entry
                        .get_project_directory()
                        .as_deref()
                        .is_some_and(|project_directory| project_directory == selected_project_directory_path.as_path())
                })
            })
            .or_else(|| if self.project_entries.is_empty() { None } else { Some(0) });
        self.update_selected_project_fields();
    }

    pub fn apply_project_items_list(
        &mut self,
        opened_project_items: Vec<(ProjectItemRef, ProjectItem)>,
    ) {
        let selected_project_item_path_before_refresh = self.selected_project_item_path();
        let hierarchy_graph = build_project_item_hierarchy_graph(opened_project_items);
        self.opened_project_item_map = hierarchy_graph.opened_project_item_map;
        self.child_paths_by_parent_path = hierarchy_graph.child_paths_by_parent_path;
        self.root_project_item_paths = hierarchy_graph.root_project_item_paths;
        self.expanded_directory_paths.retain(|expanded_directory_path| {
            hierarchy_graph
                .valid_directory_paths
                .contains(expanded_directory_path)
        });
        self.pending_move_source_paths
            .retain(|pending_move_source_path| {
                hierarchy_graph
                    .valid_project_item_paths
                    .contains(pending_move_source_path)
            });
        self.pending_delete_confirmation_paths
            .retain(|pending_delete_confirmation_path| {
                hierarchy_graph
                    .valid_project_item_paths
                    .contains(pending_delete_confirmation_path)
            });

        self.rebuild_visible_hierarchy_entries();
        self.restore_selected_project_item_path(selected_project_item_path_before_refresh);
        self.has_loaded_project_item_list_once = true;
    }

    pub fn clear_project_items(&mut self) {
        self.opened_project_item_map.clear();
        self.child_paths_by_parent_path.clear();
        self.root_project_item_paths.clear();
        self.project_item_visible_entries.clear();
        self.selected_project_item_visible_index = None;
        self.pending_move_source_paths.clear();
        self.pending_delete_confirmation_paths.clear();
        self.selected_item_path = None;
        self.has_loaded_project_item_list_once = false;
    }

    pub fn select_next_project_item(&mut self) {
        if self.project_item_visible_entries.is_empty() {
            self.selected_project_item_visible_index = None;
            self.update_selected_item_path();
            return;
        }

        let selected_project_item_visible_index = self.selected_project_item_visible_index.unwrap_or(0);
        let next_project_item_visible_index = (selected_project_item_visible_index + 1) % self.project_item_visible_entries.len();
        self.selected_project_item_visible_index = Some(next_project_item_visible_index);
        self.update_selected_item_path();
    }

    pub fn select_previous_project_item(&mut self) {
        if self.project_item_visible_entries.is_empty() {
            self.selected_project_item_visible_index = None;
            self.update_selected_item_path();
            return;
        }

        let selected_project_item_visible_index = self.selected_project_item_visible_index.unwrap_or(0);
        let previous_project_item_visible_index = if selected_project_item_visible_index == 0 {
            self.project_item_visible_entries.len() - 1
        } else {
            selected_project_item_visible_index - 1
        };

        self.selected_project_item_visible_index = Some(previous_project_item_visible_index);
        self.update_selected_item_path();
    }

    pub fn selected_project_item_path(&self) -> Option<PathBuf> {
        let selected_project_item_visible_index = self.selected_project_item_visible_index?;
        self.project_item_visible_entries
            .get(selected_project_item_visible_index)
            .map(|project_item_entry| project_item_entry.project_item_path.clone())
    }

    pub fn selected_project_items_for_struct_viewer(&self) -> Vec<(PathBuf, ProjectItem)> {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return Vec::new();
        };
        let Some(selected_project_item) = self.opened_project_item_map.get(&selected_project_item_path) else {
            return Vec::new();
        };

        vec![(selected_project_item_path, selected_project_item.clone())]
    }

    pub fn selected_project_item_directory_target_path(&self) -> Option<PathBuf> {
        let selected_project_item_path = self.selected_project_item_path()?;
        if self.is_directory_path(&selected_project_item_path) {
            Some(selected_project_item_path)
        } else {
            selected_project_item_path.parent().map(Path::to_path_buf)
        }
    }

    pub fn expand_selected_project_item_directory(&mut self) -> bool {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return false;
        };

        if !self.is_directory_path(&selected_project_item_path) {
            return false;
        }

        let was_inserted = self
            .expanded_directory_paths
            .insert(selected_project_item_path.clone());
        if was_inserted {
            self.rebuild_visible_hierarchy_entries();
            self.restore_selected_project_item_path(Some(selected_project_item_path));
        }

        was_inserted
    }

    pub fn collapse_selected_project_item_directory_or_select_parent(&mut self) -> bool {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return false;
        };

        if self
            .expanded_directory_paths
            .remove(&selected_project_item_path)
        {
            self.rebuild_visible_hierarchy_entries();
            self.restore_selected_project_item_path(Some(selected_project_item_path));
            return true;
        }

        let Some(parent_directory_path) = selected_project_item_path.parent().map(Path::to_path_buf) else {
            return false;
        };
        let parent_project_item_visible_index = self
            .project_item_visible_entries
            .iter()
            .position(|project_item_entry| project_item_entry.project_item_path == parent_directory_path);
        if let Some(parent_project_item_visible_index) = parent_project_item_visible_index {
            self.selected_project_item_visible_index = Some(parent_project_item_visible_index);
            self.update_selected_item_path();
            return true;
        }

        false
    }

    pub fn selected_project_item_is_activated(&self) -> bool {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return false;
        };
        self.opened_project_item_map
            .get(&selected_project_item_path)
            .map(ProjectItem::get_is_activated)
            .unwrap_or(false)
    }

    pub fn stage_selected_project_item_for_move(&mut self) -> bool {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return false;
        };
        self.pending_move_source_paths = vec![selected_project_item_path];
        true
    }

    pub fn has_pending_move_source_paths(&self) -> bool {
        !self.pending_move_source_paths.is_empty()
    }

    pub fn pending_move_source_paths(&self) -> Vec<PathBuf> {
        self.pending_move_source_paths.clone()
    }

    pub fn clear_pending_move_source_paths(&mut self) {
        self.pending_move_source_paths.clear();
    }

    pub fn arm_delete_confirmation_for_selected_project_item(&mut self) -> bool {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return false;
        };
        self.pending_delete_confirmation_paths = vec![selected_project_item_path];
        true
    }

    pub fn has_pending_delete_confirmation_for_selected_project_item(&self) -> bool {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return false;
        };
        self.pending_delete_confirmation_paths == vec![selected_project_item_path]
    }

    pub fn take_pending_delete_confirmation_paths(&mut self) -> Vec<PathBuf> {
        let pending_delete_confirmation_paths = self.pending_delete_confirmation_paths.clone();
        self.pending_delete_confirmation_paths.clear();
        pending_delete_confirmation_paths
    }

    pub fn build_reorder_request_paths_for_selected_project_item(
        &self,
        move_toward_previous_position: bool,
    ) -> Option<Vec<PathBuf>> {
        let selected_project_item_path = self.selected_project_item_path()?;
        let parent_directory_path = selected_project_item_path.parent().map(Path::to_path_buf);
        let mut child_paths_by_parent_path = self.child_paths_by_parent_path.clone();
        let mut root_project_item_paths = self.root_project_item_paths.clone();

        if let Some(parent_directory_path) = parent_directory_path {
            if self
                .opened_project_item_map
                .contains_key(&parent_directory_path)
            {
                let sibling_paths = child_paths_by_parent_path.get_mut(&parent_directory_path)?;
                let selected_sibling_position = sibling_paths
                    .iter()
                    .position(|sibling_path| sibling_path == &selected_project_item_path)?;
                if move_toward_previous_position {
                    if selected_sibling_position == 0 {
                        return None;
                    }
                    sibling_paths.swap(selected_sibling_position, selected_sibling_position - 1);
                } else {
                    let next_sibling_position = selected_sibling_position + 1;
                    if next_sibling_position >= sibling_paths.len() {
                        return None;
                    }
                    sibling_paths.swap(selected_sibling_position, next_sibling_position);
                }
            } else {
                let selected_root_position = root_project_item_paths
                    .iter()
                    .position(|root_path| root_path == &selected_project_item_path)?;
                if move_toward_previous_position {
                    if selected_root_position == 0 {
                        return None;
                    }
                    root_project_item_paths.swap(selected_root_position, selected_root_position - 1);
                } else {
                    let next_root_position = selected_root_position + 1;
                    if next_root_position >= root_project_item_paths.len() {
                        return None;
                    }
                    root_project_item_paths.swap(selected_root_position, next_root_position);
                }
            }
        }

        let reordered_project_item_paths = build_project_item_paths_preorder(&root_project_item_paths, &child_paths_by_parent_path);

        Some(
            reordered_project_item_paths
                .into_iter()
                .filter(|project_item_path| {
                    !project_item_path
                        .file_name()
                        .and_then(|file_name| file_name.to_str())
                        .is_some_and(|file_name| file_name == Project::PROJECT_DIR)
                })
                .collect(),
        )
    }

    pub fn select_next_project(&mut self) {
        if self.project_entries.is_empty() {
            self.selected_project_list_index = None;
            self.update_selected_project_fields();
            return;
        }

        let selected_project_index = self.selected_project_list_index.unwrap_or(0);
        let next_project_index = (selected_project_index + 1) % self.project_entries.len();
        self.selected_project_list_index = Some(next_project_index);
        self.update_selected_project_fields();
    }

    pub fn select_previous_project(&mut self) {
        if self.project_entries.is_empty() {
            self.selected_project_list_index = None;
            self.update_selected_project_fields();
            return;
        }

        let selected_project_index = self.selected_project_list_index.unwrap_or(0);
        let previous_project_index = if selected_project_index == 0 {
            self.project_entries.len() - 1
        } else {
            selected_project_index - 1
        };
        self.selected_project_list_index = Some(previous_project_index);
        self.update_selected_project_fields();
    }

    pub fn select_first_project(&mut self) {
        if self.project_entries.is_empty() {
            self.selected_project_list_index = None;
            self.update_selected_project_fields();
            return;
        }

        self.selected_project_list_index = Some(0);
        self.update_selected_project_fields();
    }

    pub fn select_last_project(&mut self) {
        if self.project_entries.is_empty() {
            self.selected_project_list_index = None;
            self.update_selected_project_fields();
            return;
        }

        let last_project_list_index = self.project_entries.len() - 1;
        self.selected_project_list_index = Some(last_project_list_index);
        self.update_selected_project_fields();
    }

    pub fn select_project_by_directory_path(
        &mut self,
        project_directory_path: &Path,
    ) -> bool {
        let matching_project_index = self.project_entries.iter().position(|project_entry| {
            project_entry
                .get_project_directory()
                .as_deref()
                .is_some_and(|entry_directory| entry_directory == project_directory_path)
        });

        if let Some(matching_project_index) = matching_project_index {
            self.selected_project_list_index = Some(matching_project_index);
            self.update_selected_project_fields();
            return true;
        }

        false
    }

    pub fn selected_project_directory_path(&self) -> Option<PathBuf> {
        self.selected_project_directory_path.clone()
    }

    pub fn selected_project_name(&self) -> Option<String> {
        self.selected_project_name.clone()
    }

    pub fn begin_search_input(&mut self) {
        self.input_mode = ProjectSelectorInputMode::Search;
    }

    pub fn commit_search_input(&mut self) {
        self.input_mode = ProjectSelectorInputMode::None;
    }

    pub fn cancel_search_input(&mut self) {
        self.input_mode = ProjectSelectorInputMode::None;
        self.pending_search_name_input.clear();
        self.apply_search_filter_to_projects();
    }

    pub fn append_pending_search_character(
        &mut self,
        pending_character: char,
    ) {
        if !Self::is_supported_search_character(pending_character) {
            return;
        }

        self.pending_search_name_input.push(pending_character);
        self.apply_search_filter_to_projects();
    }

    pub fn backspace_pending_search_name(&mut self) {
        self.pending_search_name_input.pop();
        self.apply_search_filter_to_projects();
    }

    pub fn clear_pending_search_name(&mut self) {
        self.pending_search_name_input.clear();
        self.apply_search_filter_to_projects();
    }

    pub fn pending_search_name_trimmed(&self) -> Option<String> {
        let trimmed_search_name = self.pending_search_name_input.trim();
        if trimmed_search_name.is_empty() {
            None
        } else {
            Some(trimmed_search_name.to_string())
        }
    }

    pub fn begin_create_project_input(&mut self) {
        self.input_mode = ProjectSelectorInputMode::CreatingProject;
        self.pending_project_name_input = "NewProject".to_string();
    }

    pub fn begin_rename_selected_project_input(&mut self) -> bool {
        let Some(selected_project_name) = self.selected_project_name.clone() else {
            return false;
        };

        self.input_mode = ProjectSelectorInputMode::RenamingProject;
        self.pending_project_name_input = selected_project_name;
        true
    }

    pub fn begin_create_project_directory_input(&mut self) -> bool {
        if self.selected_project_item_directory_target_path().is_none() {
            return false;
        }

        self.input_mode = ProjectSelectorInputMode::CreatingProjectDirectory;
        self.pending_project_name_input = self.build_unique_new_directory_name();
        true
    }

    pub fn cancel_project_name_input(&mut self) {
        self.input_mode = ProjectSelectorInputMode::None;
        self.pending_project_name_input.clear();
    }

    pub fn pending_project_name_trimmed(&self) -> Option<String> {
        let trimmed_project_name = self.pending_project_name_input.trim();
        if trimmed_project_name.is_empty() {
            None
        } else {
            Some(trimmed_project_name.to_string())
        }
    }

    pub fn append_pending_project_name_character(
        &mut self,
        pending_character: char,
    ) {
        if !Self::is_supported_project_name_character(pending_character) {
            return;
        }

        self.pending_project_name_input.push(pending_character);
    }

    pub fn backspace_pending_project_name(&mut self) {
        self.pending_project_name_input.pop();
    }

    pub fn clear_pending_project_name(&mut self) {
        self.pending_project_name_input.clear();
    }

    pub fn set_active_project(
        &mut self,
        active_project_name: Option<String>,
        active_project_directory_path: Option<PathBuf>,
    ) {
        self.active_project_name = active_project_name;
        self.active_project_directory_path = active_project_directory_path;
        self.sync_focus_target_to_project_context();
    }

    pub fn summary_lines(&self) -> Vec<String> {
        build_project_explorer_summary_lines(self)
    }

    pub fn visible_project_entry_rows(
        &self,
        viewport_capacity: usize,
    ) -> Vec<PaneEntryRow> {
        build_visible_project_entry_rows(self, viewport_capacity)
    }

    pub fn visible_project_item_entry_rows(
        &self,
        viewport_capacity: usize,
    ) -> Vec<PaneEntryRow> {
        build_visible_project_item_entry_rows(self, viewport_capacity)
    }

    pub fn select_first_project_item(&mut self) {
        if self.project_item_visible_entries.is_empty() {
            self.selected_project_item_visible_index = None;
            self.update_selected_item_path();
            return;
        }

        self.selected_project_item_visible_index = Some(0);
        self.update_selected_item_path();
    }

    pub fn select_last_project_item(&mut self) {
        if self.project_item_visible_entries.is_empty() {
            self.selected_project_item_visible_index = None;
            self.update_selected_item_path();
            return;
        }

        let last_visible_project_item_position = self.project_item_visible_entries.len() - 1;
        self.selected_project_item_visible_index = Some(last_visible_project_item_position);
        self.update_selected_item_path();
    }

    fn update_selected_project_fields(&mut self) {
        if let Some(selected_project_index) = self.selected_project_list_index {
            if let Some(selected_project_entry) = self.project_entries.get(selected_project_index) {
                self.selected_project_name = Some(selected_project_entry.get_name().to_string());
                self.selected_project_directory_path = selected_project_entry.get_project_directory();
                return;
            }
        }

        self.selected_project_name = None;
        self.selected_project_directory_path = None;
    }

    fn rebuild_visible_hierarchy_entries(&mut self) {
        self.project_item_visible_entries = build_visible_hierarchy_entries(
            self.is_hierarchy_expanded,
            &self.opened_project_item_map,
            &self.child_paths_by_parent_path,
            &self.root_project_item_paths,
            &self.expanded_directory_paths,
        );
    }

    fn restore_selected_project_item_path(
        &mut self,
        selected_project_item_path_before_refresh: Option<PathBuf>,
    ) {
        let selected_project_item_visible_index = selected_project_item_path_before_refresh
            .as_ref()
            .and_then(|selected_project_item_path| {
                self.project_item_visible_entries
                    .iter()
                    .position(|project_item_entry| &project_item_entry.project_item_path == selected_project_item_path)
            })
            .or_else(|| if self.project_item_visible_entries.is_empty() { None } else { Some(0) });

        self.selected_project_item_visible_index = selected_project_item_visible_index;
        self.update_selected_item_path();
    }

    fn update_selected_item_path(&mut self) {
        self.selected_item_path = self
            .selected_project_item_path()
            .map(|project_item_path| project_item_path.display().to_string());
    }

    fn is_directory_path(
        &self,
        project_item_path: &Path,
    ) -> bool {
        self.opened_project_item_map
            .get(project_item_path)
            .map(is_directory_project_item)
            .unwrap_or(false)
    }

    fn build_unique_new_directory_name(&self) -> String {
        const BASE_DIRECTORY_NAME: &str = "New Folder";
        let Some(parent_directory_path) = self.selected_project_item_directory_target_path() else {
            return BASE_DIRECTORY_NAME.to_string();
        };

        let existing_child_names: HashSet<String> = self
            .child_paths_by_parent_path
            .get(&parent_directory_path)
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|project_item_path| {
                project_item_path
                    .file_name()
                    .and_then(|file_name| file_name.to_str())
                    .map(str::to_string)
            })
            .collect();

        if !existing_child_names.contains(BASE_DIRECTORY_NAME) {
            return BASE_DIRECTORY_NAME.to_string();
        }

        let mut suffix_number = 2usize;
        loop {
            let candidate_directory_name = format!("{} {}", BASE_DIRECTORY_NAME, suffix_number);
            if !existing_child_names.contains(&candidate_directory_name) {
                return candidate_directory_name;
            }
            suffix_number += 1;
        }
    }

    fn is_supported_project_name_character(pending_character: char) -> bool {
        pending_character.is_ascii_alphanumeric()
            || pending_character == ' '
            || pending_character == '_'
            || pending_character == '-'
            || pending_character == '.'
    }

    fn is_supported_search_character(pending_character: char) -> bool {
        pending_character.is_ascii_graphic() || pending_character == ' '
    }
}

impl Default for ProjectExplorerPaneState {
    fn default() -> Self {
        Self {
            all_project_entries: Vec::new(),
            project_entries: Vec::new(),
            selected_project_list_index: None,
            selected_project_name: None,
            selected_project_directory_path: None,
            active_project_name: None,
            active_project_directory_path: None,
            selected_item_path: None,
            is_hierarchy_expanded: true,
            focus_target: ProjectExplorerFocusTarget::ProjectList,
            input_mode: ProjectSelectorInputMode::None,
            pending_search_name_input: String::new(),
            pending_project_name_input: String::new(),
            has_loaded_project_list_once: false,
            is_awaiting_project_list_response: false,
            is_creating_project: false,
            is_opening_project: false,
            is_renaming_project: false,
            is_deleting_project: false,
            is_closing_project: false,
            has_loaded_project_item_list_once: false,
            is_awaiting_project_item_list_response: false,
            is_creating_project_item: false,
            is_deleting_project_item: false,
            is_moving_project_item: false,
            is_reordering_project_item: false,
            is_toggling_project_item_activation: false,
            project_item_visible_entries: Vec::new(),
            selected_project_item_visible_index: None,
            pending_move_source_paths: Vec::new(),
            pending_delete_confirmation_paths: Vec::new(),
            status_message: "Ready.".to_string(),
            opened_project_item_map: HashMap::new(),
            child_paths_by_parent_path: HashMap::new(),
            root_project_item_paths: Vec::new(),
            expanded_directory_paths: HashSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ProjectExplorerFocusTarget, ProjectExplorerPaneState, ProjectHierarchyEntry};
    use squalr_engine_api::structures::projects::project_info::ProjectInfo;
    use squalr_engine_api::structures::projects::project_manifest::ProjectManifest;
    use std::path::PathBuf;

    #[test]
    fn set_active_project_switches_focus_to_hierarchy() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        project_explorer_pane_state.set_active_project(Some("TestProject".to_string()), Some(PathBuf::from("C:/tmp/test_project")));

        assert_eq!(project_explorer_pane_state.focus_target, ProjectExplorerFocusTarget::ProjectHierarchy);
    }

    #[test]
    fn clearing_active_project_switches_focus_to_project_list() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        project_explorer_pane_state.set_active_project(Some("TestProject".to_string()), Some(PathBuf::from("C:/tmp/test_project")));
        project_explorer_pane_state.set_active_project(None, None);

        assert_eq!(project_explorer_pane_state.focus_target, ProjectExplorerFocusTarget::ProjectList);
    }

    #[test]
    fn search_filter_restricts_project_entries() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        project_explorer_pane_state.apply_project_list(vec![
            ProjectInfo::new(PathBuf::from("C:/projects/AlphaProject/project.squalr"), None, ProjectManifest::default()),
            ProjectInfo::new(PathBuf::from("C:/projects/BetaProject/project.squalr"), None, ProjectManifest::default()),
        ]);

        project_explorer_pane_state.begin_search_input();
        for search_character in "beta".chars() {
            project_explorer_pane_state.append_pending_search_character(search_character);
        }
        project_explorer_pane_state.commit_search_input();

        assert_eq!(project_explorer_pane_state.project_entries.len(), 1);
        assert_eq!(project_explorer_pane_state.project_entries[0].get_name(), "BetaProject");
    }

    #[test]
    fn selecting_last_project_item_uses_end_navigation_behavior() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        project_explorer_pane_state.project_item_visible_entries = vec![
            ProjectHierarchyEntry {
                project_item_path: PathBuf::from("C:/projects/opened/Foo"),
                display_name: "Foo".to_string(),
                depth: 0,
                is_directory: true,
                is_expanded: false,
                is_activated: false,
            },
            ProjectHierarchyEntry {
                project_item_path: PathBuf::from("C:/projects/opened/Bar"),
                display_name: "Bar".to_string(),
                depth: 0,
                is_directory: true,
                is_expanded: false,
                is_activated: false,
            },
        ];

        project_explorer_pane_state.select_last_project_item();

        assert_eq!(project_explorer_pane_state.selected_project_item_visible_index, Some(1));
    }

    #[test]
    fn selecting_first_project_uses_home_navigation_behavior() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        project_explorer_pane_state.apply_project_list(vec![
            ProjectInfo::new(PathBuf::from("C:/projects/AlphaProject/project.squalr"), None, ProjectManifest::default()),
            ProjectInfo::new(PathBuf::from("C:/projects/BetaProject/project.squalr"), None, ProjectManifest::default()),
        ]);
        project_explorer_pane_state.select_next_project();

        project_explorer_pane_state.select_first_project();

        assert_eq!(project_explorer_pane_state.selected_project_list_index, Some(0));
        assert_eq!(project_explorer_pane_state.selected_project_name.as_deref(), Some("AlphaProject"));
    }

    #[test]
    fn selecting_last_project_uses_end_navigation_behavior() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        project_explorer_pane_state.apply_project_list(vec![
            ProjectInfo::new(PathBuf::from("C:/projects/AlphaProject/project.squalr"), None, ProjectManifest::default()),
            ProjectInfo::new(PathBuf::from("C:/projects/BetaProject/project.squalr"), None, ProjectManifest::default()),
        ]);

        project_explorer_pane_state.select_last_project();

        assert_eq!(project_explorer_pane_state.selected_project_list_index, Some(1));
        assert_eq!(project_explorer_pane_state.selected_project_name.as_deref(), Some("BetaProject"));
    }
}
