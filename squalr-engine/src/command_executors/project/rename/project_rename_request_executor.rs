use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::project::rename::project_rename_request::ProjectRenameRequest;
use squalr_engine_api::commands::project::rename::project_rename_response::ProjectRenameResponse;
use squalr_engine_projects::project::project::Project;
use std::fs;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ProjectRenameRequest {
    type ResponseType = ProjectRenameResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let project_path = &self.project_path;
        let Some(parent_path) = project_path.parent() else {
            log::error!("Error getting parent directory of project for project rename operation.");
            return ProjectRenameResponse { renamed_project_info: None };
        };

        let new_project_path = parent_path.join(&self.new_project_name);
        if let Err(err) = fs::rename(project_path, &new_project_path) {
            log::error!("Failed to rename project: {}", err);
            return ProjectRenameResponse { renamed_project_info: None };
        }

        let project_manager = engine_privileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_result = opened_project.write();

        let Ok(current_project) = opened_project_result.as_deref_mut() else {
            log::error!("Failed to access opened project");
            return ProjectRenameResponse { renamed_project_info: None };
        };

        // If we are renaming the current project, just do a full reload.
        // We may actually be able to get away with mutating state, but for now I do not see the value in that.
        if let Some(current_project_ref) = current_project {
            if current_project_ref.get_project_info().get_path() == project_path {
                match Project::load_project(&new_project_path) {
                    Ok(reopened_project) => {
                        *current_project = Some(reopened_project);
                        log::info!("The current project has been renamed. Re-opening the project.");
                    }
                    Err(_) => {
                        *current_project = None;
                        log::error!("Error re-opening the current project after rename! Closing current project.");
                    }
                }
            }
        }

        ProjectRenameResponse { renamed_project_info: None }
    }
}
