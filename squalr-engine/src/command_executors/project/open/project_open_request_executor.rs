use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::open::project_open_request::ProjectOpenRequest;
use squalr_engine_api::commands::project::open::project_open_response::ProjectOpenResponse;
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use squalr_engine_projects::settings::project_settings_config::ProjectSettingsConfig;

impl UnprivilegedCommandRequestExecutor for ProjectOpenRequest {
    type ResponseType = ProjectOpenResponse;

    fn execute(
        &self,
        engine_api_unprivileged_bindings: &dyn EngineApiUnprivilegedBindings,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        // If a path is provided, use this directly. Otherwise, try to use the project settings relative name to construct the path.
        let project_path = if let Some(path) = &self.project_path {
            path.into()
        } else {
            let name = self.project_name.as_deref().unwrap_or_default();
            ProjectSettingsConfig::get_projects_root().join(name)
        };

        /*
        match Project::load_from_path(&project_path) {
            Ok(project) => {
                let project_info = project.get_project_info().clone();
                let project_root = project.get_project_root().clone();

                engine_privileged_state
                    .get_project_manager()
                    .set_opened_project(project);

                ProjectOpenResponse {
                    opened_project_info: Some(project_info),
                    opened_project_root: Some(project_root),
                }
            }
            Err(error) => {
                log::error!("Failed to open project: {}", error);

                ProjectOpenResponse {
                    opened_project_info: None,
                    opened_project_root: None,
                }
            }
        }*/
        ProjectOpenResponse {
            opened_project_info: None,
            opened_project_root: None,
        }
    }
}
