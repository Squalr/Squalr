use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_response::ProjectItemsListResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

struct PointerPreviewEvaluation {
    resolved_target_address: Option<(u64, String)>,
    evaluated_path: String,
}

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
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            refresh_address_project_item_display_value(engine_unprivileged_state, project_item, symbol_registry);
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            refresh_pointer_project_item_display_value(engine_unprivileged_state, project_item, symbol_registry);
        }
    }
}

fn refresh_address_project_item_display_value(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    project_item: &mut ProjectItem,
    symbol_registry: &SymbolRegistry,
) {
    let address = ProjectItemTypeAddress::get_field_address(project_item);
    let module_name = ProjectItemTypeAddress::get_field_module(project_item);
    let Some(symbolic_struct_reference) = ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(project_item) else {
        ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(project_item, "");
        return;
    };
    let symbolic_struct_namespace = symbolic_struct_reference
        .get_symbolic_struct_namespace()
        .to_string();
    let Some(symbolic_struct_definition) = symbol_registry.get(&symbolic_struct_namespace) else {
        ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(project_item, "");
        return;
    };

    refresh_project_item_display_value_from_memory_read(
        engine_unprivileged_state,
        address,
        &module_name,
        symbolic_struct_definition.as_ref(),
        symbol_registry,
        |freeze_display_value| {
            ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(project_item, freeze_display_value);
        },
    );
}

fn refresh_pointer_project_item_display_value(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    project_item: &mut ProjectItem,
    symbol_registry: &SymbolRegistry,
) {
    let pointer = ProjectItemTypePointer::get_field_pointer(project_item);
    let pointer_preview_evaluation = evaluate_pointer_for_preview(engine_unprivileged_state, &pointer);

    ProjectItemTypePointer::set_field_evaluated_pointer_path(project_item, &pointer_preview_evaluation.evaluated_path);

    let Some(symbolic_struct_reference) = ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(project_item) else {
        ProjectItemTypePointer::set_field_freeze_data_value_interpreter(project_item, "");
        return;
    };
    let symbolic_struct_namespace = symbolic_struct_reference
        .get_symbolic_struct_namespace()
        .to_string();
    let Some(symbolic_struct_definition) = symbol_registry.get(&symbolic_struct_namespace) else {
        ProjectItemTypePointer::set_field_freeze_data_value_interpreter(project_item, "");
        return;
    };
    let Some((resolved_address, resolved_module_name)) = pointer_preview_evaluation.resolved_target_address else {
        ProjectItemTypePointer::set_field_freeze_data_value_interpreter(project_item, "");
        return;
    };

    refresh_project_item_display_value_from_memory_read(
        engine_unprivileged_state,
        resolved_address,
        &resolved_module_name,
        symbolic_struct_definition.as_ref(),
        symbol_registry,
        |freeze_display_value| {
            ProjectItemTypePointer::set_field_freeze_data_value_interpreter(project_item, freeze_display_value);
        },
    );
}

fn refresh_project_item_display_value_from_memory_read<SetDisplayValue>(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    address: u64,
    module_name: &str,
    symbolic_struct_definition: &SymbolicStructDefinition,
    symbol_registry: &SymbolRegistry,
    set_display_value: SetDisplayValue,
) where
    SetDisplayValue: FnOnce(&str),
{
    let Some(memory_read_response) = dispatch_memory_read_request(engine_unprivileged_state, address, module_name, symbolic_struct_definition) else {
        set_display_value("");
        return;
    };

    if !memory_read_response.success {
        set_display_value("");
        return;
    }

    let first_read_field_data_value = memory_read_response
        .valued_struct
        .get_fields()
        .first()
        .and_then(|valued_struct_field| valued_struct_field.get_data_value());
    let Some(first_read_field_data_value) = first_read_field_data_value else {
        set_display_value("");
        return;
    };

    let default_anonymous_value_string_format = symbol_registry.get_default_anonymous_value_string_format(first_read_field_data_value.get_data_type_ref());
    let freeze_display_value = symbol_registry
        .anonymize_value(first_read_field_data_value, default_anonymous_value_string_format)
        .map(|anonymous_value_string| anonymous_value_string.get_anonymous_value_string().to_string())
        .unwrap_or_default();

    set_display_value(&freeze_display_value);
}

