use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::commands::project::save::project_save_response::ProjectSaveResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSaveRequest {
    type ResponseType = ProjectSaveResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project = match opened_project.write() {
            Ok(opened_project) => opened_project,
            Err(error) => {
                log::error!("Failed to acquire opened project: {}", error);
                return ProjectSaveResponse { success: false };
            }
        };
        let opened_project = match opened_project.as_mut() {
            Some(opened_project) => opened_project,
            None => {
                log::error!("Cannot save project, no project is opened!");
                return ProjectSaveResponse { success: false };
            }
        };

        // JIRA: Reinstate this.
        /*
        if let Some(process_icon) = opened_process.get_icon() {
            opened_project.set_project_icon(Some(process_icon.clone()));
        }*/

        let project_directory_path = match opened_project.get_project_info().get_project_directory() {
            Some(project_directory_path) => project_directory_path,
            None => {
                log::error!("Failed to locate opened project folder, cannot save project!");

                return ProjectSaveResponse { success: false };
            }
        };

        // Persist the project to disk.
        match opened_project.save_to_path(&project_directory_path, false) {
            Ok(()) => {
                return ProjectSaveResponse { success: true };
            }
            Err(error) => {
                log::error!("Failed to save project: {}", error);
            }
        }

        ProjectSaveResponse { success: false }
    }
}
