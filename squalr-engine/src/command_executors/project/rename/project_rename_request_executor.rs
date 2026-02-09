use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::rename::project_rename_request::ProjectRenameRequest;
use squalr_engine_api::commands::project::rename::project_rename_response::ProjectRenameResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectRenameRequest {
    type ResponseType = ProjectRenameResponse;

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

                return ProjectRenameResponse {
                    success: false,
                    new_project_path: PathBuf::default(),
                };
            }
        };
        let is_renaming_opened_project = match opened_project.as_ref() {
            Some(opened_project) => {
                *opened_project
                    .get_project_info()
                    .get_project_directory()
                    .unwrap_or_default()
                    == self.project_directory_path
            }
            None => false,
        };
        let all_projects_directory = match self.project_directory_path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => {
                return ProjectRenameResponse {
                    success: false,
                    new_project_path: PathBuf::default(),
                };
            }
        };
        let new_project_path = all_projects_directory.join(&self.new_project_name);

        if let Err(error) = fs::rename(&self.project_directory_path, &new_project_path) {
            log::error!("Failed to rename project: {}", error);
            return ProjectRenameResponse {
                success: false,
                new_project_path: PathBuf::default(),
            };
        }

        // If we are renaming the current project, just do a full reload.
        // We may actually be able to get away with mutating state, but for now I do not see the value in that.
        if is_renaming_opened_project {
            match Project::load_from_path(&new_project_path) {
                Ok(reopened_project) => {
                    log::info!("The current project has been renamed. Re-opening the project.");

                    *opened_project = Some(reopened_project);
                }
                Err(_) => {
                    log::error!("Error re-opening the current project after rename! Closing current project.");

                    *opened_project = None;
                }
            }
        }

        ProjectRenameResponse {
            success: true,
            new_project_path: new_project_path,
        }
    }
}
