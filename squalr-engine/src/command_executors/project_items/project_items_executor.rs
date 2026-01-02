use crate::command_executors::unprivileged_command_executor::UnprivilegedCommandExecutor;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::project_items_command::ProjectItemsCommand;
use squalr_engine_api::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;

impl UnprivilegedCommandExecutor for ProjectItemsCommand {
    type ResponseType = UnprivilegedCommandResponse;

    fn execute(
        &self,
        engine_api_privileged_bindings: &dyn EngineApiUnprivilegedBindings,
    ) -> <Self as UnprivilegedCommandExecutor>::ResponseType {
        match self {
            ProjectItemsCommand::Activate {
                project_items_activate_request,
            } => project_items_activate_request
                .execute(engine_api_privileged_bindings)
                .to_engine_response(),
            ProjectItemsCommand::List { project_items_list_request } => project_items_list_request
                .execute(engine_api_privileged_bindings)
                .to_engine_response(),
        }
    }
}
