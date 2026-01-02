use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::export::project_export_response::ProjectExportResponse;
use squalr_engine_api::engine::engine_unprivileged_state::EngineUnprivilegedState;
use squalr_engine_api::{commands::project::export::project_export_request::ProjectExportRequest, structures::projects::project::Project};
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use squalr_engine_projects::settings::project_settings_config::ProjectSettingsConfig;
use std::fs::{self, OpenOptions};
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectExportRequest {
    type ResponseType = ProjectExportResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        // If a path is provided, use this directly. Otherwise, try to use the project settings relative name to construct the path.
        let project_path = if let Some(path) = &self.project_path {
            path.into()
        } else {
            let name = self.project_name.as_deref().unwrap_or_default();
            ProjectSettingsConfig::get_projects_root().join(name)
        };

        if let Ok(project) = Project::load_from_path(&project_path) {
            let export_path = project.get_project_info().get_path().join("export");
            let project_name = project.get_name();
            let export_file_path = export_path.join(format!("{}.json", project_name));

            // Best effort to create the export directory.
            let _ = fs::create_dir(&export_path);

            match OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&export_file_path)
            {
                Ok(file) => match serde_json::to_writer(file, &project) {
                    Ok(()) => {
                        log::error!("Exported project to path: {:?}", export_file_path);

                        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
                        {
                            if self.open_export_folder {
                                let _ = opener::open(&export_path);
                            }
                        }

                        return ProjectExportResponse { success: true };
                    }
                    Err(error) => {
                        log::error!("Failed to write exported project: {}", error);
                    }
                },
                Err(error) => {
                    log::error!("Failed to export project: {}", error);
                }
            }
        }

        ProjectExportResponse { success: false }
    }
}
