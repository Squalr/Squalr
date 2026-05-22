use crate::command_executors::project_items::project_item_sort_order::rename_project_item_in_sort_order;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::projects::project_item_file_mutation::{
    generate_unique_project_item_file_path_allowing, resolve_project_item_path, sanitize_file_name_component,
};
use squalr_engine_api::commands::project_items::update_details::project_items_update_details_request::ProjectItemsUpdateDetailsRequest;
use squalr_engine_api::commands::project_items::update_details::project_items_update_details_response::{
    ProjectItemsUpdateDetailsPathChange, ProjectItemsUpdateDetailsResponse,
};
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::details::DetailsFieldSource;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_api::structures::projects::project_items::details::ProjectItemDetailsEditApplier;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsUpdateDetailsRequest {
    type ResponseType = ProjectItemsUpdateDetailsResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.project_item_paths.is_empty() {
            return ProjectItemsUpdateDetailsResponse {
                success: true,
                updated_project_item_count: 0,
                path_changes: Vec::new(),
                error: None,
            };
        }

        let (details_field_source, details_value) = match self.resolve_details_update() {
            Ok(details_update) => details_update,
            Err(error) => {
                log::warn!("{}", error);
                return ProjectItemsUpdateDetailsResponse {
                    success: false,
                    updated_project_item_count: 0,
                    path_changes: Vec::new(),
                    error: Some(error),
                };
            }
        };
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project_lock.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                let error = format!("Failed to acquire opened project lock for project item details update: {}.", error);
                log::error!("{}", error);

                return ProjectItemsUpdateDetailsResponse {
                    success: false,
                    updated_project_item_count: 0,
                    path_changes: Vec::new(),
                    error: Some(error),
                };
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            let error = String::from("Cannot update project item details without an opened project.");
            log::warn!("{}", error);

            return ProjectItemsUpdateDetailsResponse {
                success: false,
                updated_project_item_count: 0,
                path_changes: Vec::new(),
                error: Some(error),
            };
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            let error = String::from("Failed to resolve opened project directory for project item details update.");
            log::error!("{}", error);

            return ProjectItemsUpdateDetailsResponse {
                success: false,
                updated_project_item_count: 0,
                path_changes: Vec::new(),
                error: Some(error),
            };
        };
        let mut updated_project_item_count = 0_u64;
        let mut path_changes = Vec::new();

        for project_item_path in &self.project_item_paths {
            let resolved_project_item_path = resolve_project_item_path(&project_directory_path, project_item_path);
            let project_item_ref = ProjectItemRef::new(resolved_project_item_path.clone());
            let should_align_file_name;

            {
                let Some(project_item) = opened_project.get_project_item_mut(&project_item_ref) else {
                    log::warn!(
                        "Cannot update project item details, project item was not found: {:?}.",
                        resolved_project_item_path
                    );
                    continue;
                };

                match ProjectItemDetailsEditApplier::apply_update(project_item, &details_field_source, &details_value) {
                    Ok(true) => {
                        project_item.set_has_unsaved_changes(true);
                        should_align_file_name = should_align_project_item_file_name(&details_field_source, project_item);
                        updated_project_item_count = updated_project_item_count.saturating_add(1);
                    }
                    Ok(false) => continue,
                    Err(error) => {
                        log::warn!("Failed to apply project item details update: {}", error);
                        continue;
                    }
                }
            }

            if should_align_file_name {
                match align_project_item_file_name(opened_project, &project_directory_path, &resolved_project_item_path) {
                    Ok(Some(updated_project_item_path)) => path_changes.push(ProjectItemsUpdateDetailsPathChange {
                        previous_project_item_path: resolved_project_item_path,
                        updated_project_item_path,
                    }),
                    Ok(None) => {}
                    Err(error) => log::warn!("{}", error),
                }
            }
        }

        if updated_project_item_count == 0 {
            return ProjectItemsUpdateDetailsResponse {
                success: true,
                updated_project_item_count,
                path_changes,
                error: None,
            };
        }

        opened_project
            .get_project_info_mut()
            .set_has_unsaved_changes(true);
        if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
            let error = format!("Failed to save project after project item details update: {}.", error);
            log::error!("{}", error);

            return ProjectItemsUpdateDetailsResponse {
                success: false,
                updated_project_item_count,
                path_changes,
                error: Some(error),
            };
        }
        drop(opened_project_guard);

        project_manager.notify_project_items_changed();

        ProjectItemsUpdateDetailsResponse {
            success: true,
            updated_project_item_count,
            path_changes,
            error: None,
        }
    }
}

fn should_align_project_item_file_name(
    details_field_source: &DetailsFieldSource,
    project_item: &ProjectItem,
) -> bool {
    let DetailsFieldSource::ProjectItemProperty { property_name } = details_field_source else {
        return false;
    };

    property_name == ProjectItem::PROPERTY_NAME && project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID
}

