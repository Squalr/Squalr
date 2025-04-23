use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::project::open::project_open_request::ProjectOpenRequest;
use squalr_engine_api::commands::project::open::project_open_response::ProjectOpenResponse;
use squalr_engine_projects::project::project::Project;
use squalr_engine_projects::settings::project_settings_config::ProjectSettingsConfig;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ProjectOpenRequest {
    type ResponseType = ProjectOpenResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let project_path = ProjectSettingsConfig::get_projects_root().join(&self.project_name);

        if let Ok(project) = Project::open_project(&project_path) {
            let project_info = project.get_project_info().clone();

            engine_privileged_state
                .get_project_manager()
                .set_opened_project(project);

            ProjectOpenResponse {
                opened_project_info: Some(project_info),
            }
        } else {
            ProjectOpenResponse { opened_project_info: None }
        }
    }
}
