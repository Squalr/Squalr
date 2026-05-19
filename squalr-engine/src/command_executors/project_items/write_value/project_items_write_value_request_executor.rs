use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::memory::memory_write_dispatch::dispatch_memory_write_request;
use crate::services::projects::project_item_file_mutation::resolve_project_item_path;
use crate::services::projects::project_item_runtime_value_write::{ProjectItemRuntimeValueWritePlanRequest, build_project_item_runtime_value_write_request};
use squalr_engine_api::commands::project_items::write_value::project_items_write_value_request::ProjectItemsWriteValueRequest;
use squalr_engine_api::commands::project_items::write_value::project_items_write_value_response::ProjectItemsWriteValueResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use std::sync::Arc;
use std::time::Duration;

impl UnprivilegedCommandRequestExecutor for ProjectItemsWriteValueRequest {
    type ResponseType = ProjectItemsWriteValueResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let opened_project_guard = match opened_project.read() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                let error = format!("Failed to acquire opened project lock for project-items write-value command: {}.", error);
                log::error!("{}", error);
                return ProjectItemsWriteValueResponse {
                    success: false,
                    error: Some(error),
                };
            }
        };
        let Some(opened_project) = opened_project_guard.as_ref() else {
            let error = String::from("Cannot write a project item value without an opened project.");
            log::warn!("{}", error);
            return ProjectItemsWriteValueResponse {
                success: false,
                error: Some(error),
            };
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            let error = String::from("Cannot write a project item value without an opened project directory.");
            log::warn!("{}", error);
            return ProjectItemsWriteValueResponse {
                success: false,
                error: Some(error),
            };
        };
        let resolved_project_item_path = resolve_project_item_path(&project_directory_path, &self.project_item_path);
        let project_item_ref = ProjectItemRef::new(resolved_project_item_path.clone());
        let Some(project_item) = opened_project
            .get_project_items()
            .get(&project_item_ref)
            .cloned()
        else {
            let error = format!("Unable to find project item `{}`.", resolved_project_item_path.display());
            log::warn!("{}", error);
            return ProjectItemsWriteValueResponse {
                success: false,
                error: Some(error),
            };
        };
        let project_symbol_catalog = opened_project
            .get_project_info()
            .get_project_symbol_catalog()
            .clone();
        drop(opened_project_guard);

        let write_plan_request = ProjectItemRuntimeValueWritePlanRequest {
            field_name: self.field_name.clone(),
            anonymous_value_string: self.anonymous_value_string.clone(),
        };
        let memory_write_request =
            match build_project_item_runtime_value_write_request(engine_unprivileged_state, &project_symbol_catalog, &project_item, &write_plan_request) {
                Ok(memory_write_request) => memory_write_request,
                Err(error) => {
                    log::warn!("Project-items write-value planning failed: {}", error);
                    return ProjectItemsWriteValueResponse {
                        success: false,
                        error: Some(error),
                    };
                }
            };

        match dispatch_memory_write_request(engine_unprivileged_state, memory_write_request, Duration::from_secs(2)) {
            Ok(memory_write_response) if memory_write_response.success => ProjectItemsWriteValueResponse { success: true, error: None },
            Ok(_memory_write_response) => {
                let error = String::from("Privileged memory write command failed.");
                log::warn!("{}", error);
                ProjectItemsWriteValueResponse {
                    success: false,
                    error: Some(error),
                }
            }
            Err(error) => {
                log::warn!("Project-items write-value dispatch failed: {}", error);
                ProjectItemsWriteValueResponse {
                    success: false,
                    error: Some(error),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemsWriteValueRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::data_types::built_in_types::u32::data_type_u32::DataTypeU32;
    use squalr_engine_api::structures::data_values::{
        anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
    };
    use squalr_engine_api::structures::projects::project::Project;
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn write_value_request_dispatches_memory_write_for_address_item() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let mut project = create_project_with_symbol_catalog(temp_directory.path(), ProjectSymbolCatalog::default());
        let project_item_relative_path = PathBuf::from(Project::PROJECT_DIR).join("health.json");
        let project_item_absolute_path = temp_directory.path().join(&project_item_relative_path);
        let project_item_ref = ProjectItemRef::new(project_item_absolute_path.clone());
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "", "", DataTypeU32::get_value_from_primitive(0));

        project
            .get_project_items_mut()
            .insert(project_item_ref, project_item);

        let mock_project_symbols_bindings = MockProjectSymbolsBindings::new();
        let captured_memory_write_requests = mock_project_symbols_bindings.captured_memory_write_requests();
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_project_symbols_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_items_write_value_response = ProjectItemsWriteValueRequest {
            project_item_path: project_item_relative_path,
            field_name: String::from("value"),
            anonymous_value_string: AnonymousValueString::new(String::from("255"), AnonymousValueStringFormat::Decimal, ContainerType::None),
        }
        .execute(&engine_execution_context);

        assert!(project_items_write_value_response.success);

        let captured_memory_write_requests = captured_memory_write_requests
            .lock()
            .expect("Expected captured memory write lock.");

        assert_eq!(captured_memory_write_requests.len(), 1);
        assert_eq!(captured_memory_write_requests[0].address, 0x1234);
        assert_eq!(captured_memory_write_requests[0].value, 255_u32.to_ne_bytes());
    }
}
