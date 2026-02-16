use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest;
use squalr_engine_api::commands::project_items::create::project_items_create_response::ProjectItemsCreateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsCreateRequest {
    type ResponseType = ProjectItemsCreateResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.project_item_type != ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
            log::error!(
                "Unsupported project item type for create command: {}. Only '{}' is currently supported.",
                self.project_item_type,
                ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID
            );

            return ProjectItemsCreateResponse {
                success: false,
                created_project_item_path: PathBuf::new(),
            };
        }

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for create command: {}", error);

                return ProjectItemsCreateResponse {
                    success: false,
                    created_project_item_path: PathBuf::new(),
                };
            }
        };
        let opened_project = match opened_project_guard.as_ref() {
            Some(opened_project) => opened_project,
            None => {
                log::warn!("Cannot create project items without an opened project.");

                return ProjectItemsCreateResponse {
                    success: false,
                    created_project_item_path: PathBuf::new(),
                };
            }
        };
        let project_directory_path = match opened_project.get_project_info().get_project_directory() {
            Some(project_directory_path) => project_directory_path,
            None => {
                log::error!("Failed to resolve opened project directory for create operation.");

                return ProjectItemsCreateResponse {
                    success: false,
                    created_project_item_path: PathBuf::new(),
                };
            }
        };
        let parent_directory_path = resolve_project_item_path(&project_directory_path, &self.parent_directory_path);
        let created_project_item_path = parent_directory_path.join(&self.project_item_name);

        if let Err(error) = fs::create_dir_all(&created_project_item_path) {
            log::error!("Failed to create project item directory {:?}: {}", created_project_item_path, error);

            return ProjectItemsCreateResponse {
                success: false,
                created_project_item_path: PathBuf::new(),
            };
        }

        if !reload_opened_project(&mut opened_project_guard, &project_directory_path) {
            return ProjectItemsCreateResponse {
                success: false,
                created_project_item_path: PathBuf::new(),
            };
        }

        project_manager.notify_project_items_changed();

        ProjectItemsCreateResponse {
            success: true,
            created_project_item_path,
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
