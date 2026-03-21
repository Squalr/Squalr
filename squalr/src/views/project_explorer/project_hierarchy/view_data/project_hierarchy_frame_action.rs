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
    OpenPointerScannerForAddress {
        address: u64,
        module_name: String,
        data_type_id: String,
    },
    RequestDeleteConfirmation(Vec<PathBuf>),
}
