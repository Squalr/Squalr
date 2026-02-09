use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use rfd::FileDialog;
use squalr_engine_api::commands::project::open::project_open_request::ProjectOpenRequest;
use squalr_engine_api::commands::project::open::project_open_response::ProjectOpenResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use squalr_engine_projects::settings::project_settings_config::ProjectSettingsConfig;
use std::path::PathBuf;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectOpenRequest {
    type ResponseType = ProjectOpenResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project = match opened_project.write() {
            Ok(opened_project) => opened_project,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for writing: {}", error);
                return ProjectOpenResponse { success: false };
            }
        };

        let mut selected_path: Option<PathBuf> = None;

        if self.open_file_browser {
            selected_path = FileDialog::new().pick_folder();
            if selected_path.is_none() {
                log::info!("File browser cancelled by user.");
                return ProjectOpenResponse { success: false };
            }
        }

        // Prioritize open options in the following order: file browser selected path > direct path > constructed from name.
        let project_directory_path = if let Some(path) = selected_path {
            path
        } else if let Some(project_directory_path) = &self.project_directory_path {
            project_directory_path.into()
        } else {
            let name = self.project_name.as_deref().unwrap_or_default();
            ProjectSettingsConfig::get_projects_root().join(name)
        };

        match Project::load_from_path(&project_directory_path) {
            Ok(project) => {
                *opened_project = Some(project);
                ProjectOpenResponse { success: true }
            }
            Err(error) => {
                log::error!("Failed to open project from path:{:?}, error: {}", project_directory_path, error);
                ProjectOpenResponse { success: false }
            }
        }
    }
}
