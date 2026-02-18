use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Flattens a project-item hierarchy into preorder path traversal order.
pub fn build_project_item_paths_preorder(
    root_project_item_paths: &[PathBuf],
    child_paths_by_parent_path: &HashMap<PathBuf, Vec<PathBuf>>,
) -> Vec<PathBuf> {
    let mut reordered_project_item_paths = Vec::new();
    for root_project_item_path in root_project_item_paths {
        append_project_item_paths_preorder(root_project_item_path, child_paths_by_parent_path, &mut reordered_project_item_paths);
    }

    reordered_project_item_paths
}

fn append_project_item_paths_preorder(
    project_item_path: &Path,
    child_paths_by_parent_path: &HashMap<PathBuf, Vec<PathBuf>>,
    reordered_project_item_paths: &mut Vec<PathBuf>,
) {
    reordered_project_item_paths.push(project_item_path.to_path_buf());
    if let Some(child_paths) = child_paths_by_parent_path.get(project_item_path) {
        for child_path in child_paths {
            append_project_item_paths_preorder(child_path, child_paths_by_parent_path, reordered_project_item_paths);
        }
    }
}
