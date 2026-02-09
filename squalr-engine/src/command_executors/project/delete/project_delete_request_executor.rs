use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::delete::project_delete_request::ProjectDeleteRequest;
use squalr_engine_api::commands::project::delete::project_delete_response::ProjectDeleteResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_projects::settings::project_settings_config::ProjectSettingsConfig;
use std::fs;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectDeleteRequest {
    type ResponseType = ProjectDeleteResponse;

    fn execute(
        &self,
        _engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        // If a path is provided, use this directly. Otherwise, try to use the project settings relative name to construct the path.
        // If no path nor project name is provided, we will just make an empty project with a default name.
        let project_directory_path = if let Some(project_directory_path) = &self.project_directory_path {
            project_directory_path.into()
        } else if let Some(project_name) = self.project_name.as_deref() {
            ProjectSettingsConfig::get_projects_root().join(project_name)
        } else {
            log::error!("No path or name given, cannot delete project.");

            return ProjectDeleteResponse { success: false };
        };

        if !project_directory_path.exists() {
            log::error!("Project directory does not exist!");

            return ProjectDeleteResponse { success: false };
        }

        match fs::remove_dir_all(project_directory_path) {
            Ok(()) => ProjectDeleteResponse { success: true },
            Err(error) => {
                log::error!("Failed to delete project directory: {}", error);

                return ProjectDeleteResponse { success: false };
            }
        }
    }
}
