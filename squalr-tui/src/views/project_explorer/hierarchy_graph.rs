use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Stores project-item hierarchy graph data derived from a project-items list response.
#[derive(Clone, Debug, Default)]
pub struct ProjectItemHierarchyGraph {
    pub opened_project_item_map: HashMap<PathBuf, ProjectItem>,
    pub child_paths_by_parent_path: HashMap<PathBuf, Vec<PathBuf>>,
    pub root_project_item_paths: Vec<PathBuf>,
    pub valid_project_item_paths: HashSet<PathBuf>,
    pub valid_directory_paths: HashSet<PathBuf>,
}

/// Builds hierarchy graph maps for project-item reducers.
pub fn build_project_item_hierarchy_graph(opened_project_items: Vec<(ProjectItemRef, ProjectItem)>) -> ProjectItemHierarchyGraph {
    let opened_project_item_map: HashMap<PathBuf, ProjectItem> = opened_project_items
        .into_iter()
        .map(|(project_item_ref, project_item)| (project_item_ref.get_project_item_path().clone(), project_item))
        .collect();

    let valid_project_item_paths: HashSet<PathBuf> = opened_project_item_map.keys().cloned().collect();
    let mut child_paths_by_parent_path: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut root_project_item_paths: Vec<PathBuf> = Vec::new();

    for project_item_path in &valid_project_item_paths {
        let parent_directory_path = project_item_path.parent().map(Path::to_path_buf);
        if parent_directory_path
            .as_ref()
            .is_some_and(|candidate_parent_path| valid_project_item_paths.contains(candidate_parent_path))
        {
            if let Some(parent_directory_path) = parent_directory_path {
                child_paths_by_parent_path
                    .entry(parent_directory_path)
                    .or_default()
                    .push(project_item_path.clone());
            }
        } else {
            root_project_item_paths.push(project_item_path.clone());
        }
    }

    root_project_item_paths.sort();
    for child_paths in child_paths_by_parent_path.values_mut() {
        child_paths.sort();
    }

    let valid_directory_paths: HashSet<PathBuf> = opened_project_item_map
        .iter()
        .filter_map(|(project_item_path, project_item)| {
            if is_directory_project_item(project_item) {
                Some(project_item_path.clone())
            } else {
                None
            }
        })
        .collect();

    ProjectItemHierarchyGraph {
        opened_project_item_map,
        child_paths_by_parent_path,
        root_project_item_paths,
        valid_project_item_paths,
        valid_directory_paths,
    }
}

/// Returns true when a project item is a directory node.
pub fn is_directory_project_item(project_item: &ProjectItem) -> bool {
    project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID
}
