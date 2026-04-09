use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum ProjectHierarchyTakeOverState {
    None,
    DeleteConfirmation { project_item_paths: Vec<PathBuf> },
    RenameProjectItem { project_item_path: PathBuf, project_item_type_id: String },
    EditProjectItemValue { project_item_path: PathBuf },
}
