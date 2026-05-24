use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::memory::memory_write_dispatch::dispatch_memory_write_request;
use crate::services::projects::project_symbol_runtime_value_write::{
    ProjectSymbolRuntimeValueWritePlanRequest, build_project_symbol_runtime_value_write_request,
};
use squalr_engine_api::commands::project_symbols::write_value::project_symbols_write_value_request::ProjectSymbolsWriteValueRequest;
use squalr_engine_api::commands::project_symbols::write_value::project_symbols_write_value_response::ProjectSymbolsWriteValueResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;
use std::time::Duration;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsWriteValueRequest {
    type ResponseType = ProjectSymbolsWriteValueResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let opened_project_guard = match opened_project.read() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                let error = format!("Failed to acquire opened project lock for project-symbols write-value command: {}.", error);
                log::error!("{}", error);
                return ProjectSymbolsWriteValueResponse {
                    success: false,
                    error: Some(error),
                };
            }
        };
        let Some(opened_project) = opened_project_guard.as_ref() else {
            let error = String::from("Cannot write a project symbol value without an opened project.");
            log::warn!("{}", error);
            return ProjectSymbolsWriteValueResponse {
                success: false,
                error: Some(error),
            };
        };
        let project_symbol_catalog = opened_project
            .get_project_info()
            .get_project_symbol_catalog()
            .clone();
        drop(opened_project_guard);

        let write_plan_request = ProjectSymbolRuntimeValueWritePlanRequest {
            address: self.address,
            module_name: self.module_name.clone(),
            symbol_type_id: self.symbol_type_id.clone(),
            container_type: self.container_type,
            field_name: self.field_name.clone(),
            anonymous_value_string: self.anonymous_value_string.clone(),
        };
        let memory_write_request =
            match build_project_symbol_runtime_value_write_request(engine_unprivileged_state, &project_symbol_catalog, &write_plan_request) {
                Ok(memory_write_request) => memory_write_request,
                Err(error) => {
                    log::warn!("Project-symbols write-value planning failed: {}", error);
                    return ProjectSymbolsWriteValueResponse {
                        success: false,
                        error: Some(error),
                    };
                }
            };

        match dispatch_memory_write_request(engine_unprivileged_state, memory_write_request, Duration::from_secs(2)) {
            Ok(memory_write_response) if memory_write_response.success => ProjectSymbolsWriteValueResponse { success: true, error: None },
            Ok(_memory_write_response) => {
                let error = String::from("Privileged memory write command failed.");
                log::warn!("{}", error);
                ProjectSymbolsWriteValueResponse {
                    success: false,
                    error: Some(error),
                }
            }
            Err(error) => {
                log::warn!("Project-symbols write-value dispatch failed: {}", error);
                ProjectSymbolsWriteValueResponse {
                    success: false,
                    error: Some(error),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsWriteValueRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::data_values::{
        anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
    };
    use squalr_engine_api::structures::projects::{project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim};
    use std::sync::Arc;

    #[test]
    fn write_value_request_dispatches_memory_write_for_scalar_symbol_claim() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Health"),
                0x1234,
                String::from("u32"),
            )],
        );
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let mock_project_symbols_bindings = MockProjectSymbolsBindings::new();
        let captured_memory_write_requests = mock_project_symbols_bindings.captured_memory_write_requests();
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_project_symbols_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_symbols_write_value_response = ProjectSymbolsWriteValueRequest {
            address: 0x1234,
            module_name: String::new(),
            symbol_type_id: String::from("u32"),
            container_type: ContainerType::None,
            field_name: String::from("value"),
            anonymous_value_string: AnonymousValueString::new(String::from("255"), AnonymousValueStringFormat::Decimal, ContainerType::None),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_write_value_response.success);

        let captured_memory_write_requests = captured_memory_write_requests
            .lock()
            .expect("Expected captured memory write lock.");

        assert_eq!(captured_memory_write_requests.len(), 1);
        assert_eq!(captured_memory_write_requests[0].address, 0x1234);
        assert_eq!(captured_memory_write_requests[0].module_name, "");
        assert_eq!(captured_memory_write_requests[0].value, 255_u32.to_ne_bytes());
    }
}
