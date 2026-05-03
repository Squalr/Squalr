use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_response::ProjectItemsPromoteSymbolConflict;
use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum ProjectHierarchyTakeOverState {
    None,
    DeleteConfirmation {
        project_item_paths: Vec<PathBuf>,
    },
    PromoteSymbolConflict {
        project_item_paths: Vec<PathBuf>,
        conflicts: Vec<ProjectItemsPromoteSymbolConflict>,
    },
    RenameProjectItem {
        project_item_path: PathBuf,
        project_item_type_id: String,
    },
    EditProjectItemValue {
        project_item_path: PathBuf,
    },
    EditPointerOffsets {
        project_item_path: PathBuf,
    },
}
