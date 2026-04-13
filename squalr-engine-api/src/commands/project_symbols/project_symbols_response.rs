use crate::commands::project_symbols::{
    create::project_symbols_create_response::ProjectSymbolsCreateResponse, delete::project_symbols_delete_response::ProjectSymbolsDeleteResponse,
    list::project_symbols_list_response::ProjectSymbolsListResponse, rename::project_symbols_rename_response::ProjectSymbolsRenameResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectSymbolsResponse {
    Create {
        project_symbols_create_response: ProjectSymbolsCreateResponse,
    },
    Delete {
        project_symbols_delete_response: ProjectSymbolsDeleteResponse,
    },
    List {
        project_symbols_list_response: ProjectSymbolsListResponse,
    },
    Rename {
        project_symbols_rename_response: ProjectSymbolsRenameResponse,
    },
}
