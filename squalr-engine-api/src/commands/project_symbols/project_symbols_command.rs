use crate::commands::project_symbols::{
    create::project_symbols_create_request::ProjectSymbolsCreateRequest,
    create_module::project_symbols_create_module_request::ProjectSymbolsCreateModuleRequest,
    delete::project_symbols_delete_request::ProjectSymbolsDeleteRequest, list::project_symbols_list_request::ProjectSymbolsListRequest,
    rename::project_symbols_rename_request::ProjectSymbolsRenameRequest,
    rename_module::project_symbols_rename_module_request::ProjectSymbolsRenameModuleRequest,
    update::project_symbols_update_request::ProjectSymbolsUpdateRequest,
};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProjectSymbolsCommand {
    /// Creates a project symbol claim.
    Create {
        #[structopt(flatten)]
        project_symbols_create_request: ProjectSymbolsCreateRequest,
    },
    /// Creates or updates a Symbol Tree module root.
    CreateModule {
        #[structopt(flatten)]
        project_symbols_create_module_request: ProjectSymbolsCreateModuleRequest,
    },
    /// Deletes project symbol claims or module roots.
    Delete {
        #[structopt(flatten)]
        project_symbols_delete_request: ProjectSymbolsDeleteRequest,
    },
    /// Lists the current project symbol store.
    List {
        #[structopt(flatten)]
        project_symbols_list_request: ProjectSymbolsListRequest,
    },
    /// Renames a project symbol claim display name.
    Rename {
        #[structopt(flatten)]
        project_symbols_rename_request: ProjectSymbolsRenameRequest,
    },
    /// Renames a Symbol Tree module root and its module-relative claims.
    RenameModule {
        #[structopt(flatten)]
        project_symbols_rename_module_request: ProjectSymbolsRenameModuleRequest,
    },
    /// Updates project symbol claim properties.
    Update {
        #[structopt(flatten)]
        project_symbols_update_request: ProjectSymbolsUpdateRequest,
    },
}
