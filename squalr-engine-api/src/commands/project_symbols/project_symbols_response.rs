use crate::commands::project_symbols::{
    create::project_symbols_create_response::ProjectSymbolsCreateResponse,
    create_module::project_symbols_create_module_response::ProjectSymbolsCreateModuleResponse,
    delete::project_symbols_delete_response::ProjectSymbolsDeleteResponse,
    execute_plugin_action::project_symbols_execute_plugin_action_response::ProjectSymbolsExecutePluginActionResponse,
    list::project_symbols_list_response::ProjectSymbolsListResponse, rename::project_symbols_rename_response::ProjectSymbolsRenameResponse,
    rename_module::project_symbols_rename_module_response::ProjectSymbolsRenameModuleResponse,
    update::project_symbols_update_response::ProjectSymbolsUpdateResponse,
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
}
