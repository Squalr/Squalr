use crate::commands::project_symbols::{
    create::project_symbols_create_response::ProjectSymbolsCreateResponse,
    create_module::project_symbols_create_module_response::ProjectSymbolsCreateModuleResponse,
    delete::project_symbols_delete_response::ProjectSymbolsDeleteResponse,
    delete_layout::project_symbols_delete_layout_response::ProjectSymbolsDeleteLayoutResponse,
    delete_resolver::project_symbols_delete_resolver_response::ProjectSymbolsDeleteResolverResponse,
    execute_plugin_action::project_symbols_execute_plugin_action_response::ProjectSymbolsExecutePluginActionResponse,
    list::project_symbols_list_response::ProjectSymbolsListResponse, rename::project_symbols_rename_response::ProjectSymbolsRenameResponse,
    rename_module::project_symbols_rename_module_response::ProjectSymbolsRenameModuleResponse,
    update::project_symbols_update_response::ProjectSymbolsUpdateResponse,
    upsert_layout::project_symbols_upsert_layout_response::ProjectSymbolsUpsertLayoutResponse,
    upsert_resolver::project_symbols_upsert_resolver_response::ProjectSymbolsUpsertResolverResponse,
    write_value::project_symbols_write_value_response::ProjectSymbolsWriteValueResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectSymbolsResponse {
    Create {
        project_symbols_create_response: ProjectSymbolsCreateResponse,
    },
    CreateModule {
        project_symbols_create_module_response: ProjectSymbolsCreateModuleResponse,
    },
    Delete {
        project_symbols_delete_response: ProjectSymbolsDeleteResponse,
    },
    DeleteLayout {
        project_symbols_delete_layout_response: ProjectSymbolsDeleteLayoutResponse,
    },
    DeleteResolver {
        project_symbols_delete_resolver_response: ProjectSymbolsDeleteResolverResponse,
    },
    ExecutePluginAction {
        project_symbols_execute_plugin_action_response: ProjectSymbolsExecutePluginActionResponse,
    },
    List {
        project_symbols_list_response: ProjectSymbolsListResponse,
    },
    Rename {
        project_symbols_rename_response: ProjectSymbolsRenameResponse,
    },
    RenameModule {
        project_symbols_rename_module_response: ProjectSymbolsRenameModuleResponse,
    },
    Update {
        project_symbols_update_response: ProjectSymbolsUpdateResponse,
    },
    UpsertLayout {
        project_symbols_upsert_layout_response: ProjectSymbolsUpsertLayoutResponse,
    },
    UpsertResolver {
        project_symbols_upsert_resolver_response: ProjectSymbolsUpsertResolverResponse,
    },
    WriteValue {
        project_symbols_write_value_response: ProjectSymbolsWriteValueResponse,
    },
}
