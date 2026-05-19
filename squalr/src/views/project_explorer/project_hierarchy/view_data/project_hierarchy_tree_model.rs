use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct ProjectHierarchyTreeModel {
    pub root_directory_path: PathBuf,
    pub project_item_map: HashMap<PathBuf, (ProjectItemRef, ProjectItem)>,
    pub child_paths_by_parent_path: HashMap<PathBuf, Vec<PathBuf>>,
}

impl ProjectHierarchyTreeModel {
    pub fn build(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
    ) -> Option<Self> {
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

        Some(Self {
            root_directory_path: project_root_directory_path,
            project_item_map,
            child_paths_by_parent_path,
        })
    }

    pub fn resolve_project_root_path(
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

    pub fn is_directory_path(
        project_item_path: &Path,
        project_item_map: &HashMap<PathBuf, (ProjectItemRef, ProjectItem)>,
    ) -> bool {
        project_item_map
            .get(project_item_path)
            .map(|(_, project_item)| Self::is_directory_project_item(project_item))
            .unwrap_or(false)
    }

    pub fn is_directory_project_item(project_item: &ProjectItem) -> bool {
        project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID
    }

    pub fn build_reorder_paths_after_target(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
        target_project_item_path: &Path,
        project_item_paths_to_insert: &[PathBuf],
        project_item_paths_to_remove: &[PathBuf],
    ) -> Option<Vec<PathBuf>> {
        if project_item_paths_to_insert.is_empty() {
            return None;
        }

        let ProjectHierarchyTreeModel {
            mut child_paths_by_parent_path,
            ..
        } = ProjectHierarchyTreeModel::build(opened_project_info, project_items)?;
        let target_directory_path = target_project_item_path.parent()?.to_path_buf();
        let sibling_paths = child_paths_by_parent_path.get_mut(&target_directory_path)?;
        let project_item_paths_to_remove: HashSet<&PathBuf> = project_item_paths_to_remove.iter().collect();

        sibling_paths.retain(|sibling_project_item_path| !project_item_paths_to_remove.contains(sibling_project_item_path));

        let target_sibling_index = sibling_paths
            .iter()
            .position(|project_item_path| project_item_path == target_project_item_path)?;
        let insert_sibling_index = target_sibling_index.saturating_add(1);

        for (inserted_project_item_index, inserted_project_item_path) in project_item_paths_to_insert.iter().cloned().enumerate() {
            sibling_paths.insert(insert_sibling_index + inserted_project_item_index, inserted_project_item_path);
        }

        Some(sibling_paths.clone())
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
}
