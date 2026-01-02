use crate::command_executors::{unprivileged_command_executor::UnprivilegedCommandExecutor, unprivileged_request_executor::UnprivilegedCommandRequestExecutor};
use squalr_engine_api::{
    commands::{
        project::project_command::ProjectCommand,
        unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse},
    },
    engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings,
};

impl UnprivilegedCommandExecutor for ProjectCommand {
    type ResponseType = UnprivilegedCommandResponse;

    fn execute(
        &self,
        engine_api_unprivileged_bindings: &dyn EngineApiUnprivilegedBindings,
    ) -> <Self as UnprivilegedCommandExecutor>::ResponseType {
        match self {
            ProjectCommand::Create { project_create_request } => project_create_request
                .execute(engine_api_unprivileged_bindings)
                .to_engine_response(),
            ProjectCommand::Open { project_open_request } => project_open_request
                .execute(engine_api_unprivileged_bindings)
                .to_engine_response(),
            ProjectCommand::Close { project_close_request } => project_close_request
                .execute(engine_api_unprivileged_bindings)
                .to_engine_response(),
            ProjectCommand::Rename { project_rename_request } => project_rename_request
                .execute(engine_api_unprivileged_bindings)
                .to_engine_response(),
            ProjectCommand::Save { project_save_request } => project_save_request
                .execute(engine_api_unprivileged_bindings)
                .to_engine_response(),
            ProjectCommand::Export { project_export_request } => project_export_request
                .execute(engine_api_unprivileged_bindings)
                .to_engine_response(),
            ProjectCommand::List { project_list_request } => project_list_request
                .execute(engine_api_unprivileged_bindings)
                .to_engine_response(),
        }
    }
}
