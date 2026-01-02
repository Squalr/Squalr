use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::create::project_create_request::ProjectCreateRequest;
use squalr_engine_api::commands::project::create::project_create_response::ProjectCreateResponse;
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_unprivileged_state::EngineUnprivilegedState;
use squalr_engine_api::utils::file_system::file_system_utils::FileSystemUtils;
use squalr_engine_projects::settings::project_settings_config::ProjectSettingsConfig;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectCreateRequest {
    type ResponseType = ProjectCreateResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        /*
        // If a path is provided, use this directly. Otherwise, try to use the project settings relative name to construct the path.
        // If no path nor project name is provided, we will just make an empty project with a default name.
        let project_path = if let Some(path) = &self.project_path {
            path.into()
        } else if let Some(project_name) = self.project_name.as_deref() {
            ProjectSettingsConfig::get_projects_root().join(project_name)
        } else {
            let project_root = ProjectSettingsConfig::get_projects_root();

            match FileSystemUtils::create_unique_folder(project_root.as_path(), "New Project") {
                Ok(path) => path,
                Err(error) => {
                    log::error!("Failed to create a unique default project name: {}", error);
                    return ProjectCreateResponse { created_project_info: None };
                }
            }
        };

        match Project::operation_create_project(&project_path) {
            Ok(project) => {
                let project_info = project.get_project_info().clone();

                ProjectCreateResponse {
                    created_project_info: Some(project_info),
                }
            }
            Err(error) => {
                log::error!("Failed to create project: {}", error);

                ProjectCreateResponse { created_project_info: None }
            }
        }*/

        ProjectCreateResponse { created_project_info: None }
    }
}
