use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::delete::project_items_delete_request::ProjectItemsDeleteRequest;
use squalr_engine_api::commands::project_items::delete::project_items_delete_response::ProjectItemsDeleteResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsDeleteRequest {
    type ResponseType = ProjectItemsDeleteResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.project_item_paths.is_empty() {
            return ProjectItemsDeleteResponse {
                success: true,
                deleted_project_item_count: 0,
            };
        }

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for delete command: {}", error);

                return ProjectItemsDeleteResponse {
                    success: false,
                    deleted_project_item_count: 0,
                };
            }
        };
        let opened_project = match opened_project_guard.as_ref() {
            Some(opened_project) => opened_project,
            None => {
                log::warn!("Cannot delete project items without an opened project.");

                return ProjectItemsDeleteResponse {
                    success: false,
                    deleted_project_item_count: 0,
                };
            }
        };
        let project_directory_path = match opened_project.get_project_info().get_project_directory() {
            Some(project_directory_path) => project_directory_path,
            None => {
                log::error!("Failed to resolve opened project directory for delete operation.");

                return ProjectItemsDeleteResponse {
                    success: false,
                    deleted_project_item_count: 0,
                };
            }
        };

        let mut deleted_project_item_count = 0_u64;
        let mut operation_success = true;

        for project_item_path in &self.project_item_paths {
            let resolved_project_item_path = resolve_project_item_path(&project_directory_path, project_item_path);

            if !resolved_project_item_path.exists() {
                continue;
            }

            if resolved_project_item_path.is_file() {
                match fs::remove_file(&resolved_project_item_path) {
                    Ok(()) => {
                        deleted_project_item_count += 1;
                    }
                    Err(error) => {
                        log::error!("Failed to delete project item file {:?}: {}", resolved_project_item_path, error);
                        operation_success = false;
                    }
                }
            } else if resolved_project_item_path.is_dir() {
                match fs::remove_dir_all(&resolved_project_item_path) {
                    Ok(()) => {
                        deleted_project_item_count += 1;
                    }
                    Err(error) => {
                        log::error!("Failed to delete project item directory {:?}: {}", resolved_project_item_path, error);
                        operation_success = false;
                    }
                }
            }
        }

        if deleted_project_item_count > 0 && !reload_opened_project(&mut opened_project_guard, &project_directory_path) {
            return ProjectItemsDeleteResponse {
                success: false,
                deleted_project_item_count,
            };
        }

        if deleted_project_item_count > 0 {
            project_manager.notify_project_items_changed();
        }

        ProjectItemsDeleteResponse {
            success: operation_success,
            deleted_project_item_count,
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
