use crate::commands::project_symbols::{
    create::project_symbols_create_request::ProjectSymbolsCreateRequest,
    create_module::project_symbols_create_module_request::ProjectSymbolsCreateModuleRequest,
    delete::project_symbols_delete_request::ProjectSymbolsDeleteRequest,
    execute_plugin_action::project_symbols_execute_plugin_action_request::ProjectSymbolsExecutePluginActionRequest,
    list::project_symbols_list_request::ProjectSymbolsListRequest, rename::project_symbols_rename_request::ProjectSymbolsRenameRequest,
    rename_module::project_symbols_rename_module_request::ProjectSymbolsRenameModuleRequest,
    set_catalog::project_symbols_set_catalog_request::ProjectSymbolsSetCatalogRequest, update::project_symbols_update_request::ProjectSymbolsUpdateRequest,
    write_value::project_symbols_write_value_request::ProjectSymbolsWriteValueRequest,
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
    /// Executes a Symbol Tree plugin action.
    ExecutePluginAction {
        #[structopt(flatten)]
        project_symbols_execute_plugin_action_request: ProjectSymbolsExecutePluginActionRequest,
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
    /// Replaces the opened project's entire symbol catalog.
    SetCatalog {
        #[structopt(flatten)]
        project_symbols_set_catalog_request: ProjectSymbolsSetCatalogRequest,
    },
    /// Updates project symbol claim properties.
    Update {
        #[structopt(flatten)]
        project_symbols_update_request: ProjectSymbolsUpdateRequest,
    },
    /// Writes an edited project symbol runtime value to process memory.
    WriteValue {
        #[structopt(flatten)]
        project_symbols_write_value_request: ProjectSymbolsWriteValueRequest,
    },
}