fn evaluate_pointer_for_preview(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    pointer: &Pointer,
) -> PointerPreviewEvaluation {
    let mut evaluated_path_segments = vec![format_pointer_root_segment(pointer)];
    let mut current_address = pointer.get_address();
    let mut current_module_name = pointer.get_module_name().to_string();

    for pointer_offset in pointer.get_offsets() {
        let Some(pointer_value) = read_pointer_value(engine_unprivileged_state, current_address, &current_module_name, pointer.get_pointer_size()) else {
            evaluated_path_segments.push(String::from("??"));

            return PointerPreviewEvaluation {
                resolved_target_address: None,
                evaluated_path: evaluated_path_segments.join(" -> "),
            };
        };
        let Some(next_address) = Pointer::apply_pointer_offset(pointer_value, *pointer_offset) else {
            evaluated_path_segments.push(String::from("??"));

            return PointerPreviewEvaluation {
                resolved_target_address: None,
                evaluated_path: evaluated_path_segments.join(" -> "),
            };
        };

        current_address = next_address;
        current_module_name.clear();
        evaluated_path_segments.push(format_pointer_address_segment(current_address));
    }

    PointerPreviewEvaluation {
        resolved_target_address: Some((current_address, current_module_name)),
        evaluated_path: evaluated_path_segments.join(" -> "),
    }
}

fn format_pointer_root_segment(pointer: &Pointer) -> String {
    if pointer.get_module_name().is_empty() {
        format_pointer_address_segment(pointer.get_address())
    } else {
        format!("{}+0x{:X}", pointer.get_module_name(), pointer.get_address())
    }
}

fn format_pointer_address_segment(address: u64) -> String {
    format!("0x{:X}", address)
}

