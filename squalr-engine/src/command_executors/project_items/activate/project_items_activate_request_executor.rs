use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::activate::project_items_activate_response::ProjectItemsActivateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsActivateRequest {
    type ResponseType = ProjectItemsActivateResponse;

    fn execute(
        &self,
        _engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        /*
        match engine_privileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
        {
            Ok(mut project_manager) => {
                if let Some(project_manager) = project_manager.as_mut() {
                    for project_item_path in &self.project_item_paths {
                        let project_item_path = Path::new(&project_item_path);

                        /*
                        if let Some(project_item) = project_manager.find_project_item_mut(project_item_path) {
                            project_item.set_activated(
                                engine_privileged_state.get_engine_bindings(),
                                &engine_privileged_state.get_registries(),
                                self.is_activated,
                            );
                        } else {
                            log::error!("Failed to find project item: {:?}", project_item_path)
                        }*/
                    }
                } else {
                    log::error!("Unable to activate project items, no opened project.");
                }
            }
            Err(error) => {
                log::error!("Error acquiring project manager: {}", error)
            }
        }*/

        ProjectItemsActivateResponse {}
    }
}
