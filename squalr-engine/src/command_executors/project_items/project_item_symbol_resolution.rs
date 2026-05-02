use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer,
    project_item_type_symbol_ref::ProjectItemTypeSymbolRef,
};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::projects::project_symbol_claim::ProjectSymbolClaim;
use squalr_engine_api::structures::projects::project_symbol_locator::ProjectSymbolLocator;
use squalr_engine_api::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};
use std::sync::{Arc, mpsc};

pub fn is_promotable_project_item(project_item: &ProjectItem) -> bool {
    let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

    (project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID || project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID)
        && resolve_project_item_type_id(project_item).is_some()
}

pub fn resolve_project_item_symbol_ref_key(project_item: &ProjectItem) -> Option<String> {
    let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

    if project_item_type_id != ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
        return None;
    }

    let symbol_key = ProjectItemTypeSymbolRef::get_field_symbol_key(project_item);

    if symbol_key.trim().is_empty() { None } else { Some(symbol_key) }
}

pub fn resolve_project_item_symbol_claim<'a>(
    project_symbol_catalog: &'a ProjectSymbolCatalog,
    project_item: &ProjectItem,
) -> Option<&'a ProjectSymbolClaim> {
    let symbol_key = resolve_project_item_symbol_ref_key(project_item)?;

    project_symbol_catalog.find_symbol_claim(&symbol_key)
}

pub fn resolve_project_item_struct_layout_id(
    project_symbol_catalog: &ProjectSymbolCatalog,
    project_item: &ProjectItem,
) -> Option<String> {
    let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

    if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
        let mut project_item = project_item.clone();

        return ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(&mut project_item)
            .map(|symbolic_struct_ref| symbolic_struct_ref.get_symbolic_struct_namespace().to_string());
    }

    if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
        return ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(project_item)
            .map(|symbolic_struct_ref| symbolic_struct_ref.get_symbolic_struct_namespace().to_string());
    }

    if project_item_type_id == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
        return resolve_project_item_symbol_claim(project_symbol_catalog, project_item).map(|symbol_claim| symbol_claim.get_struct_layout_id().to_string());
    }

    None
}

pub fn resolve_project_item_type_id(project_item: &ProjectItem) -> Option<&'static str> {
    let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

    if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
        return Some(ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID);
    }

    if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
        return Some(ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID);
    }

    if project_item_type_id == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
        return Some(ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID);
    }

    None
}

pub fn resolve_project_item_locator(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    project_item: &ProjectItem,
) -> Option<ProjectSymbolLocator> {
    let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

    if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
        let mut project_item = project_item.clone();
        let address = ProjectItemTypeAddress::get_field_address(&mut project_item);
        let module_name = ProjectItemTypeAddress::get_field_module(&mut project_item);

        return Some(build_locator(address, &module_name));
    }

    if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
        let pointer = ProjectItemTypePointer::get_field_pointer(project_item);
        let (address, module_name) = resolve_pointer_runtime_target(engine_execution_context, &pointer)?;

        return Some(build_locator(address, &module_name));
    }

    if project_item_type_id == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
        return resolve_project_item_symbol_claim(project_symbol_catalog, project_item).map(|symbol_claim| symbol_claim.get_locator().clone());
    }

    None
}

pub fn resolve_pointer_runtime_target(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    pointer: &Pointer,
) -> Option<(u64, String)> {
    let mut current_address = pointer.get_address();
    let mut current_module_name = pointer.get_module_name().to_string();

    for pointer_offset in pointer.get_offsets() {
        let pointer_value = read_pointer_value(engine_execution_context, current_address, &current_module_name, pointer.get_pointer_size())?;
        current_address = Pointer::apply_pointer_offset(pointer_value, *pointer_offset)?;
        current_module_name.clear();
    }

    Some((current_address, current_module_name))
}

fn build_locator(
    address: u64,
    module_name: &str,
) -> ProjectSymbolLocator {
    if module_name.trim().is_empty() {
        ProjectSymbolLocator::new_absolute_address(address)
    } else {
        ProjectSymbolLocator::new_module_offset(module_name.to_string(), address)
    }
}

fn read_pointer_value(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    address: u64,
    module_name: &str,
    pointer_size: PointerScanPointerSize,
) -> Option<u64> {
    let symbolic_struct_definition = SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
        pointer_size.to_data_type_ref(),
        ContainerType::None,
    )]);
    let memory_read_response = dispatch_memory_read_request(engine_execution_context, address, module_name, &symbolic_struct_definition)?;

    if !memory_read_response.success {
        return None;
    }

    let data_value = memory_read_response
        .valued_struct
        .get_fields()
        .first()
        .and_then(|valued_struct_field| valued_struct_field.get_data_value())?;

    pointer_size.read_address_value(data_value)
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
                        "Unexpected response variant for project-item symbol resolution memory read request: {:?}",
                        unexpected_response
                    )),
                };
                let _ = memory_read_response_sender.send(conversion_result);
            }),
        ),
        Err(error) => {
            log::error!("Failed to acquire engine bindings lock for project-item symbol resolution: {}", error);
            return None;
        }
    };

    if let Err(error) = dispatch_result {
        log::error!("Failed to dispatch project-item symbol resolution memory read request: {}", error);
        return None;
    }

    match memory_read_response_receiver.recv_timeout(std::time::Duration::from_secs(1)) {
        Ok(Ok(memory_read_response)) => Some(memory_read_response),
        Ok(Err(error)) => {
            log::error!("Failed to convert project-item symbol resolution memory read response: {}", error);
            None
        }
        Err(error) => {
            log::error!("Timed out waiting for project-item symbol resolution memory read response: {}", error);
            None
        }
    }
}
