use crate::views::project_explorer::project_hierarchy::view_data::project_hierarchy_create_item_kind::ProjectHierarchyCreateItemKind;
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
    CreateProjectItem {
        target_project_item_path: PathBuf,
        create_item_kind: ProjectHierarchyCreateItemKind,
    },
    OpenPointerScannerForAddress {
        address: u64,
        module_name: String,
        data_type_id: String,
    },
    OpenMemoryViewerForAddress {
        address: u64,
        module_name: String,
    },
    RequestRename(PathBuf),
    RequestValueEdit(PathBuf),
    RequestDeleteConfirmation(Vec<PathBuf>),
}
