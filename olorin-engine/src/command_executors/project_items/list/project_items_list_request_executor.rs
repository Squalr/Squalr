use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use olorin_engine_api::commands::project_items::list::project_items_list_response::ProjectItemsListResponse;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ProjectItemsListRequest {
    type ResponseType = ProjectItemsListResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
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
        return ProjectItemsListResponse::default();
    }
}
