use crate::command_executors::project_items::project_item_sort_order::append_project_items_to_sort_order;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest;
use squalr_engine_api::commands::project_items::create::project_items_create_response::ProjectItemsCreateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
    project_item_type_pointer::ProjectItemTypePointer, project_item_type_symbol_ref::ProjectItemTypeSymbolRef,
};
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsCreateRequest {
    type ResponseType = ProjectItemsCreateResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.project_item_type == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
            return create_directory_item(self, engine_unprivileged_state);
        }

        if self.project_item_type == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            return create_pointer_item(self, engine_unprivileged_state);
        }

        if self.project_item_type == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            return create_address_item(self, engine_unprivileged_state);
        }

        if self.project_item_type == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
            return create_symbol_ref_item(self, engine_unprivileged_state);
        }

        log::error!(
            "Unsupported project item type for create command: {}. Supported types: '{}', '{}', '{}', '{}'.",
            self.project_item_type,
            ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID,
            ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID,
            ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID,
            ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID,
        );

        ProjectItemsCreateResponse {
            success: false,
            created_project_item_path: PathBuf::new(),
        }
    }
}

fn create_directory_item(
    project_items_create_request: &ProjectItemsCreateRequest,
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
) -> ProjectItemsCreateResponse {
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
    let parent_directory_path = resolve_project_item_path(&project_directory_path, &project_items_create_request.parent_directory_path);
    let created_project_item_path = parent_directory_path.join(&project_items_create_request.project_item_name);

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

    let Some(reloaded_opened_project) = opened_project_guard.as_mut() else {
        return ProjectItemsCreateResponse {
            success: false,
            created_project_item_path: PathBuf::new(),
        };
    };

    append_project_items_to_sort_order(reloaded_opened_project, &project_directory_path, &[created_project_item_path.clone()]);

    if let Err(error) = reloaded_opened_project.save_to_path(&project_directory_path, false) {
        log::error!("Failed to save project after directory create operation: {}", error);

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

fn create_address_item(
    project_items_create_request: &ProjectItemsCreateRequest,
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
) -> ProjectItemsCreateResponse {
    let data_type_id = project_items_create_request
        .data_type_id
        .clone()
        .filter(|data_type_id| !data_type_id.trim().is_empty())
        .unwrap_or_else(|| String::from("u8"));
    let project_manager = engine_unprivileged_state.get_project_manager();
    let opened_project = project_manager.get_opened_project();
    let mut opened_project_guard = match opened_project.write() {
        Ok(opened_project_guard) => opened_project_guard,
        Err(error) => {
            log::error!("Failed to acquire opened project lock for address create command: {}", error);

            return ProjectItemsCreateResponse {
                success: false,
                created_project_item_path: PathBuf::new(),
            };
        }
    };
    let opened_project = match opened_project_guard.as_mut() {
        Some(opened_project) => opened_project,
        None => {
            log::warn!("Cannot create address project items without an opened project.");

            return ProjectItemsCreateResponse {
                success: false,
                created_project_item_path: PathBuf::new(),
            };
        }
    };
    let project_directory_path = match opened_project.get_project_info().get_project_directory() {
        Some(project_directory_path) => project_directory_path,
        None => {
            log::error!("Failed to resolve opened project directory for address create operation.");

            return ProjectItemsCreateResponse {
                success: false,
                created_project_item_path: PathBuf::new(),
            };
        }
    };
    let project_root_directory_path = project_directory_path.join(Project::PROJECT_DIR);
    let requested_parent_directory_path = resolve_project_item_path(&project_directory_path, &project_items_create_request.parent_directory_path);
    let parent_directory_path = if requested_parent_directory_path.starts_with(&project_root_directory_path) {
        requested_parent_directory_path
    } else {
        project_root_directory_path
    };
    let project_item_file_stem = sanitize_file_name_component(&project_items_create_request.project_item_name);
    let created_project_item_path = generate_unique_project_item_file_path(&parent_directory_path, opened_project.get_project_items(), &project_item_file_stem);
    let project_item_ref = ProjectItemRef::new(created_project_item_path.clone());
    let requested_address = project_items_create_request.address.unwrap_or(0);
    let requested_module_name = project_items_create_request
        .module_name
        .as_deref()
        .unwrap_or("");
    let mut project_item = ProjectItemTypeAddress::new_project_item(
        &project_items_create_request.project_item_name,
        requested_address,
        requested_module_name,
        "",
        DataTypeU8::get_value_from_primitive(0),
    );
    ProjectItemTypeAddress::set_field_symbolic_struct_definition_reference(&mut project_item, &data_type_id);

    opened_project
        .get_project_items_mut()
        .insert(project_item_ref, project_item);

    if let Err(error) = create_placeholder_file(&created_project_item_path) {
        log::error!("Failed creating address project item placeholder file: {}", error);

        return ProjectItemsCreateResponse {
            success: false,
            created_project_item_path: PathBuf::new(),
        };
    }

    append_project_items_to_sort_order(opened_project, &project_directory_path, &[created_project_item_path.clone()]);

    if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
        log::error!("Failed to save project after address create operation: {}", error);

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

fn create_pointer_item(
    project_items_create_request: &ProjectItemsCreateRequest,
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
) -> ProjectItemsCreateResponse {
    let Some(pointer) = project_items_create_request.pointer.as_ref() else {
        log::error!("Pointer project item creation requires pointer-chain data.");

        return ProjectItemsCreateResponse {
            success: false,
            created_project_item_path: PathBuf::new(),
        };
    };
    let data_type_id = project_items_create_request
        .data_type_id
        .clone()
        .filter(|data_type_id| !data_type_id.trim().is_empty())
        .unwrap_or_else(|| String::from("u8"));
    let project_manager = engine_unprivileged_state.get_project_manager();
    let opened_project = project_manager.get_opened_project();
    let mut opened_project_guard = match opened_project.write() {
        Ok(opened_project_guard) => opened_project_guard,
        Err(error) => {
            log::error!("Failed to acquire opened project lock for pointer create command: {}", error);

            return ProjectItemsCreateResponse {
                success: false,
                created_project_item_path: PathBuf::new(),
            };
        }
    };
    let opened_project = match opened_project_guard.as_mut() {
        Some(opened_project) => opened_project,
        None => {
            log::warn!("Cannot create pointer project items without an opened project.");

            return ProjectItemsCreateResponse {
                success: false,
                created_project_item_path: PathBuf::new(),
            };
        }
    };
    let project_directory_path = match opened_project.get_project_info().get_project_directory() {
        Some(project_directory_path) => project_directory_path,
        None => {
            log::error!("Failed to resolve opened project directory for pointer create operation.");

            return ProjectItemsCreateResponse {
                success: false,
                created_project_item_path: PathBuf::new(),
            };
        }
    };
    let project_root_directory_path = project_directory_path.join(Project::PROJECT_DIR);
    let requested_parent_directory_path = resolve_project_item_path(&project_directory_path, &project_items_create_request.parent_directory_path);
    let parent_directory_path = if requested_parent_directory_path.starts_with(&project_root_directory_path) {
        requested_parent_directory_path
    } else {
        project_root_directory_path
    };
    let project_item_file_stem = sanitize_file_name_component(&project_items_create_request.project_item_name);
    let created_project_item_path = generate_unique_project_item_file_path(&parent_directory_path, opened_project.get_project_items(), &project_item_file_stem);
    let project_item_ref = ProjectItemRef::new(created_project_item_path.clone());
    let project_item = ProjectItemTypePointer::new_project_item(&project_items_create_request.project_item_name, pointer, "", &data_type_id);

    opened_project
        .get_project_items_mut()
        .insert(project_item_ref, project_item);

    if let Err(error) = create_placeholder_file(&created_project_item_path) {
        log::error!("Failed creating pointer project item placeholder file: {}", error);

        return ProjectItemsCreateResponse {
            success: false,
            created_project_item_path: PathBuf::new(),
        };
    }

    append_project_items_to_sort_order(opened_project, &project_directory_path, &[created_project_item_path.clone()]);

    if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
        log::error!("Failed to save project after pointer create operation: {}", error);

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

fn create_symbol_ref_item(
    project_items_create_request: &ProjectItemsCreateRequest,
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
) -> ProjectItemsCreateResponse {
    let project_manager = engine_unprivileged_state.get_project_manager();
    let opened_project = project_manager.get_opened_project();
    let mut opened_project_guard = match opened_project.write() {
        Ok(opened_project_guard) => opened_project_guard,
        Err(error) => {
            log::error!("Failed to acquire opened project lock for symbol-ref create command: {}", error);

            return ProjectItemsCreateResponse {
                success: false,
                created_project_item_path: PathBuf::new(),
            };
        }
    };
    let opened_project = match opened_project_guard.as_mut() {
        Some(opened_project) => opened_project,
        None => {
            log::warn!("Cannot create symbol-ref project items without an opened project.");

            return ProjectItemsCreateResponse {
                success: false,
                created_project_item_path: PathBuf::new(),
            };
        }
    };
    let project_directory_path = match opened_project.get_project_info().get_project_directory() {
        Some(project_directory_path) => project_directory_path,
        None => {
            log::error!("Failed to resolve opened project directory for symbol-ref create operation.");

            return ProjectItemsCreateResponse {
                success: false,
                created_project_item_path: PathBuf::new(),
            };
        }
    };
    let project_root_directory_path = project_directory_path.join(Project::PROJECT_DIR);
    let requested_parent_directory_path = resolve_project_item_path(&project_directory_path, &project_items_create_request.parent_directory_path);
    let parent_directory_path = if requested_parent_directory_path.starts_with(&project_root_directory_path) {
        requested_parent_directory_path
    } else {
        project_root_directory_path
    };
    let project_item_file_stem = sanitize_file_name_component(&project_items_create_request.project_item_name);
    let created_project_item_path = generate_unique_project_item_file_path(&parent_directory_path, opened_project.get_project_items(), &project_item_file_stem);
    let project_item_ref = ProjectItemRef::new(created_project_item_path.clone());
    let project_item = ProjectItemTypeSymbolRef::new_project_item(&project_items_create_request.project_item_name, "", "");

    opened_project
        .get_project_items_mut()
        .insert(project_item_ref, project_item);

    if let Err(error) = create_placeholder_file(&created_project_item_path) {
        log::error!("Failed creating symbol-ref project item placeholder file: {}", error);

        return ProjectItemsCreateResponse {
            success: false,
            created_project_item_path: PathBuf::new(),
        };
    }

    append_project_items_to_sort_order(opened_project, &project_directory_path, &[created_project_item_path.clone()]);

    if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
        log::error!("Failed to save project after symbol-ref create operation: {}", error);

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

fn generate_unique_project_item_file_path(
    parent_directory_path: &Path,
    project_items: &std::collections::HashMap<ProjectItemRef, squalr_engine_api::structures::projects::project_items::project_item::ProjectItem>,
    project_item_file_stem: &str,
) -> PathBuf {
    let mut duplicate_sequence_number = 0_u64;

    loop {
        let project_item_file_name = if duplicate_sequence_number == 0 {
            format!("{}.{}", project_item_file_stem, Project::PROJECT_ITEM_EXTENSION.trim_start_matches('.'))
        } else {
            format!(
                "{}_{}.{}",
                project_item_file_stem,
                duplicate_sequence_number,
                Project::PROJECT_ITEM_EXTENSION.trim_start_matches('.')
            )
        };
        let project_item_absolute_path = parent_directory_path.join(project_item_file_name);
        let project_item_ref = ProjectItemRef::new(project_item_absolute_path.clone());

        if project_items.contains_key(&project_item_ref) {
            duplicate_sequence_number = duplicate_sequence_number.saturating_add(1);
            continue;
        }

        return project_item_absolute_path;
    }
}

fn create_placeholder_file(file_path: &Path) -> Result<(), String> {
    if let Some(parent_path) = file_path.parent() {
        fs::create_dir_all(parent_path).map_err(|error| format!("Failed creating project item parent directory {:?}: {}", parent_path, error))?;
    }

    if !file_path.exists() {
        File::create(file_path).map_err(|error| format!("Failed creating project item file {:?}: {}", file_path, error))?;
    }

    Ok(())
}

fn sanitize_file_name_component(file_name_component: &str) -> String {
    let mut sanitized_component = String::with_capacity(file_name_component.len());
    let mut previous_character_was_underscore = false;

    for name_character in file_name_component.chars() {
        let mapped_character = if name_character.is_ascii_alphanumeric() { name_character } else { '_' };

        if mapped_character == '_' {
            if previous_character_was_underscore {
                continue;
            }

            previous_character_was_underscore = true;
        } else {
            previous_character_was_underscore = false;
        }

        sanitized_component.push(mapped_character);
    }

    let trimmed_component = sanitized_component.trim_matches('_');

    if trimmed_component.is_empty() {
        String::from("project_item")
    } else {
        trimmed_component.to_string()
    }
}
