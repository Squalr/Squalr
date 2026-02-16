use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum ProjectHierarchyFrameAction {
    None,
    SelectProjectItem {
        project_item_path: PathBuf,
        additive_selection: bool,
        range_selection: bool,
    },
    ToggleDirectoryExpansion(PathBuf),
    SetProjectItemActivation(PathBuf, bool),
    CreateDirectory(PathBuf),
    RequestDeleteConfirmation(Vec<PathBuf>),
}
