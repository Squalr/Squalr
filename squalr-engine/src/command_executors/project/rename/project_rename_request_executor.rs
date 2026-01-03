use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::rename::project_rename_request::ProjectRenameRequest;
use squalr_engine_api::commands::project::rename::project_rename_response::ProjectRenameResponse;
use squalr_engine_api::engine::engine_unprivileged_state::EngineUnprivilegedState;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::fs;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectRenameRequest {
    type ResponseType = ProjectRenameResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project = match opened_project.write() {
            Ok(opened_project) => opened_project,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for writing: {}", error);

                return ProjectRenameResponse { success: false };
            }
        };
        let is_renaming_opened_project = match project_manager.get_opened_project().read() {
            Ok(current_project) => match current_project.as_ref() {
                Some(current_project) => *current_project.get_project_info().get_project_file_path() == self.project_file_path,
                None => false,
            },
            Err(error) => {
                log::error!("Failed to check if renaming current project, aborting: {}", error);

                return ProjectRenameResponse { success: false };
            }
        };
        let project_directory_path = match self.project_file_path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => {
                log::error!("Failed to get parent path for project path: {:?}", self.project_file_path);

                return ProjectRenameResponse { success: false };
            }
        };

        let new_project_path = project_directory_path.join(&self.new_project_name);

        if let Err(error) = fs::rename(self.project_file_path.to_path_buf(), &new_project_path) {
            log::error!("Failed to rename project: {}", error);

            return ProjectRenameResponse { success: false };
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

        ProjectRenameResponse { success: true }
    }
}
