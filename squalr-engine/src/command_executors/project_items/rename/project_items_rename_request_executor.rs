use crate::command_executors::project_items::project_item_sort_order::rename_project_item_in_sort_order;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::projects::project_item_file_mutation::{
    generate_unique_project_item_file_path_allowing, resolve_project_item_path, sanitize_file_name_component,
};
use squalr_engine_api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest;
use squalr_engine_api::commands::project_items::rename::project_items_rename_response::ProjectItemsRenameResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::fs;
use std::io::ErrorKind;
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
        let Some(opened_project) = opened_project_guard.as_mut() else {
            log::warn!("Cannot rename project items without an opened project.");

            return ProjectItemsRenameResponse {
                success: false,
                renamed_project_item_path: PathBuf::new(),
            };
        };
        let project_directory_path = match opened_project.get_project_info().get_project_directory() {
            Some(project_directory_path) => project_directory_path.clone(),
            None => {
                log::error!("Failed to resolve opened project directory for rename operation.");

                return ProjectItemsRenameResponse {
                    success: false,
                    renamed_project_item_path: PathBuf::new(),
                };
            }
        };
        let requested_project_item_name = self.project_item_name.trim();

        if requested_project_item_name.is_empty() {
            log::warn!("Cannot rename project item to an empty name.");

            return ProjectItemsRenameResponse {
                success: false,
                renamed_project_item_path: PathBuf::new(),
            };
        }

        let source_project_item_path = resolve_project_item_path(&project_directory_path, &self.project_item_path);
        let source_project_item_ref = ProjectItemRef::new(source_project_item_path.clone());
        let Some(source_project_item) = opened_project.get_project_items().get(&source_project_item_ref) else {
            log::warn!("Cannot rename missing project item: {:?}.", source_project_item_path);

            return ProjectItemsRenameResponse {
                success: false,
                renamed_project_item_path: PathBuf::new(),
            };
        };
        let target_project_item_path =
            match build_target_project_item_path(opened_project, &source_project_item_path, source_project_item, requested_project_item_name) {
                Ok(target_project_item_path) => target_project_item_path,
                Err(error) => {
                    log::error!("{}", error);

                    return ProjectItemsRenameResponse {
                        success: false,
                        renamed_project_item_path: PathBuf::new(),
                    };
                }
            };

        if source_project_item_path != target_project_item_path {
            if let Err(error) = rename_project_item_file_or_directory(&source_project_item_path, &target_project_item_path) {
                log::error!("{}", error);

                return ProjectItemsRenameResponse {
                    success: false,
                    renamed_project_item_path: PathBuf::new(),
                };
            }

            let Some(project_item) = opened_project
                .get_project_items_mut()
                .remove(&source_project_item_ref)
            else {
                log::error!("Renamed project item disappeared before project map update: {:?}.", source_project_item_path);

                return ProjectItemsRenameResponse {
                    success: false,
                    renamed_project_item_path: PathBuf::new(),
                };
            };

            opened_project
                .get_project_items_mut()
                .insert(ProjectItemRef::new(target_project_item_path.clone()), project_item);
            rename_project_item_in_sort_order(opened_project, &project_directory_path, &source_project_item_path, &target_project_item_path);
        }

        let renamed_project_item_ref = ProjectItemRef::new(target_project_item_path.clone());
        let Some(renamed_project_item) = opened_project.get_project_item_mut(&renamed_project_item_ref) else {
            log::warn!("Renamed project item was not found after project map update: {:?}.", target_project_item_path);

            return ProjectItemsRenameResponse {
                success: false,
                renamed_project_item_path: PathBuf::new(),
            };
        };
        let renamed_project_item_display_name =
            if renamed_project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
                target_project_item_path
                    .file_name()
                    .and_then(|file_name| file_name.to_str())
                    .unwrap_or(requested_project_item_name)
                    .to_string()
            } else {
                requested_project_item_name.to_string()
            };

        renamed_project_item.set_field_name(&renamed_project_item_display_name);
        renamed_project_item.set_has_unsaved_changes(true);
        opened_project
            .get_project_info_mut()
            .set_has_unsaved_changes(true);

        if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
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

fn build_target_project_item_path(
    opened_project: &Project,
    source_project_item_path: &Path,
    source_project_item: &ProjectItem,
    requested_project_item_name: &str,
) -> Result<PathBuf, String> {
    let Some(parent_directory_path) = source_project_item_path.parent() else {
        return Err(format!("Failed to resolve parent directory for project item {:?}.", source_project_item_path));
    };

    if source_project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
        return Ok(generate_unique_project_item_directory_path_allowing(
            parent_directory_path,
            requested_project_item_name,
            Some(source_project_item_path),
        ));
    }

    let project_item_file_stem = sanitize_file_name_component(requested_project_item_name, "project_item");

    Ok(generate_unique_project_item_file_path_allowing(
        parent_directory_path,
        opened_project.get_project_items(),
        &project_item_file_stem,
        Some(source_project_item_path),
    ))
}

