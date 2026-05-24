use crate::commands::project_items::{
    activate::project_items_activate_response::ProjectItemsActivateResponse, add::project_items_add_response::ProjectItemsAddResponse,
    create::project_items_create_response::ProjectItemsCreateResponse, delete::project_items_delete_response::ProjectItemsDeleteResponse,
    duplicate::project_items_duplicate_response::ProjectItemsDuplicateResponse, list::project_items_list_response::ProjectItemsListResponse,
    move_item::project_items_move_response::ProjectItemsMoveResponse, promote_symbol::project_items_promote_symbol_response::ProjectItemsPromoteSymbolResponse,
    rename::project_items_rename_response::ProjectItemsRenameResponse, reorder::project_items_reorder_response::ProjectItemsReorderResponse,
    strip_symbol::project_items_strip_symbol_response::ProjectItemsStripSymbolResponse,
    update_details::project_items_update_details_response::ProjectItemsUpdateDetailsResponse,
    write_value::project_items_write_value_response::ProjectItemsWriteValueResponse,
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
    Delete {
        project_items_delete_response: ProjectItemsDeleteResponse,
    },
    Duplicate {
        project_items_duplicate_response: ProjectItemsDuplicateResponse,
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
    StripSymbol {
        project_items_strip_symbol_response: ProjectItemsStripSymbolResponse,
    },
    UpdateDetails {
        project_items_update_details_response: ProjectItemsUpdateDetailsResponse,
    },
    WriteValue {
        project_items_write_value_response: ProjectItemsWriteValueResponse,
    },
}
