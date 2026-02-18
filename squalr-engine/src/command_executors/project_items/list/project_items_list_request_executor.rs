use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_response::ProjectItemsListResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

impl UnprivilegedCommandRequestExecutor for ProjectItemsListRequest {
    type ResponseType = ProjectItemsListResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();

        match opened_project_lock.read() {
            Ok(opened_project_guard) => {
                let opened_project = match opened_project_guard.as_ref() {
                    Some(opened_project) => opened_project,
                    None => return ProjectItemsListResponse::default(),
                };
                let opened_project_root = opened_project.get_project_root().cloned();
                let mut opened_project_items = opened_project
                    .get_project_items()
                    .iter()
                    .map(|(project_item_ref, project_item)| (project_item_ref.clone(), project_item.clone()))
                    .collect::<Vec<(ProjectItemRef, ProjectItem)>>();

                refresh_project_item_display_values(engine_unprivileged_state, &mut opened_project_items);

                ProjectItemsListResponse {
                    opened_project_info: Some(opened_project.get_project_info().clone()),
                    opened_project_root,
                    opened_project_items,
                }
            }
            Err(error) => {
                log::error!("Error obtaining opened project lock for list command: {}", error);
                ProjectItemsListResponse::default()
            }
        }
    }
}

fn refresh_project_item_display_values(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    opened_project_items: &mut [(ProjectItemRef, ProjectItem)],
) {
    let symbol_registry = SymbolRegistry::get_instance();

    for (_, project_item) in opened_project_items.iter_mut() {
        if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            continue;
        }

        let address = ProjectItemTypeAddress::get_field_address(project_item);
        let module_name = ProjectItemTypeAddress::get_field_module(project_item);
        let Some(symbolic_struct_reference) = ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(project_item) else {
            ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(project_item, "");
            continue;
        };
        let symbolic_struct_namespace = symbolic_struct_reference
            .get_symbolic_struct_namespace()
            .to_string();
        let Some(symbolic_struct_definition) = symbol_registry.get(&symbolic_struct_namespace) else {
            ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(project_item, "");
            continue;
        };

        let Some(memory_read_response) = dispatch_memory_read_request(engine_unprivileged_state, address, &module_name, symbolic_struct_definition.as_ref())
        else {
            ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(project_item, "");
            continue;
        };

        if !memory_read_response.success {
            ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(project_item, "");
            continue;
        }

        let first_read_field_data_value = memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field| valued_struct_field.get_data_value());
        let Some(first_read_field_data_value) = first_read_field_data_value else {
            ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(project_item, "");
            continue;
        };

        let default_anonymous_value_string_format = symbol_registry.get_default_anonymous_value_string_format(first_read_field_data_value.get_data_type_ref());
        let freeze_display_value = symbol_registry
            .anonymize_value(first_read_field_data_value, default_anonymous_value_string_format)
            .map(|anonymous_value_string| anonymous_value_string.get_anonymous_value_string().to_string())
            .unwrap_or_default();
        ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(project_item, &freeze_display_value);
    }
}

fn dispatch_memory_read_request(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    address: u64,
    module_name: &str,
    symbolic_struct_definition: &SymbolicStructDefinition,
) -> Option<MemoryReadResponse> {
    let memory_read_request = MemoryReadRequest {
        address,
        module_name: module_name.to_string(),
        symbolic_struct_definition: symbolic_struct_definition.clone(),
        suppress_logging: true,
    };
    let memory_read_command = memory_read_request.to_engine_command();
    let (memory_read_response_sender, memory_read_response_receiver) = mpsc::channel();

    let dispatch_result = match engine_unprivileged_state.get_bindings().read() {
        Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
            memory_read_command,
            Box::new(move |engine_response| {
                let conversion_result = match MemoryReadResponse::from_engine_response(engine_response) {
                    Ok(memory_read_response) => Ok(memory_read_response),
                    Err(unexpected_response) => Err(format!(
                        "Unexpected response variant for project-items list memory read request: {:?}",
                        unexpected_response
                    )),
                };
                let _ = memory_read_response_sender.send(conversion_result);
            }),
        ),
        Err(error) => {
            log::error!("Failed to acquire engine bindings lock for project-items list memory read request: {}", error);
            return None;
        }
    };

    if let Err(error) = dispatch_result {
        log::error!("Failed to dispatch project-items list memory read request: {}", error);
        return None;
    }

    match memory_read_response_receiver.recv_timeout(Duration::from_secs(2)) {
        Ok(Ok(memory_read_response)) => Some(memory_read_response),
        Ok(Err(error)) => {
            log::error!("Failed to convert project-items list memory read response: {}", error);
            None
        }
        Err(error) => {
            log::error!("Timed out waiting for project-items list memory read response: {}", error);
            None
        }
    }
}
