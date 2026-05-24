use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::projects::project_item_activation::{apply_project_item_activation, dispatch_memory_freeze_request};
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::activate::project_items_activate_response::ProjectItemsActivateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsActivateRequest {
    type ResponseType = ProjectItemsActivateResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project_lock.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for project-item activation: {}", error);

                return ProjectItemsActivateResponse {};
            }
        };
        let opened_project = match opened_project_guard.as_mut() {
            Some(opened_project) => opened_project,
            None => {
                log::warn!("Unable to activate project items because no project is opened.");

                return ProjectItemsActivateResponse {};
            }
        };
        let activation_change_set = apply_project_item_activation(opened_project.get_project_items_mut(), &self.project_item_paths, self.is_activated);

        drop(opened_project_guard);

        if activation_change_set.has_activation_changes {
            dispatch_memory_freeze_request(engine_unprivileged_state, &activation_change_set.freeze_targets, self.is_activated);
            project_manager.notify_project_items_changed();
        }

        ProjectItemsActivateResponse {}
    }
}
