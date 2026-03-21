use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectHierarchyMenuTarget {
    ToolbarAdd { target_project_item_path: PathBuf },
    ProjectItem(PathBuf),
}
