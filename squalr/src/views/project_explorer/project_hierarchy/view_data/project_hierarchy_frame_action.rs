use crate::views::project_explorer::project_hierarchy::view_data::project_hierarchy_create_item_kind::ProjectHierarchyCreateItemKind;
use squalr_engine_api::commands::project_items::convert_symbol_ref::project_items_convert_symbol_ref_request::ProjectItemSymbolRefConversionTarget;
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
    CopyProjectItems(Vec<PathBuf>),
    CutProjectItems(Vec<PathBuf>),
    PasteProjectItems {
        target_project_item_path: PathBuf,
    },
    OpenPointerScannerForAddress {
        address: u64,
        module_name: String,
        data_type_id: String,
    },
    OpenMemoryViewerForAddress {
        address: u64,
        module_name: String,
        selection_byte_count: u64,
    },
    OpenCodeViewerForAddress {
        address: u64,
        module_name: String,
    },
    PromoteToSymbol {
        project_item_paths: Vec<PathBuf>,
        overwrite_conflicting_symbols: bool,
    },
    ConvertSymbolRef {
        project_item_paths: Vec<PathBuf>,
        conversion_target: ProjectItemSymbolRefConversionTarget,
    },
    RequestRename(PathBuf),
    RequestValueEdit(PathBuf),
    RequestDeleteConfirmation(Vec<PathBuf>),
}
