use crate::command_executors::project_items::project_item_sort_order::append_project_items_to_sort_order;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::duplicate::project_items_duplicate_request::ProjectItemsDuplicateRequest;
use squalr_engine_api::commands::project_items::duplicate::project_items_duplicate_response::ProjectItemsDuplicateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::utils::file_system::file_system_utils::FileSystemUtils;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsDuplicateRequest {
    type ResponseType = ProjectItemsDuplicateResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.project_item_paths.is_empty() {
            return ProjectItemsDuplicateResponse {
                success: true,
                duplicated_project_item_count: 0,
                duplicated_project_item_paths: Vec::new(),
            };
        }

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for duplicate command: {}", error);

                return ProjectItemsDuplicateResponse::default();
            }
        };
        let opened_project = match opened_project_guard.as_ref() {
            Some(opened_project) => opened_project,
            None => {
                log::warn!("Cannot duplicate project items without an opened project.");

                return ProjectItemsDuplicateResponse::default();
            }
        };
        let project_directory_path = match opened_project.get_project_info().get_project_directory() {
            Some(project_directory_path) => project_directory_path,
            None => {
                log::error!("Failed to resolve opened project directory for duplicate operation.");

                return ProjectItemsDuplicateResponse::default();
            }
        };
        let project_root_directory_path = project_directory_path.join(Project::PROJECT_DIR);
        let requested_target_directory_path = resolve_project_item_path(&project_directory_path, &self.target_directory_path);
        let target_directory_path = if requested_target_directory_path.starts_with(&project_root_directory_path) {
            requested_target_directory_path
        } else {
            project_root_directory_path.clone()
        };

        if let Err(error) = fs::create_dir_all(&target_directory_path) {
            log::error!("Failed to create duplicate target directory {:?}: {}", target_directory_path, error);

            return ProjectItemsDuplicateResponse::default();
        }

        let mut duplicated_project_item_paths = Vec::new();
        let mut operation_success = true;

        for project_item_path in &self.project_item_paths {
            let source_project_item_path = resolve_project_item_path(&project_directory_path, project_item_path);

            if source_project_item_path == project_root_directory_path {
                operation_success = false;
                continue;
            }

            if !source_project_item_path.exists() {
                log::warn!("Cannot duplicate missing project item path: {:?}.", source_project_item_path);
                operation_success = false;
                continue;
            }

            if source_project_item_path.is_dir() && target_directory_path.starts_with(&source_project_item_path) {
                log::warn!(
                    "Cannot duplicate project directory {:?} into descendant target {:?}.",
                    source_project_item_path,
                    target_directory_path
                );
                operation_success = false;
                continue;
            }

            let destination_project_item_path = generate_unique_duplicate_project_item_path(&target_directory_path, &source_project_item_path);

            let duplicate_result = if source_project_item_path.is_dir() {
                FileSystemUtils::copy_dir_all(&source_project_item_path, &destination_project_item_path)
            } else {
                fs::copy(&source_project_item_path, &destination_project_item_path).map(|_bytes_copied| ())
            };

            match duplicate_result {
                Ok(()) => duplicated_project_item_paths.push(destination_project_item_path),
                Err(error) => {
                    log::error!(
                        "Failed to duplicate project item from {:?} to {:?}: {}",
                        source_project_item_path,
                        destination_project_item_path,
                        error
                    );
                    operation_success = false;
                }
            }
        }

        if duplicated_project_item_paths.is_empty() {
            return ProjectItemsDuplicateResponse {
                success: operation_success,
                duplicated_project_item_count: 0,
                duplicated_project_item_paths,
            };
        }

        if !reload_opened_project(&mut opened_project_guard, &project_directory_path) {
            return ProjectItemsDuplicateResponse {
                success: false,
                duplicated_project_item_count: duplicated_project_item_paths.len() as u64,
                duplicated_project_item_paths,
            };
        }

        let Some(reloaded_opened_project) = opened_project_guard.as_mut() else {
            return ProjectItemsDuplicateResponse {
                success: false,
                duplicated_project_item_count: duplicated_project_item_paths.len() as u64,
                duplicated_project_item_paths,
            };
        };

        append_project_items_to_sort_order(reloaded_opened_project, &project_directory_path, &duplicated_project_item_paths);

        if let Err(error) = reloaded_opened_project.save_to_path(&project_directory_path, false) {
            log::error!("Failed to save project after duplicate operation: {}", error);

            return ProjectItemsDuplicateResponse {
                success: false,
                duplicated_project_item_count: duplicated_project_item_paths.len() as u64,
                duplicated_project_item_paths,
            };
        }

        project_manager.notify_project_items_changed();

        ProjectItemsDuplicateResponse {
            success: operation_success,
            duplicated_project_item_count: duplicated_project_item_paths.len() as u64,
            duplicated_project_item_paths,
        }
    }
}

fn resolve_project_item_path(
    project_directory_path: &Path,
    project_item_path: &Path,
) -> PathBuf {
    if FileSystemUtils::is_cross_platform_absolute_path(project_item_path) {
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
            log::error!("Failed to reload project after duplicate operation: {}", error);
            false
        }
    }
}

fn generate_unique_duplicate_project_item_path(
    target_directory_path: &Path,
    source_project_item_path: &Path,
) -> PathBuf {
    let source_file_name = source_project_item_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or("project_item");
    let source_stem = source_project_item_path
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
        .unwrap_or(source_file_name);
    let source_extension = source_project_item_path
        .extension()
        .and_then(|extension| extension.to_str());
    let is_directory = source_project_item_path.is_dir();
    let mut duplicate_sequence_number = 0_u64;

    loop {
        let candidate_name = if duplicate_sequence_number == 0 {
            if is_directory {
                source_file_name.to_string()
            } else if let Some(source_extension) = source_extension {
                format!("{}.{}", source_stem, source_extension)
            } else {
                source_stem.to_string()
            }
        } else if is_directory {
            format!("{}_{}", source_file_name, duplicate_sequence_number)
        } else if let Some(source_extension) = source_extension {
            format!("{}_{}.{}", source_stem, duplicate_sequence_number, source_extension)
        } else {
            format!("{}_{}", source_stem, duplicate_sequence_number)
        };
        let candidate_path = target_directory_path.join(candidate_name);

        if !candidate_path.exists() {
            return candidate_path;
        }

        duplicate_sequence_number = duplicate_sequence_number.saturating_add(1);
    }
}
