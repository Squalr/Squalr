use crate::commands::project_items::{
    activate::project_items_activate_response::ProjectItemsActivateResponse, add::project_items_add_response::ProjectItemsAddResponse,
    convert_symbol_ref::project_items_convert_symbol_ref_response::ProjectItemsConvertSymbolRefResponse,
    create::project_items_create_response::ProjectItemsCreateResponse, delete::project_items_delete_response::ProjectItemsDeleteResponse,
    list::project_items_list_response::ProjectItemsListResponse, move_item::project_items_move_response::ProjectItemsMoveResponse,
    promote_symbol::project_items_promote_symbol_response::ProjectItemsPromoteSymbolResponse,
    rename::project_items_rename_response::ProjectItemsRenameResponse, reorder::project_items_reorder_response::ProjectItemsReorderResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectItemsResponse {
    Add {
        project_items_add_response: ProjectItemsAddResponse,
    },
    Activate {
        project_items_activate_response: ProjectItemsActivateResponse,
    },
    Create {
        project_items_create_response: ProjectItemsCreateResponse,
    },
    ConvertSymbolRef {
        project_items_convert_symbol_ref_response: ProjectItemsConvertSymbolRefResponse,
    },
    Delete {
        project_items_delete_response: ProjectItemsDeleteResponse,
    },
    List {
        project_items_list_response: ProjectItemsListResponse,
    },
    Move {
        project_items_move_response: ProjectItemsMoveResponse,
    },
    PromoteSymbol {
        project_items_promote_symbol_response: ProjectItemsPromoteSymbolResponse,
    },
    Rename {
        project_items_rename_response: ProjectItemsRenameResponse,
    },
    Reorder {
        project_items_reorder_response: ProjectItemsReorderResponse,
    },
}
