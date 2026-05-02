use crate::command_executors::{unprivileged_command_executor::UnprivilegedCommandExecutor, unprivileged_request_executor::UnprivilegedCommandRequestExecutor};
use squalr_engine_api::{
    commands::{
        project_symbols::project_symbols_command::ProjectSymbolsCommand,
        unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse},
    },
    engine::engine_execution_context::EngineExecutionContext,
};
use std::sync::Arc;

impl UnprivilegedCommandExecutor for ProjectSymbolsCommand {
    type ResponseType = UnprivilegedCommandResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandExecutor>::ResponseType {
        match self {
            ProjectSymbolsCommand::Create {
                project_symbols_create_request,
            } => project_symbols_create_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
            ProjectSymbolsCommand::CreateModule {
                project_symbols_create_module_request,
            } => project_symbols_create_module_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
            ProjectSymbolsCommand::Delete {
                project_symbols_delete_request,
            } => project_symbols_delete_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
            ProjectSymbolsCommand::List { project_symbols_list_request } => project_symbols_list_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
            ProjectSymbolsCommand::Rename {
                project_symbols_rename_request,
            } => project_symbols_rename_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
            ProjectSymbolsCommand::RenameModule {
                project_symbols_rename_module_request,
            } => project_symbols_rename_module_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
            ProjectSymbolsCommand::Update {
                project_symbols_update_request,
            } => project_symbols_update_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
        }
    }
}