fn read_pointer_value(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    address: u64,
    module_name: &str,
    pointer_size: PointerScanPointerSize,
) -> Option<u64> {
    let symbol_registry = SymbolRegistry::get_instance();
    let symbolic_struct_definition = symbol_registry.get(pointer_size.to_data_type_ref().get_data_type_id())?;
    let memory_read_response = dispatch_memory_read_request(engine_unprivileged_state, address, module_name, symbolic_struct_definition.as_ref())?;

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

#[cfg(test)]
mod tests {
    use super::evaluate_pointer_for_preview;
    use crossbeam_channel::{Receiver, unbounded};
    use squalr_engine_api::commands::memory::memory_command::MemoryCommand;
    use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
    use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
    use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
    use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
    use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
    use squalr_engine_api::commands::unprivileged_command_response::UnprivilegedCommandResponse;
    use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
    use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::events::engine_event::EngineEvent;
    use squalr_engine_api::structures::data_types::built_in_types::{u32::data_type_u32::DataTypeU32, u64::data_type_u64::DataTypeU64};
    use squalr_engine_api::structures::memory::pointer::Pointer;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
    use squalr_engine_session::engine_unprivileged_state::{EngineUnprivilegedState, EngineUnprivilegedStateOptions};
    use std::sync::{Arc, Mutex, RwLock};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedMemoryReadRequest {
        address: u64,
        module_name: String,
    }

    struct MockMemoryReadBindings {
        captured_memory_read_requests: Arc<Mutex<Vec<CapturedMemoryReadRequest>>>,
        memory_read_response_factory: Arc<dyn Fn(&MemoryReadRequest) -> MemoryReadResponse + Send + Sync>,
    }

    impl MockMemoryReadBindings {
        fn new(memory_read_response_factory: impl Fn(&MemoryReadRequest) -> MemoryReadResponse + Send + Sync + 'static) -> Self {
            Self {
                captured_memory_read_requests: Arc::new(Mutex::new(Vec::new())),
                memory_read_response_factory: Arc::new(memory_read_response_factory),
            }
        }

        fn captured_memory_read_requests(&self) -> Arc<Mutex<Vec<CapturedMemoryReadRequest>>> {
            self.captured_memory_read_requests.clone()
        }
    }

    impl EngineApiUnprivilegedBindings for MockMemoryReadBindings {
        fn dispatch_privileged_command(
            &self,
            engine_command: PrivilegedCommand,
            callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            let PrivilegedCommand::Memory(MemoryCommand::Read { memory_read_request }) = engine_command else {
                return Err(EngineBindingError::unavailable("dispatching pointer preview memory reads in tests"));
            };
            let mut captured_memory_read_requests = self
                .captured_memory_read_requests
                .lock()
                .map_err(|error| EngineBindingError::lock_failure("capturing pointer preview memory reads in tests", error.to_string()))?;

            captured_memory_read_requests.push(CapturedMemoryReadRequest {
                address: memory_read_request.address,
                module_name: memory_read_request.module_name.clone(),
            });
            drop(captured_memory_read_requests);

            callback((self.memory_read_response_factory)(&memory_read_request).to_engine_response());

            Ok(())
        }

        fn dispatch_unprivileged_command(
            &self,
            _engine_command: UnprivilegedCommand,
            _engine_execution_context: &Arc<dyn EngineExecutionContext>,
            _callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            Err(EngineBindingError::unavailable("dispatching unprivileged commands in pointer preview tests"))
        }

        fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, EngineBindingError> {
            let (_event_sender, event_receiver) = unbounded();

            Ok(event_receiver)
        }
    }

    fn create_pointer_memory_read_response(
        pointer_value: u64,
        pointer_size: PointerScanPointerSize,
        success: bool,
    ) -> MemoryReadResponse {
        let valued_struct = if success {
            let value_field = match pointer_size {
                PointerScanPointerSize::Pointer32 => {
                    DataTypeU32::get_value_from_primitive(pointer_value as u32).to_named_valued_struct_field("value".to_string(), true)
                }
                PointerScanPointerSize::Pointer64 => {
                    DataTypeU64::get_value_from_primitive(pointer_value).to_named_valued_struct_field("value".to_string(), true)
                }
            };

            ValuedStruct::new_anonymous(vec![value_field])
        } else {
            ValuedStruct::default()
        };

        MemoryReadResponse {
            valued_struct,
            address: pointer_value,
            success,
        }
    }

    fn create_execution_context(mock_memory_read_bindings: MockMemoryReadBindings) -> Arc<dyn EngineExecutionContext> {
        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = Arc::new(RwLock::new(mock_memory_read_bindings));

        EngineUnprivilegedState::new_with_options(engine_bindings, EngineUnprivilegedStateOptions { enable_console_logging: false })
    }

    #[test]
    fn evaluate_pointer_for_preview_resolves_full_chain_and_clears_module_after_first_hop() {
        let mock_memory_read_bindings =
            MockMemoryReadBindings::new(
                |memory_read_request| match (memory_read_request.address, memory_read_request.module_name.as_str()) {
                    (0x1000, "game.exe") => create_pointer_memory_read_response(0x2000, PointerScanPointerSize::Pointer64, true),
                    (0x2020, "") => create_pointer_memory_read_response(0x3000, PointerScanPointerSize::Pointer64, true),
                    unexpected_request => panic!("Unexpected memory read request: {unexpected_request:?}"),
                },
            );
        let captured_memory_read_requests = mock_memory_read_bindings.captured_memory_read_requests();
        let engine_execution_context = create_execution_context(mock_memory_read_bindings);
        let pointer = Pointer::new_with_size(0x1000, vec![0x20, -0x10], "game.exe".to_string(), PointerScanPointerSize::Pointer64);

        let resolved_target_address = evaluate_pointer_for_preview(&engine_execution_context, &pointer).resolved_target_address;
        let captured_memory_read_requests = captured_memory_read_requests
            .lock()
            .expect("Expected captured pointer preview memory reads.");

        assert_eq!(resolved_target_address, Some((0x2FF0, String::new())));
        assert_eq!(
            *captured_memory_read_requests,
            vec![
                CapturedMemoryReadRequest {
                    address: 0x1000,
                    module_name: "game.exe".to_string(),
                },
                CapturedMemoryReadRequest {
                    address: 0x2020,
                    module_name: String::new(),
                },
            ]
        );
    }

    #[test]
    fn evaluate_pointer_for_preview_returns_none_when_intermediate_read_fails() {
        let mock_memory_read_bindings =
            MockMemoryReadBindings::new(
                |memory_read_request| match (memory_read_request.address, memory_read_request.module_name.as_str()) {
                    (0x1000, "game.exe") => create_pointer_memory_read_response(0x2000, PointerScanPointerSize::Pointer32, true),
                    (0x2010, "") => create_pointer_memory_read_response(0, PointerScanPointerSize::Pointer32, false),
                    unexpected_request => panic!("Unexpected memory read request: {unexpected_request:?}"),
                },
            );
        let engine_execution_context = create_execution_context(mock_memory_read_bindings);
        let pointer = Pointer::new_with_size(0x1000, vec![0x10, 0x8], "game.exe".to_string(), PointerScanPointerSize::Pointer32);

        let resolved_target_address = evaluate_pointer_for_preview(&engine_execution_context, &pointer).resolved_target_address;

        assert!(resolved_target_address.is_none());
    }

    #[test]
    fn evaluate_pointer_for_preview_records_materialized_pointer_path() {
        let mock_memory_read_bindings =
            MockMemoryReadBindings::new(
                |memory_read_request| match (memory_read_request.address, memory_read_request.module_name.as_str()) {
                    (0x1000, "game.exe") => create_pointer_memory_read_response(0x2000, PointerScanPointerSize::Pointer64, true),
                    (0x2020, "") => create_pointer_memory_read_response(0x3000, PointerScanPointerSize::Pointer64, true),
                    unexpected_request => panic!("Unexpected memory read request: {unexpected_request:?}"),
                },
            );
        let engine_execution_context = create_execution_context(mock_memory_read_bindings);
        let pointer = Pointer::new_with_size(0x1000, vec![0x20, -0x10], "game.exe".to_string(), PointerScanPointerSize::Pointer64);

        let pointer_preview_evaluation = evaluate_pointer_for_preview(&engine_execution_context, &pointer);

        assert_eq!(pointer_preview_evaluation.resolved_target_address, Some((0x2FF0, String::new())));
        assert_eq!(pointer_preview_evaluation.evaluated_path, "game.exe+0x1000 -> 0x2020 -> 0x2FF0");
    }

    #[test]
    fn evaluate_pointer_for_preview_marks_failed_hops() {
        let mock_memory_read_bindings =
            MockMemoryReadBindings::new(
                |memory_read_request| match (memory_read_request.address, memory_read_request.module_name.as_str()) {
                    (0x1000, "game.exe") => create_pointer_memory_read_response(0x2000, PointerScanPointerSize::Pointer32, true),
                    (0x2010, "") => create_pointer_memory_read_response(0, PointerScanPointerSize::Pointer32, false),
                    unexpected_request => panic!("Unexpected memory read request: {unexpected_request:?}"),
                },
            );
        let engine_execution_context = create_execution_context(mock_memory_read_bindings);
        let pointer = Pointer::new_with_size(0x1000, vec![0x10, 0x8], "game.exe".to_string(), PointerScanPointerSize::Pointer32);

        let pointer_preview_evaluation = evaluate_pointer_for_preview(&engine_execution_context, &pointer);

        assert_eq!(pointer_preview_evaluation.resolved_target_address, None);
        assert_eq!(pointer_preview_evaluation.evaluated_path, "game.exe+0x1000 -> 0x2010 -> ??");
    }
}
