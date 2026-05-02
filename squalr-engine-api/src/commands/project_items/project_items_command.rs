use crate::commands::project_items::{
    activate::project_items_activate_request::ProjectItemsActivateRequest, add::project_items_add_request::ProjectItemsAddRequest,
    convert_symbol_ref::project_items_convert_symbol_ref_request::ProjectItemsConvertSymbolRefRequest,
    create::project_items_create_request::ProjectItemsCreateRequest, delete::project_items_delete_request::ProjectItemsDeleteRequest,
    duplicate::project_items_duplicate_request::ProjectItemsDuplicateRequest, list::project_items_list_request::ProjectItemsListRequest,
    move_item::project_items_move_request::ProjectItemsMoveRequest, promote_symbol::project_items_promote_symbol_request::ProjectItemsPromoteSymbolRequest,
    rename::project_items_rename_request::ProjectItemsRenameRequest, reorder::project_items_reorder_request::ProjectItemsReorderRequest,
};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProjectItemsCommand {
    /// Adds project items from the provided scan results.
    Add {
        #[structopt(flatten)]
        project_items_add_request: ProjectItemsAddRequest,
    },
    /// Activates project items.
    Activate {
        #[structopt(flatten)]
        project_items_activate_request: ProjectItemsActivateRequest,
    },
    /// Creates a project item.
    Create {
        #[structopt(flatten)]
        project_items_create_request: ProjectItemsCreateRequest,
    },
    /// Converts symbol-ref project items back to concrete item types.
    ConvertSymbolRef {
        #[structopt(flatten)]
        project_items_convert_symbol_ref_request: ProjectItemsConvertSymbolRefRequest,
    },
    /// Deletes project items.
    Delete {
        #[structopt(flatten)]
        project_items_delete_request: ProjectItemsDeleteRequest,
    },
    /// Duplicates project items into a target directory.
    Duplicate {
        #[structopt(flatten)]
        project_items_duplicate_request: ProjectItemsDuplicateRequest,
    },
    /// Lists opened project items.
    List {
        #[structopt(flatten)]
        project_items_list_request: ProjectItemsListRequest,
    },
    /// Moves project items.
    Move {
        #[structopt(flatten)]
        project_items_move_request: ProjectItemsMoveRequest,
    },
    /// Promotes project items into symbol claims.
    PromoteSymbol {
        #[structopt(flatten)]
        project_items_promote_symbol_request: ProjectItemsPromoteSymbolRequest,
    },
    /// Renames a project item.
    Rename {
        #[structopt(flatten)]
        project_items_rename_request: ProjectItemsRenameRequest,
    },
    /// Reorders project items for persisted hierarchy display.
    Reorder {
        #[structopt(flatten)]
        project_items_reorder_request: ProjectItemsReorderRequest,
    },
}
