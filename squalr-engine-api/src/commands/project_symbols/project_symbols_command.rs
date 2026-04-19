use crate::commands::project_symbols::{
    create::project_symbols_create_request::ProjectSymbolsCreateRequest, delete::project_symbols_delete_request::ProjectSymbolsDeleteRequest,
    list::project_symbols_list_request::ProjectSymbolsListRequest, rename::project_symbols_rename_request::ProjectSymbolsRenameRequest,
    update::project_symbols_update_request::ProjectSymbolsUpdateRequest,
};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProjectSymbolsCommand {
    /// Creates a rooted project symbol.
    Create {
        #[structopt(flatten)]
        project_symbols_create_request: ProjectSymbolsCreateRequest,
    },
    /// Deletes rooted project symbols.
    Delete {
        #[structopt(flatten)]
        project_symbols_delete_request: ProjectSymbolsDeleteRequest,
    },
    /// Lists the current project symbol store.
    List {
        #[structopt(flatten)]
        project_symbols_list_request: ProjectSymbolsListRequest,
    },
    /// Renames a rooted project symbol display name.
    Rename {
        #[structopt(flatten)]
        project_symbols_rename_request: ProjectSymbolsRenameRequest,
    },
    /// Updates rooted project symbol properties.
    Update {
        #[structopt(flatten)]
        project_symbols_update_request: ProjectSymbolsUpdateRequest,
    },
}
