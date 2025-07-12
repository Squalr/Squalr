use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use olorin_engine_api::commands::project::save::project_save_response::ProjectSaveResponse;
use olorin_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
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
            .as_deref_mut()
        {
            if let Some(project) = project {
                // Attempt to update the project icon if we are attached to a process.
                if let Some(opened_process) = engine_privileged_state
                    .get_process_manager()
                    .get_opened_process()
                {
                    if let Some(process_icon) = opened_process.get_icon() {
                        project.set_project_icon(Some(process_icon.clone()));
                    }
                }

                let project_path = project.get_project_info().get_path().to_owned();

                // Persist the project to disk.
                match project.save_to_path(&project_path, true, true) {
                    Ok(_) => {
                        return ProjectSaveResponse { success: true };
                    }
                    Err(error) => {
                        log::error!("Failed to save project: {}", error);
                    }
                }
            }
        }

        ProjectSaveResponse { success: false }
    }
}
