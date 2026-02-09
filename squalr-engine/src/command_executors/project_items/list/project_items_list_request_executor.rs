use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_response::ProjectItemsListResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsListRequest {
    type ResponseType = ProjectItemsListResponse;

    fn execute(
        &self,
        _engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        /*
        match engine_privileged_state
            .get_project_manager()
            .get_opened_project()
            .read()
        {
            Ok(opened_project) => match opened_project.as_ref() {
                Some(opened_project) => {
                    return ProjectItemsListResponse {
                        opened_project_info: Some(opened_project.get_project_info().clone()),
                        opened_project_root: Some(opened_project.get_project_root().clone()),
                    };
                }
                None => {
                    return ProjectItemsListResponse::default();
                }
            },
            Err(error) => {
                log::error!("Error obtaining opened project lock: {}", error);
                return ProjectItemsListResponse::default();
            }
        }*/
        ProjectItemsListResponse::default()
    }
}
