use squalr_engine::services::projects::project_item_symbol_resolution::{resolve_project_item_runtime_value_target, resolve_project_item_struct_layout_id};
use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType};
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Clone)]
pub struct ProjectItemValueEditContext {
    pub project_item_name: String,
    pub value_field_name: String,
    pub validation_data_type_ref: DataTypeRef,
    pub initial_value_edit: AnonymousValueString,
}

impl ProjectItemValueEditContext {
    pub fn build(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Option<Self> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();
        let value_field_name = if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypePointer::PROPERTY_FREEZE_DISPLAY_VALUE
        } else {
            return None;
        };
        let value_field = project_item.get_properties().get_field(value_field_name)?;
        let value_data_value = value_field.get_data_value()?;
        let symbolic_struct_namespace = resolve_project_item_struct_layout_id(&ProjectSymbolCatalog::default(), project_item);
        let symbolic_field_definition = symbolic_struct_namespace
            .as_deref()
            .and_then(|symbolic_struct_namespace| SymbolicFieldDefinition::from_str(symbolic_struct_namespace).ok());
        let validation_data_type_ref = symbolic_field_definition
            .as_ref()
            .map(|symbolic_field_definition| symbolic_field_definition.get_data_type_ref().clone())
            .unwrap_or_else(|| value_data_value.get_data_type_ref().clone());
        let container_type = symbolic_field_definition
            .map(|symbolic_field_definition| symbolic_field_definition.get_container_type())
            .unwrap_or(ContainerType::None);
        let default_format = engine_unprivileged_state.get_default_anonymous_value_string_format(&validation_data_type_ref);
        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let initial_value_edit = symbolic_struct_namespace
            .as_deref()
            .and_then(|symbolic_struct_namespace| {
                Self::read_project_item_runtime_value_from_memory(&engine_execution_context, opened_project_info, project_item, symbolic_struct_namespace)
            })
            .unwrap_or_else(|| {
                let raw_display_value = String::from_utf8(value_data_value.get_value_bytes().clone()).unwrap_or_default();

                AnonymousValueString::new(raw_display_value, default_format, container_type)
            });

        Some(Self {
            project_item_name: project_item.get_field_name(),
            value_field_name: value_field_name.to_string(),
            validation_data_type_ref,
            initial_value_edit,
        })
    }

    pub fn build_display_values(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        validation_data_type_ref: &DataTypeRef,
        value_edit: &AnonymousValueString,
    ) -> Vec<AnonymousValueString> {
        let Ok(data_value) = engine_unprivileged_state.deanonymize_value_string(validation_data_type_ref, value_edit) else {
            return Vec::new();
        };

        engine_unprivileged_state
            .anonymize_value_to_supported_formats(&data_value)
            .unwrap_or_else(|_| vec![value_edit.clone()])
    }

    fn read_project_item_runtime_value_from_memory(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
        symbolic_struct_namespace: &str,
    ) -> Option<AnonymousValueString> {
        let project_symbol_catalog = opened_project_info.map(|opened_project_info| opened_project_info.get_project_symbol_catalog());
        let (address, module_name) = resolve_project_item_runtime_value_target(engine_execution_context, project_symbol_catalog, project_item)?;
        let symbolic_struct_definition = engine_execution_context.resolve_struct_layout_definition(symbolic_struct_namespace)?;
        let memory_read_response = Self::dispatch_memory_read_request(engine_execution_context, address, &module_name, &symbolic_struct_definition)?;

        if !memory_read_response.success {
            return None;
        }

        let read_data_value = memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field| valued_struct_field.get_data_value())?;
        let default_format = engine_execution_context.get_default_anonymous_value_string_format(read_data_value.get_data_type_ref());

        engine_execution_context
            .anonymize_value(read_data_value, default_format)
            .ok()
    }

    fn dispatch_memory_read_request(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
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

        let dispatch_result = match engine_execution_context.get_bindings().read() {
            Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
                memory_read_command,
                Box::new(move |engine_response| {
                    let conversion_result = match MemoryReadResponse::from_engine_response(engine_response) {
                        Ok(memory_read_response) => Ok(memory_read_response),
                        Err(unexpected_response) => Err(format!(
                            "Unexpected response variant for project hierarchy value edit memory read request: {:?}",
                            unexpected_response
                        )),
                    };
                    let _ = memory_read_response_sender.send(conversion_result);
                }),
            ),
            Err(error) => {
                log::error!(
                    "Failed to acquire engine bindings lock for project hierarchy value edit memory read request: {}",
                    error
                );
                return None;
            }
        };

        if let Err(error) = dispatch_result {
            log::error!("Failed to dispatch project hierarchy value edit memory read request: {}", error);
            return None;
        }

        match memory_read_response_receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(memory_read_response)) => Some(memory_read_response),
            Ok(Err(error)) => {
                log::error!("Failed to convert project hierarchy value edit memory read response: {}", error);
                None
            }
            Err(error) => {
                log::error!("Timed out waiting for project hierarchy value edit memory read response: {}", error);
                None
            }
        }
    }
}
