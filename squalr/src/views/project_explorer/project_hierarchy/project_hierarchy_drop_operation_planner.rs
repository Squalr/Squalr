use crate::views::project_explorer::project_hierarchy::view_data::{
    project_hierarchy_drop_target::ProjectHierarchyDropTarget, project_hierarchy_tree_model::ProjectHierarchyTreeModel,
};
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectHierarchyDropOperation {
    Reorder {
        project_item_paths: Vec<PathBuf>,
    },
    Move {
        project_item_paths: Vec<PathBuf>,
        target_directory_path: PathBuf,
    },
    MoveAndReorder {
        project_item_paths: Vec<PathBuf>,
        target_directory_path: PathBuf,
        reordered_project_item_paths: Vec<PathBuf>,
    },
}

pub struct ProjectHierarchyDropOperationPlanner;

impl ProjectHierarchyDropOperationPlanner {
    pub fn build(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
        dragged_project_item_paths: &[PathBuf],
        drop_target: &ProjectHierarchyDropTarget,
    ) -> Option<ProjectHierarchyDropOperation> {
        if dragged_project_item_paths.is_empty() {
            return None;
        }

        let ProjectHierarchyTreeModel {
            project_item_map,
            mut child_paths_by_parent_path,
            ..
        } = ProjectHierarchyTreeModel::build(opened_project_info, project_items)?;
        let target_project_item_path = drop_target.target_project_item_path();
        let target_is_directory = ProjectHierarchyTreeModel::is_directory_path(target_project_item_path, &project_item_map);
        let target_directory_path = match drop_target {
            ProjectHierarchyDropTarget::Into(_) => {
                if !target_is_directory {
                    return None;
                }

                target_project_item_path.to_path_buf()
            }
            ProjectHierarchyDropTarget::Before(_) | ProjectHierarchyDropTarget::After(_) => target_project_item_path.parent()?.to_path_buf(),
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

        if matches!(drop_target, ProjectHierarchyDropTarget::Into(_)) {
            return Some(ProjectHierarchyDropOperation::Move {
                project_item_paths: dragged_project_item_paths.to_vec(),
                target_directory_path,
            });
        }

        let dragged_paths_in_target_sibling_order: Vec<PathBuf> = child_paths_by_parent_path
            .get(&target_directory_path)?
            .iter()
            .filter(|sibling_project_item_path| dragged_project_item_path_set.contains(*sibling_project_item_path))
            .cloned()
            .collect();
        let sibling_paths = child_paths_by_parent_path.get_mut(&target_directory_path)?;
        sibling_paths.retain(|sibling_project_item_path| !dragged_project_item_path_set.contains(sibling_project_item_path));
        let target_sibling_index = sibling_paths
            .iter()
            .position(|project_item_path| project_item_path == target_project_item_path)?;
        let insert_sibling_index = match drop_target {
            ProjectHierarchyDropTarget::Before(_) => target_sibling_index,
            ProjectHierarchyDropTarget::After(_) => target_sibling_index.saturating_add(1),
            ProjectHierarchyDropTarget::Into(_) => return None,
        };

        if !all_dragged_items_share_target_parent {
            let projected_dragged_project_item_paths: Vec<PathBuf> = dragged_project_item_paths
                .iter()
                .map(|dragged_project_item_path| target_directory_path.join(dragged_project_item_path.file_name().unwrap_or_default()))
                .collect();

            for (dragged_path_insert_index, projected_dragged_project_item_path) in projected_dragged_project_item_paths.iter().cloned().enumerate() {
                sibling_paths.insert(insert_sibling_index + dragged_path_insert_index, projected_dragged_project_item_path);
            }

            return Some(ProjectHierarchyDropOperation::MoveAndReorder {
                project_item_paths: dragged_project_item_paths.to_vec(),
                target_directory_path,
                reordered_project_item_paths: sibling_paths.clone(),
            });
        }

        if dragged_paths_in_target_sibling_order.len() != dragged_project_item_path_set.len() {
            return None;
        }

        for (dragged_path_insert_index, dragged_project_item_path) in dragged_paths_in_target_sibling_order.into_iter().enumerate() {
            sibling_paths.insert(insert_sibling_index + dragged_path_insert_index, dragged_project_item_path);
        }

        Some(ProjectHierarchyDropOperation::Reorder {
            project_item_paths: sibling_paths.clone(),
        })
    }
}
