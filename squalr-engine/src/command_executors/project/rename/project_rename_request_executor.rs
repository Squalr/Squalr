use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::rename::project_rename_request::ProjectRenameRequest;
use squalr_engine_api::commands::project::rename::project_rename_response::ProjectRenameResponse;
use squalr_engine_api::engine::engine_unprivileged_state::EngineUnprivilegedState;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::fs;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectRenameRequest {
    type ResponseType = ProjectRenameResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_path = &self.project_path;
        let Some(parent_path) = project_path.parent() else {
            log::error!("Error getting parent directory of project for project rename operation.");
            return ProjectRenameResponse { renamed_project_info: None };
        };

        /*
        let project_manager = engine_privileged_state.get_project_manager();
        let is_renaming_opened_project = match project_manager.get_opened_project().read() {
            Ok(current_project) => match current_project.as_ref() {
                Some(current_project) => current_project.get_project_info().get_path() == project_path,
                None => false,
            },
            Err(error) => {
                log::error!("Failed to check if renaming current project, aborting: {}", error);
                return ProjectRenameResponse { renamed_project_info: None };
            }
        };

        let new_project_path = parent_path.join(&self.new_project_name);
        if let Err(error) = fs::rename(project_path, &new_project_path) {
            log::error!("Failed to rename project: {}", error);
            return ProjectRenameResponse { renamed_project_info: None };
        }

        // If we are renaming the current project, just do a full reload.
        // We may actually be able to get away with mutating state, but for now I do not see the value in that.
        if is_renaming_opened_project {
            match Project::load_from_path(&new_project_path) {
                Ok(reopened_project) => {
                    log::info!("The current project has been renamed. Re-opening the project.");
                    project_manager.set_opened_project(reopened_project);
                }
                Err(_) => {
                    log::error!("Error re-opening the current project after rename! Closing current project.");
                    project_manager.close_opened_project();
                }
            }
        }
        */
        ProjectRenameResponse { renamed_project_info: None }
    }
}