fn align_project_item_file_name(
    opened_project: &mut Project,
    project_directory_path: &Path,
    source_project_item_path: &Path,
) -> Result<Option<PathBuf>, String> {
    let source_project_item_ref = ProjectItemRef::new(source_project_item_path.to_path_buf());
    let Some(project_item) = opened_project.get_project_items().get(&source_project_item_ref) else {
        return Ok(None);
    };
    let display_name = project_item.get_field_name().to_string();
    let Some(parent_directory_path) = source_project_item_path.parent() else {
        return Err(format!(
            "Cannot align project item file name without a parent directory: {:?}.",
            source_project_item_path
        ));
    };
    let project_item_file_stem = sanitize_file_name_component(&display_name, "project_item");
    let mut rename_attempt_count = 0_u64;

    loop {
        let target_project_item_path = generate_unique_project_item_file_path_allowing(
            parent_directory_path,
            opened_project.get_project_items(),
            &project_item_file_stem,
            Some(source_project_item_path),
        );

        if target_project_item_path == source_project_item_path {
            return Ok(None);
        }

        match fs::rename(source_project_item_path, &target_project_item_path) {
            Ok(()) => {
                remap_project_item_path(opened_project, project_directory_path, source_project_item_path, &target_project_item_path)?;
                return Ok(Some(target_project_item_path));
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                rename_attempt_count = rename_attempt_count.saturating_add(1);

                if rename_attempt_count >= 1024 {
                    return Err(format!(
                        "Could not find a free project item file name while aligning {:?} to display name `{}`.",
                        source_project_item_path, display_name
                    ));
                }
            }
            Err(error) => {
                return Err(format!(
                    "Could not align project item file name from {:?} to {:?}: {}.",
                    source_project_item_path, target_project_item_path, error
                ));
            }
        }
    }
}

fn remap_project_item_path(
    opened_project: &mut Project,
    project_directory_path: &Path,
    source_project_item_path: &Path,
    target_project_item_path: &Path,
) -> Result<(), String> {
    let source_project_item_ref = ProjectItemRef::new(source_project_item_path.to_path_buf());
    let target_project_item_ref = ProjectItemRef::new(target_project_item_path.to_path_buf());
    let Some(project_item) = opened_project
        .get_project_items_mut()
        .remove(&source_project_item_ref)
    else {
        return Err(format!(
            "Renamed project item was not found in the opened project: {:?}.",
            source_project_item_path
        ));
    };

    opened_project
        .get_project_items_mut()
        .insert(target_project_item_ref, project_item);
    rename_project_item_in_sort_order(opened_project, project_directory_path, source_project_item_path, target_project_item_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ProjectItemsUpdateDetailsRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::data_types::built_in_types::u64::data_type_u64::DataTypeU64;
    use squalr_engine_api::structures::details::{DetailsFieldSource, DetailsValue};
    use squalr_engine_api::structures::projects::project::Project;
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
    use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
    use std::fs::{self, File};
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn update_details_request_persists_project_item_property() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let mut project = create_project_with_symbol_catalog(temp_directory.path(), ProjectSymbolCatalog::default());
        let project_item_relative_path = PathBuf::from(Project::PROJECT_DIR).join("health.json");
        let project_item_absolute_path = temp_directory.path().join(&project_item_relative_path);
        let project_item_ref = ProjectItemRef::new(project_item_absolute_path.clone());
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));

        project
            .get_project_items_mut()
            .insert(project_item_ref.clone(), project_item);

        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());
        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_items_update_details_response = ProjectItemsUpdateDetailsRequest::from_details_update(
            vec![project_item_relative_path],
            DetailsFieldSource::ProjectItemProperty {
                property_name: ProjectItem::PROPERTY_ICON_ID.to_string(),
            },
            DetailsValue::Text(String::from("u64")),
        )
        .execute(&engine_execution_context);

        assert!(project_items_update_details_response.success);
        assert_eq!(project_items_update_details_response.updated_project_item_count, 1);
        assert!(project_items_update_details_response.path_changes.is_empty());

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
            .get(&project_item_ref)
            .expect("Expected updated project item.");

        assert_eq!(updated_project_item.get_field_icon_id(), "u64");
        assert!(updated_project_item.get_has_unsaved_changes());
    }

    #[test]
    fn update_details_request_updates_display_name_and_renames_project_item_file_without_conflict() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let mut project = create_project_with_symbol_catalog(temp_directory.path(), ProjectSymbolCatalog::default());
        let source_project_item_relative_path = PathBuf::from(Project::PROJECT_DIR).join("winmine.exe+0x579C.json");
        let source_project_item_absolute_path = temp_directory.path().join(&source_project_item_relative_path);
        let source_project_item_ref = ProjectItemRef::new(source_project_item_absolute_path.clone());
        let source_project_item =
            ProjectItemTypeAddress::new_project_item("winmine.exe+0x579C", 0x579C, "winmine.exe", "", DataTypeU64::get_value_from_primitive(0));
        let colliding_project_item_relative_path = PathBuf::from(Project::PROJECT_DIR).join("winmine_exe_0x579C.json");
        let colliding_project_item_absolute_path = temp_directory
            .path()
            .join(&colliding_project_item_relative_path);
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
        let project_items_update_details_response = ProjectItemsUpdateDetailsRequest::from_details_update(
            vec![source_project_item_relative_path],
            DetailsFieldSource::ProjectItemProperty {
                property_name: ProjectItem::PROPERTY_NAME.to_string(),
            },
            DetailsValue::Text(String::from("winmine_exe_0x579C")),
        )
        .execute(&engine_execution_context);

        assert!(project_items_update_details_response.success);
        assert_eq!(project_items_update_details_response.updated_project_item_count, 1);
        assert_eq!(project_items_update_details_response.path_changes.len(), 1);
        assert_eq!(
            project_items_update_details_response.path_changes[0].previous_project_item_path,
            source_project_item_absolute_path
        );
        assert_eq!(
            project_items_update_details_response.path_changes[0].updated_project_item_path,
            expected_renamed_project_item_absolute_path
        );
        assert!(!source_project_item_absolute_path.exists());
        assert!(expected_renamed_project_item_absolute_path.exists());
        assert!(colliding_project_item_absolute_path.exists());

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
            .expect("Expected updated source project item to be moved to conflict-free path.");

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
