use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest;
use squalr_engine_api::commands::project_items::rename::project_items_rename_response::ProjectItemsRenameResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsRenameRequest {
    type ResponseType = ProjectItemsRenameResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for rename command: {}", error);

                return ProjectItemsRenameResponse {
                    success: false,
                    renamed_project_item_path: PathBuf::new(),
                };
            }
        };
        let opened_project = match opened_project_guard.as_ref() {
            Some(opened_project) => opened_project,
            None => {
                log::warn!("Cannot rename project items without an opened project.");

                return ProjectItemsRenameResponse {
                    success: false,
                    renamed_project_item_path: PathBuf::new(),
                };
            }
        };
        let project_directory_path = match opened_project.get_project_info().get_project_directory() {
            Some(project_directory_path) => project_directory_path,
            None => {
                log::error!("Failed to resolve opened project directory for rename operation.");

                return ProjectItemsRenameResponse {
                    success: false,
                    renamed_project_item_path: PathBuf::new(),
                };
            }
        };
        let source_project_item_path = resolve_project_item_path(&project_directory_path, &self.project_item_path);
        let target_project_item_path = match source_project_item_path.parent() {
            Some(parent_directory_path) => parent_directory_path.join(&self.project_item_name),
            None => {
                log::error!("Failed to resolve parent directory for project item {:?}", source_project_item_path);
                return ProjectItemsRenameResponse {
                    success: false,
                    renamed_project_item_path: PathBuf::new(),
                };
            }
        };

        if let Err(error) = fs::rename(&source_project_item_path, &target_project_item_path) {
            log::error!(
                "Failed to rename project item from {:?} to {:?}: {}",
                source_project_item_path,
                target_project_item_path,
                error
            );
            return ProjectItemsRenameResponse {
                success: false,
                renamed_project_item_path: PathBuf::new(),
            };
        }

        if !reload_opened_project(&mut opened_project_guard, &project_directory_path) {
            return ProjectItemsRenameResponse {
                success: false,
                renamed_project_item_path: PathBuf::new(),
            };
        }

        project_manager.notify_project_items_changed();

        ProjectItemsRenameResponse {
            success: true,
            renamed_project_item_path: target_project_item_path,
        }
    }
}

fn resolve_project_item_path(
    project_directory_path: &Path,
    project_item_path: &Path,
) -> PathBuf {
    if project_item_path.is_absolute() {
        project_item_path.to_path_buf()
    } else {
        project_directory_path.join(project_item_path)
    }
}

fn reload_opened_project(
    opened_project_guard: &mut Option<Project>,
    project_directory_path: &Path,
) -> bool {
    match Project::load_from_path(project_directory_path) {
        Ok(reloaded_project) => {
            *opened_project_guard = Some(reloaded_project);
            true
        }
        Err(error) => {
            log::error!("Failed to reload project after project item mutation: {}", error);
            false
        }
    }
}
