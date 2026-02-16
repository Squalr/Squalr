use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum ProjectHierarchyTakeOverState {
    None,
    DeleteConfirmation { project_item_paths: Vec<PathBuf> },
}
