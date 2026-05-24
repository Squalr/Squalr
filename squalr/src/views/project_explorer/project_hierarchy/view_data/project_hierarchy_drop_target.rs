use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectHierarchyDropTarget {
    Into(PathBuf),
    Before(PathBuf),
    After(PathBuf),
}

impl ProjectHierarchyDropTarget {
    pub fn target_project_item_path(&self) -> &Path {
        match self {
            Self::Into(project_item_path) | Self::Before(project_item_path) | Self::After(project_item_path) => project_item_path.as_path(),
        }
    }
}
