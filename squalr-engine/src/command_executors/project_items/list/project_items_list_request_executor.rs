use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_response::ProjectItemsListResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsListRequest {
    type ResponseType = ProjectItemsListResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();

        match opened_project_lock.read() {
            Ok(opened_project_guard) => {
                let opened_project = match opened_project_guard.as_ref() {
                    Some(opened_project) => opened_project,
                    None => return ProjectItemsListResponse::default(),
                };
                let opened_project_root = opened_project.get_project_root().cloned();
                let opened_project_items = opened_project
                    .get_project_items()
                    .iter()
                    .map(|(project_item_ref, project_item)| (project_item_ref.clone(), project_item.clone()))
                    .collect();

                ProjectItemsListResponse {
                    opened_project_info: Some(opened_project.get_project_info().clone()),
                    opened_project_root,
                    opened_project_items,
                }
            }
            Err(error) => {
                log::error!("Error obtaining opened project lock for list command: {}", error);
                ProjectItemsListResponse::default()
            }
        }
    }
}
