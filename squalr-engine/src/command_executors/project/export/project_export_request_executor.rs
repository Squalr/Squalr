use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::export::project_export_request::ProjectExportRequest;
use squalr_engine_api::commands::project::export::project_export_response::ProjectExportResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::fs::{self, OpenOptions};
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectExportRequest {
    type ResponseType = ProjectExportResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let opened_project = match opened_project.read() {
            Ok(opened_project) => opened_project,
            Err(error) => {
                log::error!("Failed to acquire opened project: {}", error);
                return ProjectExportResponse { success: false };
            }
        };
        let opened_project = match opened_project.as_ref() {
            Some(opened_project) => opened_project,
            None => {
                log::error!("Cannot export project, no project is opened!");
                return ProjectExportResponse { success: false };
            }
        };
        let project_folder = match opened_project.get_project_info().get_project_directory() {
            Some(project_folder) => project_folder,
            None => {
                log::error!("Cannot export project, failed to locate project folder.");
                return ProjectExportResponse { success: false };
            }
        };
        let export_path = project_folder.join("export");
        let project_name = opened_project.get_name();
        let export_file_path = export_path.join(format!("{}.json", project_name));

        // Best effort to create the export directory.
        let _ = fs::create_dir(&export_path);

        match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&export_file_path)
        {
            Ok(file) => match serde_json::to_writer(file, &opened_project) {
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

        ProjectExportResponse { success: false }
    }
}