fn generate_unique_project_item_directory_path_allowing(
    parent_directory_path: &Path,
    requested_project_item_name: &str,
    allowed_existing_project_item_path: Option<&Path>,
) -> PathBuf {
    let requested_directory_name = Path::new(requested_project_item_name)
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .map(str::trim)
        .filter(|file_name| !file_name.is_empty())
        .unwrap_or("project_directory");
    let mut duplicate_sequence_number = 0_u64;

    loop {
        let directory_name = if duplicate_sequence_number == 0 {
            requested_directory_name.to_string()
        } else {
            format!("{}_{}", requested_directory_name, duplicate_sequence_number)
        };
        let target_project_item_path = parent_directory_path.join(directory_name);
        let is_allowed_existing_path = allowed_existing_project_item_path
            .map(|allowed_existing_project_item_path| target_project_item_path == allowed_existing_project_item_path)
            .unwrap_or(false);

        if is_allowed_existing_path || !target_project_item_path.exists() {
            return target_project_item_path;
        }

        duplicate_sequence_number = duplicate_sequence_number.saturating_add(1);
    }
}

fn rename_project_item_file_or_directory(
    source_project_item_path: &Path,
    target_project_item_path: &Path,
) -> Result<(), String> {
    match fs::rename(source_project_item_path, target_project_item_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => Err(format!(
            "Project item rename target unexpectedly existed after conflict resolution: {:?}.",
            target_project_item_path
        )),
        Err(error) => Err(format!(
            "Failed to rename project item from {:?} to {:?}: {}.",
            source_project_item_path, target_project_item_path, error
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemsRenameRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::data_types::built_in_types::u64::data_type_u64::DataTypeU64;
    use squalr_engine_api::structures::projects::project::Project;
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
    use std::fs::{self, File};
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn rename_request_updates_display_name_and_chooses_conflict_free_backing_file() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let mut project = create_project_with_symbol_catalog(temp_directory.path(), ProjectSymbolCatalog::default());
        let source_project_item_relative_path = PathBuf::from(Project::PROJECT_DIR).join("winmine.exe+0x579C.json");
        let source_project_item_absolute_path = temp_directory.path().join(&source_project_item_relative_path);
        let source_project_item_ref = ProjectItemRef::new(source_project_item_absolute_path.clone());
        let source_project_item =
            ProjectItemTypeAddress::new_project_item("winmine.exe+0x579C", 0x579C, "winmine.exe", "", DataTypeU64::get_value_from_primitive(0));
        let colliding_project_item_absolute_path = temp_directory
            .path()
            .join(Project::PROJECT_DIR)
            .join("winmine_exe_0x579C.json");
        let colliding_project_item_ref = ProjectItemRef::new(colliding_project_item_absolute_path.clone());
        let colliding_project_item = ProjectItemTypeAddress::new_project_item("Existing", 0x579C, "winmine.exe", "", DataTypeU64::get_value_from_primitive(0));
        let expected_renamed_project_item_absolute_path = temp_directory
            .path()
            .join(Project::PROJECT_DIR)
            .join("winmine_exe_0x579C_1.json");
        let expected_renamed_project_item_ref = ProjectItemRef::new(expected_renamed_project_item_absolute_path.clone());

        fs::create_dir_all(temp_directory.path().join(Project::PROJECT_DIR)).expect("Expected project items directory to be created.");
        serde_json::to_writer_pretty(
            File::create(&source_project_item_absolute_path).expect("Expected source project item file to be created."),
            &source_project_item,
        )
        .expect("Expected source project item to be written.");
        serde_json::to_writer_pretty(
            File::create(&colliding_project_item_absolute_path).expect("Expected colliding project item file to be created."),
            &colliding_project_item,
        )
        .expect("Expected colliding project item to be written.");

        project
            .get_project_items_mut()
            .insert(source_project_item_ref.clone(), source_project_item);
        project
            .get_project_items_mut()
            .insert(colliding_project_item_ref.clone(), colliding_project_item);

        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());
        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_items_rename_response = ProjectItemsRenameRequest {
            project_item_path: source_project_item_relative_path,
            project_item_name: String::from("winmine_exe_0x579C"),
        }
        .execute(&engine_execution_context);

        assert!(project_items_rename_response.success);
        assert_eq!(
            project_items_rename_response.renamed_project_item_path,
            expected_renamed_project_item_absolute_path
        );
        assert!(!source_project_item_absolute_path.exists());
        assert!(colliding_project_item_absolute_path.exists());
        assert!(expected_renamed_project_item_absolute_path.exists());

        let opened_project_lock = engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project_guard = opened_project_lock
            .read()
            .expect("Expected opened project read lock in test.");
        let opened_project = opened_project_guard
            .as_ref()
            .expect("Expected opened project in test.");
        let updated_project_item = opened_project
            .get_project_items()
            .get(&expected_renamed_project_item_ref)
            .expect("Expected renamed project item at conflict-free path.");

        assert_eq!(updated_project_item.get_field_name(), "winmine_exe_0x579C");
        assert!(
            !opened_project
                .get_project_items()
                .contains_key(&source_project_item_ref)
        );
        assert!(
            opened_project
                .get_project_items()
                .contains_key(&colliding_project_item_ref)
        );
    }
}
