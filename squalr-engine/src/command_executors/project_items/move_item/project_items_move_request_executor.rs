use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::move_item::project_items_move_request::ProjectItemsMoveRequest;
use squalr_engine_api::commands::project_items::move_item::project_items_move_response::ProjectItemsMoveResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsMoveRequest {
    type ResponseType = ProjectItemsMoveResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.project_item_paths.is_empty() {
            return ProjectItemsMoveResponse {
                success: true,
                moved_project_item_count: 0,
            };
        }

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for move command: {}", error);

                return ProjectItemsMoveResponse {
                    success: false,
                    moved_project_item_count: 0,
                };
            }
        };
        let opened_project = match opened_project_guard.as_ref() {
            Some(opened_project) => opened_project,
            None => {
                log::warn!("Cannot move project items without an opened project.");

                return ProjectItemsMoveResponse {
                    success: false,
                    moved_project_item_count: 0,
                };
            }
        };
        let project_directory_path = match opened_project.get_project_info().get_project_directory() {
            Some(project_directory_path) => project_directory_path,
            None => {
                log::error!("Failed to resolve opened project directory for move operation.");

                return ProjectItemsMoveResponse {
                    success: false,
                    moved_project_item_count: 0,
                };
            }
        };
        let target_directory_path = resolve_project_item_path(&project_directory_path, &self.target_directory_path);

        if let Err(error) = fs::create_dir_all(&target_directory_path) {
            log::error!("Failed to create move target directory {:?}: {}", target_directory_path, error);
            return ProjectItemsMoveResponse {
                success: false,
                moved_project_item_count: 0,
            };
        }

        let mut moved_project_item_count = 0_u64;
        let mut operation_success = true;

        for project_item_path in &self.project_item_paths {
            let source_project_item_path = resolve_project_item_path(&project_directory_path, project_item_path);
            let source_file_name = match source_project_item_path.file_name() {
                Some(source_file_name) => source_file_name.to_os_string(),
                None => {
                    log::error!("Project item path has no file name and cannot be moved: {:?}", source_project_item_path);
                    operation_success = false;
                    continue;
                }
            };
            let destination_project_item_path = target_directory_path.join(source_file_name);

            match fs::rename(&source_project_item_path, &destination_project_item_path) {
                Ok(()) => {
                    moved_project_item_count += 1;
                }
                Err(error) => {
                    log::error!(
                        "Failed to move project item from {:?} to {:?}: {}",
                        source_project_item_path,
                        destination_project_item_path,
                        error
                    );
                    operation_success = false;
                }
            }
        }

        if moved_project_item_count > 0 && !reload_opened_project(&mut opened_project_guard, &project_directory_path) {
            return ProjectItemsMoveResponse {
                success: false,
                moved_project_item_count,
            };
        }

        if moved_project_item_count > 0 {
            project_manager.notify_project_items_changed();
        }

        ProjectItemsMoveResponse {
            success: operation_success,
            moved_project_item_count,
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
