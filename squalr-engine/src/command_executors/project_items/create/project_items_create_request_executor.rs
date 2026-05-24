use crate::command_executors::project_items::project_item_sort_order::append_project_items_to_sort_order;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::projects::project_item_file_mutation::{
    create_placeholder_file, generate_unique_project_item_file_path, resolve_project_file_parent_directory_path, resolve_project_item_path,
    sanitize_file_name_component,
};
use squalr_engine_api::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest;
use squalr_engine_api::commands::project_items::create::project_items_create_response::ProjectItemsCreateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_address_target::ProjectItemAddressTarget,
};
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
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
        if self.is_directory {
            return create_directory_item(self, engine_unprivileged_state);
        }

        create_address_item(self, engine_unprivileged_state)
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
    let parent_directory_path = resolve_project_file_parent_directory_path(&project_directory_path, &project_items_create_request.parent_directory_path);
    let project_item_file_stem = sanitize_file_name_component(&project_items_create_request.project_item_name, "project_item");
    let created_project_item_path = generate_unique_project_item_file_path(&parent_directory_path, opened_project.get_project_items(), &project_item_file_stem);
    let project_item_ref = ProjectItemRef::new(created_project_item_path.clone());
    let address = project_items_create_request.address.unwrap_or(0);
    let module_name = project_items_create_request
        .module_name
        .clone()
        .unwrap_or_default();
    let mut project_item = ProjectItemTypeAddress::new_project_item(
        &project_items_create_request.project_item_name,
        address,
        &module_name,
        "",
        DataTypeU8::get_value_from_primitive(0),
    );
    if let Some(pointer_offsets) = project_items_create_request.pointer_offsets.clone() {
        ProjectItemTypeAddress::set_address_target(
            &mut project_item,
            ProjectItemAddressTarget::new(module_name.clone(), pointer_offsets, Default::default()),
        );
    }
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

#[cfg(test)]
mod tests {
    use super::ProjectItemsCreateRequest;
    use crate::command_executors::project_symbols::test_support::{MockProjectSymbolsBindings, create_engine_unprivileged_state};
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::memory::pointer_chain_segment::PointerChainSegment;
    use squalr_engine_api::structures::projects::{
        project::Project, project_info::ProjectInfo, project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress,
        project_items::project_item_ref::ProjectItemRef, project_manifest::ProjectManifest,
    };
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn create_address_item_uses_explicit_pointer_offsets_when_provided() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_file_path = temp_directory.path().join(Project::PROJECT_FILE);
        let project_root_path = temp_directory.path().join(Project::PROJECT_DIR);
        let project_root_ref = ProjectItemRef::new(project_root_path);
        let project_info = ProjectInfo::new(project_file_path, None, ProjectManifest::default());
        let project = Project::new(project_info, std::collections::HashMap::new(), project_root_ref);
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_items_create_response = ProjectItemsCreateRequest {
            parent_directory_path: PathBuf::new(),
            project_item_name: String::from("Timer"),
            is_directory: false,
            address: Some(0x579C),
            module_name: Some(String::from("winmine.exe")),
            data_type_id: Some(String::from("u32")),
            pointer_offsets: Some(vec![PointerChainSegment::Symbol(String::from("Timer"))]),
        }
        .execute(&engine_execution_context);

        assert!(project_items_create_response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected project to load after address create.");
        let created_project_item = loaded_project
            .get_project_items()
            .values()
            .find(|project_item| project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID)
            .expect("Expected created address item.");
        let mut created_project_item = created_project_item.clone();

        assert_eq!(
            ProjectItemTypeAddress::get_address_target(&mut created_project_item).get_pointer_offsets(),
            &[PointerChainSegment::Symbol(String::from("Timer"))]
        );
    }
}
