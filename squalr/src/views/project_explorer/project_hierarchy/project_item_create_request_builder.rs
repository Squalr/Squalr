use crate::views::project_explorer::project_hierarchy::view_data::{
    project_hierarchy_create_item_kind::ProjectHierarchyCreateItemKind, project_hierarchy_tree_model::ProjectHierarchyTreeModel,
};
use squalr_engine_api::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct ProjectItemCreateRequestBuilder;

impl ProjectItemCreateRequestBuilder {
    pub fn build(
        project_items: &[(ProjectItemRef, ProjectItem)],
        target_project_item_path: &Path,
        create_item_kind: ProjectHierarchyCreateItemKind,
    ) -> ProjectItemsCreateRequest {
        let parent_directory_path = Self::resolve_parent_directory_path(project_items, target_project_item_path);

        match create_item_kind {
            ProjectHierarchyCreateItemKind::Directory => ProjectItemsCreateRequest {
                parent_directory_path: parent_directory_path.clone(),
                project_item_name: Self::build_unique_directory_name(project_items, &parent_directory_path),
                is_directory: true,
                address: None,
                module_name: None,
                data_type_id: None,
                pointer_offsets: None,
            },
            ProjectHierarchyCreateItemKind::Address => ProjectItemsCreateRequest {
                parent_directory_path,
                project_item_name: ProjectItemTypeAddress::DEFAULT_PROJECT_ITEM_NAME.to_string(),
                is_directory: false,
                address: Some(0),
                module_name: Some(String::new()),
                data_type_id: None,
                pointer_offsets: None,
            },
        }
    }

    pub fn resolve_parent_directory_path(
        project_items: &[(ProjectItemRef, ProjectItem)],
        target_project_item_path: &Path,
    ) -> PathBuf {
        let is_target_directory = project_items
            .iter()
            .find(|(project_item_ref, _)| project_item_ref.get_project_item_path() == target_project_item_path)
            .map(|(_, project_item)| ProjectHierarchyTreeModel::is_directory_project_item(project_item))
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
}
