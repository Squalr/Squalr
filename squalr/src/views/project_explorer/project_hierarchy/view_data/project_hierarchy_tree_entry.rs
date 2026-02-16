use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use std::path::PathBuf;

#[derive(Clone)]
pub struct ProjectHierarchyTreeEntry {
    pub project_item_ref: ProjectItemRef,
    pub project_item: ProjectItem,
    pub project_item_path: PathBuf,
    pub display_name: String,
    pub preview_value: String,
    pub is_activated: bool,
    pub depth: usize,
    pub is_directory: bool,
    pub has_children: bool,
    pub is_expanded: bool,
}
