use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::commands::project::save::project_save_response::ProjectSaveResponse;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ProjectSaveRequest {
    type ResponseType = ProjectSaveResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Ok(project) = engine_privileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .as_deref()
        {
            if let Some(project) = project {
                match project.save(true) {
                    Ok(_) => {
                        return ProjectSaveResponse { success: true };
                    }
                    Err(err) => {
                        log::error!("Failed to save project: {}", err);
                    }
                }
            }
        }

        ProjectSaveResponse { success: false }
    }
}
