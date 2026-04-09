use crate::command_executors::project_items::project_item_sort_order::rename_project_item_in_sort_order;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest;
use squalr_engine_api::commands::project_items::rename::project_items_rename_response::ProjectItemsRenameResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
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

        if source_project_item_path == target_project_item_path {
            return ProjectItemsRenameResponse {
                success: true,
                renamed_project_item_path: source_project_item_path,
            };
        }

        if target_project_item_path.exists() {
            log::warn!(
                "Cannot rename project item {:?} to {:?} because the target already exists.",
                source_project_item_path,
                target_project_item_path
            );
            return ProjectItemsRenameResponse {
                success: false,
                renamed_project_item_path: PathBuf::new(),
            };
        }

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

        let Some(reloaded_opened_project) = opened_project_guard.as_mut() else {
            return ProjectItemsRenameResponse {
                success: false,
                renamed_project_item_path: PathBuf::new(),
            };
        };
        let renamed_project_item_ref = ProjectItemRef::new(target_project_item_path.clone());

        if let Some(renamed_project_item) = reloaded_opened_project.get_project_item_mut(&renamed_project_item_ref) {
            let renamed_project_item_display_name = build_project_item_display_name(&target_project_item_path, renamed_project_item);
            renamed_project_item.set_field_name(&renamed_project_item_display_name);
            renamed_project_item.set_has_unsaved_changes(true);
            reloaded_opened_project
                .get_project_info_mut()
                .set_has_unsaved_changes(true);
        } else {
            log::warn!("Renamed project item was not found after reload: {:?}", target_project_item_path);
        }

        rename_project_item_in_sort_order(
            reloaded_opened_project,
            &project_directory_path,
            &source_project_item_path,
            &target_project_item_path,
        );

        if let Err(error) = reloaded_opened_project.save_to_path(&project_directory_path, false) {
            log::error!("Failed to save project after project item rename operation: {}", error);
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

fn build_project_item_display_name(
    project_item_path: &Path,
    project_item: &squalr_engine_api::structures::projects::project_items::project_item::ProjectItem,
) -> String {
    let current_file_name = project_item_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    if project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
        current_file_name.to_string()
    } else {
        Path::new(current_file_name)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or(current_file_name)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::build_project_item_display_name;
    use squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
    use squalr_engine_api::structures::projects::project_items::built_in_types::{
        project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
    };
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use std::path::Path;

    #[test]
    fn build_project_item_display_name_uses_file_stem_for_file_items() {
        let project_item = ProjectItemTypeAddress::new_project_item("Address", 0x1234, "", "", DataTypeU8::get_value_from_primitive(0));

        let display_name = build_project_item_display_name(Path::new("C:/Projects/TestProject/project_items/0x5D123.json"), &project_item);

        assert_eq!(display_name, "0x5D123".to_string());
    }

    #[test]
    fn build_project_item_display_name_uses_directory_name_for_directory_items() {
        let project_item_ref = ProjectItemRef::new("C:/Projects/TestProject/project_items/Cheats".into());
        let project_item = ProjectItemTypeDirectory::new_project_item(&project_item_ref);

        let display_name = build_project_item_display_name(Path::new("C:/Projects/TestProject/project_items/Player Cheats"), &project_item);

        assert_eq!(display_name, "Player Cheats".to_string());
    }
}
