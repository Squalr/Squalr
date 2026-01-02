use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::close::project_close_request::ProjectCloseRequest;
use squalr_engine_api::commands::project::close::project_close_response::ProjectCloseResponse;
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;

impl UnprivilegedCommandRequestExecutor for ProjectCloseRequest {
    type ResponseType = ProjectCloseResponse;

    fn execute(
        &self,
        engine_api_unprivileged_bindings: &dyn EngineApiUnprivilegedBindings,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        /*
        engine_privileged_state
            .get_project_manager()
            .close_opened_project();
        */
        ProjectCloseResponse {}
    }
}
