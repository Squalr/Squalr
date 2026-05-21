use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectHierarchyMenuTarget {
    EmptySpace { target_project_item_path: PathBuf },
    ToolbarAdd { target_project_item_path: PathBuf },
    ProjectItem(PathBuf),
}
