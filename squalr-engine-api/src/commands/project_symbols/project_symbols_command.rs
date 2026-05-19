use crate::commands::project_symbols::{
    create::project_symbols_create_request::ProjectSymbolsCreateRequest,
    create_module::project_symbols_create_module_request::ProjectSymbolsCreateModuleRequest,
    delete::project_symbols_delete_request::ProjectSymbolsDeleteRequest,
    delete_layout::project_symbols_delete_layout_request::ProjectSymbolsDeleteLayoutRequest,
    delete_resolver::project_symbols_delete_resolver_request::ProjectSymbolsDeleteResolverRequest,
    execute_plugin_action::project_symbols_execute_plugin_action_request::ProjectSymbolsExecutePluginActionRequest,
    list::project_symbols_list_request::ProjectSymbolsListRequest, rename::project_symbols_rename_request::ProjectSymbolsRenameRequest,
    rename_module::project_symbols_rename_module_request::ProjectSymbolsRenameModuleRequest,
    update::project_symbols_update_request::ProjectSymbolsUpdateRequest,
    upsert_layout::project_symbols_upsert_layout_request::ProjectSymbolsUpsertLayoutRequest,
    upsert_resolver::project_symbols_upsert_resolver_request::ProjectSymbolsUpsertResolverRequest,
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
    /// Deletes a reusable symbol layout.
    DeleteLayout {
        #[structopt(flatten)]
        project_symbols_delete_layout_request: ProjectSymbolsDeleteLayoutRequest,
    },
    /// Deletes a reusable symbol resolver.
    DeleteResolver {
        #[structopt(flatten)]
        project_symbols_delete_resolver_request: ProjectSymbolsDeleteResolverRequest,
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
    /// Updates project symbol claim properties.
    Update {
        #[structopt(flatten)]
        project_symbols_update_request: ProjectSymbolsUpdateRequest,
    },
    /// Creates or replaces a reusable symbol layout.
    UpsertLayout {
        #[structopt(flatten)]
        project_symbols_upsert_layout_request: ProjectSymbolsUpsertLayoutRequest,
    },
    /// Creates or replaces a reusable symbol resolver.
    UpsertResolver {
        #[structopt(flatten)]
        project_symbols_upsert_resolver_request: ProjectSymbolsUpsertResolverRequest,
    },
    /// Writes an edited project symbol runtime value to process memory.
    WriteValue {
        #[structopt(flatten)]
        project_symbols_write_value_request: ProjectSymbolsWriteValueRequest,
    },
}
