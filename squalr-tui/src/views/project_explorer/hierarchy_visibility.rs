use crate::views::project_explorer::hierarchy_graph::is_directory_project_item;
use crate::views::project_explorer::pane_state::ProjectHierarchyEntry;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Builds currently visible hierarchy entries from the expanded-directory state.
pub fn build_visible_hierarchy_entries(
    is_hierarchy_expanded: bool,
    opened_project_item_map: &HashMap<PathBuf, ProjectItem>,
    child_paths_by_parent_path: &HashMap<PathBuf, Vec<PathBuf>>,
    root_project_item_paths: &[PathBuf],
    expanded_directory_paths: &HashSet<PathBuf>,
) -> Vec<ProjectHierarchyEntry> {
    if !is_hierarchy_expanded {
        return Vec::new();
    }

    let mut visible_entries = Vec::new();
    for root_project_item_path in root_project_item_paths {
        append_visible_hierarchy_entries(
            root_project_item_path,
            0,
            opened_project_item_map,
            child_paths_by_parent_path,
            expanded_directory_paths,
            &mut visible_entries,
        );
    }

    visible_entries
}

fn append_visible_hierarchy_entries(
    project_item_path: &Path,
    project_item_depth: usize,
    opened_project_item_map: &HashMap<PathBuf, ProjectItem>,
    child_paths_by_parent_path: &HashMap<PathBuf, Vec<PathBuf>>,
    expanded_directory_paths: &HashSet<PathBuf>,
    visible_entries: &mut Vec<ProjectHierarchyEntry>,
) {
    let Some(project_item) = opened_project_item_map.get(project_item_path) else {
        return;
    };

    let is_directory = is_directory_project_item(project_item);
    let is_expanded = is_directory && expanded_directory_paths.contains(project_item_path);
    let child_paths = child_paths_by_parent_path
        .get(project_item_path)
        .cloned()
        .unwrap_or_default();

    let mut display_name = project_item.get_field_name();
    if display_name.is_empty() {
        display_name = project_item_path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or_default()
            .to_string();
    }

    visible_entries.push(ProjectHierarchyEntry {
        project_item_path: project_item_path.to_path_buf(),
        display_name,
        depth: project_item_depth,
        is_directory,
        is_expanded,
        is_activated: project_item.get_is_activated(),
    });

    if !is_expanded {
        return;
    }

    for child_path in &child_paths {
        append_visible_hierarchy_entries(
            child_path,
            project_item_depth + 1,
            opened_project_item_map,
            child_paths_by_parent_path,
            expanded_directory_paths,
            visible_entries,
        );
    }
}
