use crate::views::project_explorer::project_hierarchy::project_item_create_request_builder::ProjectItemCreateRequestBuilder;
use crate::views::project_explorer::project_hierarchy::view_data::{
    project_hierarchy_clipboard::{ProjectHierarchyClipboard, ProjectHierarchyClipboardMode},
    project_hierarchy_tree_entry::ProjectHierarchyTreeEntry,
    project_hierarchy_tree_model::ProjectHierarchyTreeModel,
};
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectHierarchyPasteTarget {
    pub target_directory_path: PathBuf,
    pub insert_after_project_item_path: Option<PathBuf>,
}

pub struct ProjectHierarchyClipboardController;

impl ProjectHierarchyClipboardController {
    pub fn filter_copy_paths(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
        tree_entries: &[ProjectHierarchyTreeEntry],
        project_item_paths: Vec<PathBuf>,
    ) -> Vec<PathBuf> {
        let copyable_project_item_paths = project_item_paths
            .into_iter()
            .filter(|project_item_path| !Self::is_protected_project_item_path(opened_project_info, project_items, project_item_path))
            .collect::<Vec<PathBuf>>();

        Self::reduce_paths_to_root_set(tree_entries, &copyable_project_item_paths)
    }

    pub fn filter_cut_paths(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
        tree_entries: &[ProjectHierarchyTreeEntry],
        project_item_paths: Vec<PathBuf>,
    ) -> Vec<PathBuf> {
        let cuttable_project_item_paths = project_item_paths
            .into_iter()
            .filter(|project_item_path| !Self::is_protected_project_item_path(opened_project_info, project_items, project_item_path))
            .collect::<Vec<PathBuf>>();

        Self::reduce_paths_to_root_set(tree_entries, &cuttable_project_item_paths)
    }

    pub fn resolve_paste_target(
        project_items: &[(ProjectItemRef, ProjectItem)],
        target_project_item_path: &Path,
    ) -> ProjectHierarchyPasteTarget {
        let target_directory_path = ProjectItemCreateRequestBuilder::resolve_parent_directory_path(project_items, target_project_item_path);
        let insert_after_project_item_path = if Self::is_directory_project_item_path(project_items, target_project_item_path) {
            None
        } else {
            Some(target_project_item_path.to_path_buf())
        };

        ProjectHierarchyPasteTarget {
            target_directory_path,
            insert_after_project_item_path,
        }
    }

    pub fn can_paste(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
        clipboard: &ProjectHierarchyClipboard,
        target_project_item_path: &Path,
    ) -> bool {
        let Some(current_project_file_path) = opened_project_info.map(|opened_project_info| opened_project_info.get_project_file_path().clone()) else {
            return false;
        };
        let paste_target = Self::resolve_paste_target(project_items, target_project_item_path);

        if clipboard.get_project_file_path() != Some(&current_project_file_path) {
            return false;
        }

        !Self::filter_pasteable_paths(
            opened_project_info,
            project_items,
            clipboard.get_project_item_paths(),
            &paste_target,
            clipboard.get_mode(),
        )
        .is_empty()
    }

    pub fn filter_pasteable_paths(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
        project_item_paths: &[PathBuf],
        paste_target: &ProjectHierarchyPasteTarget,
        clipboard_mode: Option<&ProjectHierarchyClipboardMode>,
    ) -> Vec<PathBuf> {
        project_item_paths
            .iter()
            .filter(|project_item_path| !Self::is_protected_project_item_path(opened_project_info, project_items, project_item_path))
            .filter(|project_item_path| match clipboard_mode {
                Some(ProjectHierarchyClipboardMode::Copy) => {
                    !Self::is_directory_project_item_path(project_items, project_item_path)
                        || (!paste_target
                            .target_directory_path
                            .starts_with(project_item_path.as_path())
                            && !paste_target
                                .insert_after_project_item_path
                                .as_ref()
                                .map(|insert_after_project_item_path| insert_after_project_item_path.starts_with(project_item_path))
                                .unwrap_or(false))
                }
                Some(ProjectHierarchyClipboardMode::Cut) => {
                    if paste_target
                        .target_directory_path
                        .starts_with(project_item_path.as_path())
                    {
                        return false;
                    }

                    match &paste_target.insert_after_project_item_path {
                        Some(insert_after_project_item_path) => {
                            *project_item_path != insert_after_project_item_path && !insert_after_project_item_path.starts_with(project_item_path)
                        }
                        None => project_item_path.parent() != Some(paste_target.target_directory_path.as_path()),
                    }
                }
                None => false,
            })
            .cloned()
            .collect()
    }

    pub fn is_cut_project_item_path(
        clipboard: &ProjectHierarchyClipboard,
        project_item_path: &Path,
    ) -> bool {
        if !clipboard.is_cut() {
            return false;
        }

        clipboard
            .get_project_item_paths()
            .iter()
            .any(|cut_project_item_path| project_item_path == cut_project_item_path || project_item_path.starts_with(cut_project_item_path))
    }

    fn reduce_paths_to_root_set(
        tree_entries: &[ProjectHierarchyTreeEntry],
        project_item_paths: &[PathBuf],
    ) -> Vec<PathBuf> {
        let selected_project_item_path_set: HashSet<&PathBuf> = project_item_paths.iter().collect();

        tree_entries
            .iter()
            .map(|tree_entry| tree_entry.project_item_path.clone())
            .filter(|project_item_path| selected_project_item_path_set.contains(project_item_path))
            .filter(|project_item_path| {
                !project_item_paths
                    .iter()
                    .any(|candidate_root_path| candidate_root_path != project_item_path && project_item_path.starts_with(candidate_root_path))
            })
            .collect()
    }

    fn is_protected_project_item_path(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
        project_item_path: &Path,
    ) -> bool {
        ProjectHierarchyTreeModel::resolve_project_root_path(opened_project_info, project_items)
            .as_ref()
            .map(|root_project_item_path| root_project_item_path == project_item_path)
            .unwrap_or(false)
    }

    fn is_directory_project_item_path(
        project_items: &[(ProjectItemRef, ProjectItem)],
        project_item_path: &Path,
    ) -> bool {
        project_items
            .iter()
            .find(|(project_item_ref, _)| project_item_ref.get_project_item_path() == project_item_path)
            .map(|(_, project_item)| ProjectHierarchyTreeModel::is_directory_project_item(project_item))
            .unwrap_or(false)
    }
}
