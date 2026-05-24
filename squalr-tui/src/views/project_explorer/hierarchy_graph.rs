use squalr_engine_api::structures::projects::project_info::ProjectInfo;
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
pub fn build_project_item_hierarchy_graph(
    opened_project_info: Option<&ProjectInfo>,
    opened_project_items: Vec<(ProjectItemRef, ProjectItem)>,
) -> ProjectItemHierarchyGraph {
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

    let sort_order_lookup = build_sort_order_lookup(opened_project_info);
    root_project_item_paths.sort_by(|left_path, right_path| compare_paths(left_path, right_path, &sort_order_lookup));
    for child_paths in child_paths_by_parent_path.values_mut() {
        child_paths.sort_by(|left_path, right_path| compare_paths(left_path, right_path, &sort_order_lookup));
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

fn build_sort_order_lookup(opened_project_info: Option<&ProjectInfo>) -> HashMap<PathBuf, usize> {
    let Some(opened_project_info) = opened_project_info else {
        return HashMap::new();
    };
    let Some(project_directory_path) = opened_project_info.get_project_directory() else {
        return HashMap::new();
    };

    opened_project_info
        .get_project_manifest()
        .get_project_item_sort_order()
        .iter()
        .enumerate()
        .map(|(sort_order_position, relative_project_item_path)| (project_directory_path.join(relative_project_item_path), sort_order_position))
        .collect()
}

fn compare_paths(
    left_path: &PathBuf,
    right_path: &PathBuf,
    sort_order_lookup: &HashMap<PathBuf, usize>,
) -> std::cmp::Ordering {
    let left_sort_order_position = sort_order_lookup.get(left_path).copied().unwrap_or(usize::MAX);
    let right_sort_order_position = sort_order_lookup.get(right_path).copied().unwrap_or(usize::MAX);

    if left_sort_order_position != right_sort_order_position {
        return left_sort_order_position.cmp(&right_sort_order_position);
    }

    left_path.cmp(right_path)
}

/// Returns true when a project item is a directory node.
pub fn is_directory_project_item(project_item: &ProjectItem) -> bool {
    project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID
}
