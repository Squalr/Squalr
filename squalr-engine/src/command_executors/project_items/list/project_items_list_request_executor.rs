use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_response::ProjectItemsListResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType};
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::structs::symbolic_field_definition::SymbolicFieldDefinition;
use squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct CachedMemoryReadKey {
    address: u64,
    module_name: String,
    layout_key: String,
}

struct ProjectItemPreviewRefreshSession {
    requested_preview_project_item_paths: Option<HashSet<PathBuf>>,
    cached_memory_read_responses: HashMap<CachedMemoryReadKey, Option<MemoryReadResponse>>,
    cached_pointer_preview_evaluations: HashMap<Pointer, PointerPreviewEvaluation>,
}

#[derive(Clone)]
struct ProjectItemPreviewReadDefinition {
    layout_key: String,
    symbolic_struct_definition: SymbolicStructDefinition,
    symbolic_field_container_type: ContainerType,
    preview_was_truncated: bool,
}

impl ProjectItemPreviewRefreshSession {
    fn new(requested_preview_project_item_paths: Option<Vec<PathBuf>>) -> Self {
        Self {
            requested_preview_project_item_paths: requested_preview_project_item_paths.map(|project_item_paths| project_item_paths.into_iter().collect()),
            cached_memory_read_responses: HashMap::new(),
            cached_pointer_preview_evaluations: HashMap::new(),
        }
    }

    fn should_refresh_preview(
        &self,
        project_item_path: &Path,
    ) -> bool {
        self.requested_preview_project_item_paths
            .as_ref()
            .map(|requested_preview_project_item_paths| requested_preview_project_item_paths.contains(project_item_path))
            .unwrap_or(true)
    }
}

#[derive(Clone)]
struct PointerPreviewEvaluation {
    resolved_target_address: Option<(u64, String)>,
    evaluated_path: String,
}

const MAX_ARRAY_PREVIEW_CHARACTER_COUNT: usize = 96;
const MAX_ARRAY_PREVIEW_ELEMENT_COUNT: u64 = 64;

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
                let mut project_item_preview_refresh_session = ProjectItemPreviewRefreshSession::new(self.preview_project_item_paths.clone());

                refresh_project_item_display_values(engine_unprivileged_state, &mut opened_project_items, &mut project_item_preview_refresh_session);

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
    project_item_preview_refresh_session: &mut ProjectItemPreviewRefreshSession,
) {
    for (project_item_ref, project_item) in opened_project_items.iter_mut() {
        if !project_item_preview_refresh_session.should_refresh_preview(project_item_ref.get_project_item_path()) {
            continue;
        }

        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            refresh_address_project_item_display_value(engine_unprivileged_state, project_item, project_item_preview_refresh_session);
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            refresh_pointer_project_item_display_value(engine_unprivileged_state, project_item, project_item_preview_refresh_session);
        }
    }
}

fn refresh_address_project_item_display_value(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    project_item: &mut ProjectItem,
    project_item_preview_refresh_session: &mut ProjectItemPreviewRefreshSession,
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
    let Some(project_item_preview_read_definition) = build_project_item_preview_read_definition(engine_unprivileged_state, &symbolic_struct_namespace) else {
        ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(project_item, "");
        return;
    };

    refresh_project_item_display_value_from_memory_read(
        engine_unprivileged_state,
        project_item_preview_refresh_session,
        address,
        &module_name,
        &project_item_preview_read_definition,
        |freeze_display_value| {
            ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(project_item, freeze_display_value);
        },
    );
}

fn refresh_pointer_project_item_display_value(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    project_item: &mut ProjectItem,
    project_item_preview_refresh_session: &mut ProjectItemPreviewRefreshSession,
) {
    let pointer = ProjectItemTypePointer::get_field_pointer(project_item);
    let pointer_preview_evaluation = evaluate_pointer_for_preview(engine_unprivileged_state, &pointer, project_item_preview_refresh_session);

    ProjectItemTypePointer::set_field_evaluated_pointer_path(project_item, &pointer_preview_evaluation.evaluated_path);

    let Some(symbolic_struct_reference) = ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(project_item) else {
        ProjectItemTypePointer::set_field_freeze_data_value_interpreter(project_item, "");
        return;
    };
    let symbolic_struct_namespace = symbolic_struct_reference
        .get_symbolic_struct_namespace()
        .to_string();
    let Some(project_item_preview_read_definition) = build_project_item_preview_read_definition(engine_unprivileged_state, &symbolic_struct_namespace) else {
        ProjectItemTypePointer::set_field_freeze_data_value_interpreter(project_item, "");
        return;
    };
    let Some((resolved_address, resolved_module_name)) = pointer_preview_evaluation.resolved_target_address else {
        ProjectItemTypePointer::set_field_freeze_data_value_interpreter(project_item, "");
        return;
    };

    refresh_project_item_display_value_from_memory_read(
        engine_unprivileged_state,
        project_item_preview_refresh_session,
        resolved_address,
        &resolved_module_name,
        &project_item_preview_read_definition,
        |freeze_display_value| {
            ProjectItemTypePointer::set_field_freeze_data_value_interpreter(project_item, freeze_display_value);
        },
    );
}

fn refresh_project_item_display_value_from_memory_read<SetDisplayValue>(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    project_item_preview_refresh_session: &mut ProjectItemPreviewRefreshSession,
    address: u64,
    module_name: &str,
    project_item_preview_read_definition: &ProjectItemPreviewReadDefinition,
    set_display_value: SetDisplayValue,
) where
    SetDisplayValue: FnOnce(&str),
{
    let Some(memory_read_response) = dispatch_memory_read_request(
        engine_unprivileged_state,
        project_item_preview_refresh_session,
        address,
        module_name,
        &project_item_preview_read_definition.layout_key,
        &project_item_preview_read_definition.symbolic_struct_definition,
    ) else {
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

    let default_anonymous_value_string_format =
        engine_unprivileged_state.get_default_anonymous_value_string_format(first_read_field_data_value.get_data_type_ref());
    let freeze_display_value = engine_unprivileged_state
        .anonymize_value(first_read_field_data_value, default_anonymous_value_string_format)
        .map(|anonymous_value_string| {
            format_project_item_preview_value(
                &anonymous_value_string,
                project_item_preview_read_definition.symbolic_field_container_type,
                project_item_preview_read_definition.preview_was_truncated,
            )
        })
        .unwrap_or_default();

    set_display_value(&freeze_display_value);
}

fn build_project_item_preview_read_definition(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    symbolic_struct_namespace: &str,
) -> Option<ProjectItemPreviewReadDefinition> {
    let symbolic_field_container_type = resolve_project_item_container_type(symbolic_struct_namespace);
    let symbolic_struct_definition = engine_unprivileged_state.resolve_struct_layout_definition(symbolic_struct_namespace)?;
    let preview_field_definition = SymbolicFieldDefinition::from_str(symbolic_struct_namespace).ok();

    let Some(preview_field_definition) = preview_field_definition else {
        return Some(ProjectItemPreviewReadDefinition {
            layout_key: symbolic_struct_namespace.to_string(),
            symbolic_struct_definition,
            symbolic_field_container_type,
            preview_was_truncated: false,
        });
    };

    let preview_container_type = match preview_field_definition.get_container_type() {
        ContainerType::ArrayFixed(length) if length > MAX_ARRAY_PREVIEW_ELEMENT_COUNT => ContainerType::ArrayFixed(MAX_ARRAY_PREVIEW_ELEMENT_COUNT),
        container_type => container_type,
    };
    let preview_was_truncated = preview_container_type != preview_field_definition.get_container_type();

    Some(ProjectItemPreviewReadDefinition {
        layout_key: if preview_was_truncated {
            format!("{}|preview:{}", symbolic_struct_namespace, preview_container_type)
        } else {
            symbolic_struct_namespace.to_string()
        },
        symbolic_struct_definition: if preview_was_truncated {
            SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                preview_field_definition.get_data_type_ref().clone(),
                preview_container_type,
            )])
        } else {
            symbolic_struct_definition
        },
        symbolic_field_container_type,
        preview_was_truncated,
    })
}

fn resolve_project_item_container_type(symbolic_struct_namespace: &str) -> ContainerType {
    SymbolicFieldDefinition::from_str(symbolic_struct_namespace)
        .map(|symbolic_field_definition| symbolic_field_definition.get_container_type())
        .unwrap_or(ContainerType::None)
}

fn format_project_item_preview_value(
    anonymous_value_string: &AnonymousValueString,
    symbolic_field_container_type: ContainerType,
    preview_was_truncated: bool,
) -> String {
    let effective_container_type = if matches!(anonymous_value_string.get_container_type(), ContainerType::Array | ContainerType::ArrayFixed(_)) {
        anonymous_value_string.get_container_type()
    } else {
        symbolic_field_container_type
    };
    let display_value = anonymous_value_string.get_anonymous_value_string();

    if matches!(effective_container_type, ContainerType::Array | ContainerType::ArrayFixed(_)) && !display_value.is_empty() {
        let preview_value = if preview_was_truncated {
            append_array_preview_ellipsis(display_value)
        } else {
            truncate_array_preview_value(display_value)
        };

        format!("[{}]", preview_value)
    } else {
        display_value.to_string()
    }
}

fn append_array_preview_ellipsis(display_value: &str) -> String {
    let trimmed_display_value = display_value.trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'));

    if trimmed_display_value.is_empty() {
        String::from("...")
    } else {
        format!("{}...", trimmed_display_value)
    }
}

fn truncate_array_preview_value(display_value: &str) -> String {
    let display_value_character_count = display_value.chars().count();

    if display_value_character_count <= MAX_ARRAY_PREVIEW_CHARACTER_COUNT {
        return display_value.to_string();
    }

    let truncated_prefix: String = display_value
        .chars()
        .take(MAX_ARRAY_PREVIEW_CHARACTER_COUNT)
        .collect::<String>()
        .trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'))
        .to_string();

    format!("{}...", truncated_prefix)
}

fn evaluate_pointer_for_preview(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    pointer: &Pointer,
    project_item_preview_refresh_session: &mut ProjectItemPreviewRefreshSession,
) -> PointerPreviewEvaluation {
    if let Some(pointer_preview_evaluation) = project_item_preview_refresh_session
        .cached_pointer_preview_evaluations
        .get(pointer)
    {
        return pointer_preview_evaluation.clone();
    }

    let mut evaluated_path_segments = vec![format_pointer_root_segment(pointer)];
    let mut current_address = pointer.get_address();
    let mut current_module_name = pointer.get_module_name().to_string();

    for pointer_offset in pointer.get_offsets() {
        let Some(pointer_value) = read_pointer_value(
            engine_unprivileged_state,
            project_item_preview_refresh_session,
            current_address,
            &current_module_name,
            pointer.get_pointer_size(),
        ) else {
            evaluated_path_segments.push(String::from("??"));

            let pointer_preview_evaluation = PointerPreviewEvaluation {
                resolved_target_address: None,
                evaluated_path: evaluated_path_segments.join(" -> "),
            };

            project_item_preview_refresh_session
                .cached_pointer_preview_evaluations
                .insert(pointer.clone(), pointer_preview_evaluation.clone());

            return pointer_preview_evaluation;
        };
        let Some(next_address) = Pointer::apply_pointer_offset(pointer_value, *pointer_offset) else {
            evaluated_path_segments.push(String::from("??"));

            let pointer_preview_evaluation = PointerPreviewEvaluation {
                resolved_target_address: None,
                evaluated_path: evaluated_path_segments.join(" -> "),
            };

            project_item_preview_refresh_session
                .cached_pointer_preview_evaluations
                .insert(pointer.clone(), pointer_preview_evaluation.clone());

            return pointer_preview_evaluation;
        };

        current_address = next_address;
        current_module_name.clear();
        evaluated_path_segments.push(format_pointer_address_segment(current_address));
    }

    let pointer_preview_evaluation = PointerPreviewEvaluation {
        resolved_target_address: Some((current_address, current_module_name)),
        evaluated_path: evaluated_path_segments.join(" -> "),
    };

    project_item_preview_refresh_session
        .cached_pointer_preview_evaluations
        .insert(pointer.clone(), pointer_preview_evaluation.clone());

    pointer_preview_evaluation
}

fn format_pointer_root_segment(pointer: &Pointer) -> String {
    pointer.get_root_display_text()
}

fn format_pointer_address_segment(address: u64) -> String {
    format!("0x{:X}", address)
}

fn read_pointer_value(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    project_item_preview_refresh_session: &mut ProjectItemPreviewRefreshSession,
    address: u64,
    module_name: &str,
    pointer_size: PointerScanPointerSize,
) -> Option<u64> {
    let symbolic_struct_definition = engine_unprivileged_state.resolve_struct_layout_definition(pointer_size.to_data_type_ref().get_data_type_id())?;
    let memory_read_response = dispatch_memory_read_request(
        engine_unprivileged_state,
        project_item_preview_refresh_session,
        address,
        module_name,
        &pointer_size.to_string(),
        &symbolic_struct_definition,
    )?;

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
    project_item_preview_refresh_session: &mut ProjectItemPreviewRefreshSession,
    address: u64,
    module_name: &str,
    layout_key: &str,
    symbolic_struct_definition: &SymbolicStructDefinition,
) -> Option<MemoryReadResponse> {
    let cached_memory_read_key = CachedMemoryReadKey {
        address,
        module_name: module_name.to_string(),
        layout_key: layout_key.to_string(),
    };

    if let Some(cached_memory_read_response) = project_item_preview_refresh_session
        .cached_memory_read_responses
        .get(&cached_memory_read_key)
    {
        return cached_memory_read_response.clone();
    }

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

    let memory_read_response = match memory_read_response_receiver.recv_timeout(Duration::from_secs(2)) {
        Ok(Ok(memory_read_response)) => Some(memory_read_response),
        Ok(Err(error)) => {
            log::error!("Failed to convert project-items list memory read response: {}", error);
            None
        }
        Err(error) => {
            log::error!("Timed out waiting for project-items list memory read response: {}", error);
            None
        }
    };

    project_item_preview_refresh_session
        .cached_memory_read_responses
        .insert(cached_memory_read_key, memory_read_response.clone());

    memory_read_response
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_ARRAY_PREVIEW_CHARACTER_COUNT, MAX_ARRAY_PREVIEW_ELEMENT_COUNT, ProjectItemPreviewRefreshSession, build_project_item_preview_read_definition,
        evaluate_pointer_for_preview, format_project_item_preview_value, refresh_pointer_project_item_display_value, refresh_project_item_display_values,
    };
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
    use squalr_engine_api::structures::data_types::built_in_types::{
        u16::data_type_u16::DataTypeU16, u32::data_type_u32::DataTypeU32, u64::data_type_u64::DataTypeU64,
    };
    use squalr_engine_api::structures::data_values::{
        anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
    };
    use squalr_engine_api::structures::memory::pointer::Pointer;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use squalr_engine_api::structures::projects::project_items::built_in_types::{
        project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer,
    };
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
    use squalr_engine_session::engine_unprivileged_state::{EngineUnprivilegedState, EngineUnprivilegedStateOptions};
    use std::path::PathBuf;
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

        fn subscribe_to_engine_events(&self) -> Result<Receiver<squalr_engine_api::engine::engine_event_envelope::EngineEventEnvelope>, EngineBindingError> {
            let (_event_sender, event_receiver) = unbounded();

            Ok(event_receiver)
        }
    }

    fn create_pointer_memory_read_response(
        pointer_value: u64,
        pointer_size: PointerScanPointerSize,
        success: bool,
    ) -> MemoryReadResponse {
        fn create_three_byte_pointer_value(
            pointer_value: u32,
            data_type_id: &str,
            is_big_endian: bool,
        ) -> squalr_engine_api::structures::data_values::data_value::DataValue {
            let value_bytes = if is_big_endian {
                vec![
                    (pointer_value >> 16) as u8,
                    (pointer_value >> 8) as u8,
                    pointer_value as u8,
                ]
            } else {
                vec![
                    pointer_value as u8,
                    (pointer_value >> 8) as u8,
                    (pointer_value >> 16) as u8,
                ]
            };

            squalr_engine_api::structures::data_values::data_value::DataValue::new(
                squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef::new(data_type_id),
                value_bytes,
            )
        }

        let valued_struct = if success {
            let value_field = match pointer_size {
                PointerScanPointerSize::Pointer24 => {
                    create_three_byte_pointer_value(pointer_value as u32, "u24", false).to_named_valued_struct_field("value".to_string(), true)
                }
                PointerScanPointerSize::Pointer24be => {
                    create_three_byte_pointer_value(pointer_value as u32, "u24be", true).to_named_valued_struct_field("value".to_string(), true)
                }
                PointerScanPointerSize::Pointer32 => {
                    DataTypeU32::get_value_from_primitive(pointer_value as u32).to_named_valued_struct_field("value".to_string(), true)
                }
                PointerScanPointerSize::Pointer32be => {
                    squalr_engine_api::structures::data_types::built_in_types::u32be::data_type_u32be::DataTypeU32be::get_value_from_primitive(
                        pointer_value as u32,
                    )
                    .to_named_valued_struct_field("value".to_string(), true)
                }
                PointerScanPointerSize::Pointer64 => {
                    DataTypeU64::get_value_from_primitive(pointer_value).to_named_valued_struct_field("value".to_string(), true)
                }
                PointerScanPointerSize::Pointer64be => {
                    squalr_engine_api::structures::data_types::built_in_types::u64be::data_type_u64be::DataTypeU64be::get_value_from_primitive(pointer_value)
                        .to_named_valued_struct_field("value".to_string(), true)
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

    fn create_value_memory_read_response(
        data_value: squalr_engine_api::structures::data_values::data_value::DataValue,
        success: bool,
    ) -> MemoryReadResponse {
        let valued_struct = if success {
            ValuedStruct::new_anonymous(vec![data_value.to_named_valued_struct_field("value".to_string(), true)])
        } else {
            ValuedStruct::default()
        };

        MemoryReadResponse {
            valued_struct,
            address: 0,
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
        let mut project_item_preview_refresh_session = ProjectItemPreviewRefreshSession::new(None);
        let resolved_target_address =
            evaluate_pointer_for_preview(&engine_execution_context, &pointer, &mut project_item_preview_refresh_session).resolved_target_address;
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
        let mut project_item_preview_refresh_session = ProjectItemPreviewRefreshSession::new(None);
        let resolved_target_address =
            evaluate_pointer_for_preview(&engine_execution_context, &pointer, &mut project_item_preview_refresh_session).resolved_target_address;

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
        let mut project_item_preview_refresh_session = ProjectItemPreviewRefreshSession::new(None);
        let pointer_preview_evaluation = evaluate_pointer_for_preview(&engine_execution_context, &pointer, &mut project_item_preview_refresh_session);

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
        let mut project_item_preview_refresh_session = ProjectItemPreviewRefreshSession::new(None);
        let pointer_preview_evaluation = evaluate_pointer_for_preview(&engine_execution_context, &pointer, &mut project_item_preview_refresh_session);

        assert_eq!(pointer_preview_evaluation.resolved_target_address, None);
        assert_eq!(pointer_preview_evaluation.evaluated_path, "game.exe+0x1000 -> 0x2010 -> ??");
    }

    #[test]
    fn refresh_pointer_project_item_display_value_reads_final_value_from_resolved_address() {
        let mock_memory_read_bindings =
            MockMemoryReadBindings::new(
                |memory_read_request| match (memory_read_request.address, memory_read_request.module_name.as_str()) {
                    (0x1000, "game.exe") => create_pointer_memory_read_response(0x2000, PointerScanPointerSize::Pointer64, true),
                    (0x2020, "") => create_pointer_memory_read_response(0x3000, PointerScanPointerSize::Pointer64, true),
                    (0x2FF0, "") => create_value_memory_read_response(DataTypeU16::get_value_from_primitive(0x1234), true),
                    unexpected_request => panic!("Unexpected memory read request: {unexpected_request:?}"),
                },
            );
        let captured_memory_read_requests = mock_memory_read_bindings.captured_memory_read_requests();
        let engine_execution_context = create_execution_context(mock_memory_read_bindings);
        let pointer = Pointer::new_with_size(0x1000, vec![0x20, -0x10], "game.exe".to_string(), PointerScanPointerSize::Pointer64);
        let mut pointer_project_item = ProjectItemTypePointer::new_project_item("Pointer", &pointer, "", "u16");
        let mut project_item_preview_refresh_session = ProjectItemPreviewRefreshSession::new(None);

        refresh_pointer_project_item_display_value(&engine_execution_context, &mut pointer_project_item, &mut project_item_preview_refresh_session);

        let captured_memory_read_requests = captured_memory_read_requests
            .lock()
            .expect("Expected captured pointer preview memory reads.");

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
                CapturedMemoryReadRequest {
                    address: 0x2FF0,
                    module_name: String::new(),
                },
            ]
        );
        assert_eq!(
            ProjectItemTypePointer::get_field_evaluated_pointer_path(&pointer_project_item),
            "game.exe+0x1000 -> 0x2020 -> 0x2FF0"
        );
        assert_eq!(ProjectItemTypePointer::get_field_freeze_data_value_interpreter(&pointer_project_item), "4660");
    }

    #[test]
    fn refresh_pointer_project_item_display_value_still_reads_final_value_after_project_item_round_trip() {
        let mock_memory_read_bindings =
            MockMemoryReadBindings::new(
                |memory_read_request| match (memory_read_request.address, memory_read_request.module_name.as_str()) {
                    (0x1000, "game.exe") => create_pointer_memory_read_response(0x2000, PointerScanPointerSize::Pointer64, true),
                    (0x2020, "") => create_pointer_memory_read_response(0x3000, PointerScanPointerSize::Pointer64, true),
                    (0x2FF0, "") => create_value_memory_read_response(DataTypeU16::get_value_from_primitive(0x1234), true),
                    unexpected_request => panic!("Unexpected memory read request: {unexpected_request:?}"),
                },
            );
        let captured_memory_read_requests = mock_memory_read_bindings.captured_memory_read_requests();
        let engine_execution_context = create_execution_context(mock_memory_read_bindings);
        let pointer = Pointer::new_with_size(0x1000, vec![0x20, -0x10], "game.exe".to_string(), PointerScanPointerSize::Pointer64);
        let pointer_project_item = ProjectItemTypePointer::new_project_item("Pointer", &pointer, "", "u16");
        let serialized_pointer_project_item = serde_json::to_string(&pointer_project_item).expect("Expected pointer project item serialization to succeed.");
        let mut deserialized_pointer_project_item =
            serde_json::from_str::<squalr_engine_api::structures::projects::project_items::project_item::ProjectItem>(&serialized_pointer_project_item)
                .expect("Expected pointer project item deserialization to succeed.");
        let mut project_item_preview_refresh_session = ProjectItemPreviewRefreshSession::new(None);

        refresh_pointer_project_item_display_value(
            &engine_execution_context,
            &mut deserialized_pointer_project_item,
            &mut project_item_preview_refresh_session,
        );

        let captured_memory_read_requests = captured_memory_read_requests
            .lock()
            .expect("Expected captured pointer preview memory reads.");

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
                CapturedMemoryReadRequest {
                    address: 0x2FF0,
                    module_name: String::new(),
                },
            ]
        );
        assert_eq!(
            ProjectItemTypePointer::get_field_evaluated_pointer_path(&deserialized_pointer_project_item),
            "game.exe+0x1000 -> 0x2020 -> 0x2FF0"
        );
        assert_eq!(
            ProjectItemTypePointer::get_field_freeze_data_value_interpreter(&deserialized_pointer_project_item),
            "4660"
        );
        assert_eq!(
            ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(&deserialized_pointer_project_item)
                .expect("Expected symbolic struct reference after project-item round trip.")
                .get_symbolic_struct_namespace(),
            "u16"
        );
    }

    #[test]
    fn refresh_project_item_display_values_skips_unrequested_previews() {
        let mock_memory_read_bindings = MockMemoryReadBindings::new(|memory_read_request| {
            panic!(
                "Did not expect memory read request while refresh was filtered out: {:?}",
                (memory_read_request.address, memory_read_request.module_name.as_str())
            )
        });
        let captured_memory_read_requests = mock_memory_read_bindings.captured_memory_read_requests();
        let engine_execution_context = create_execution_context(mock_memory_read_bindings);
        let mut opened_project_items = vec![(
            ProjectItemRef::new(PathBuf::from("project/health.json")),
            ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU16::get_value_from_primitive(0)),
        )];
        let mut project_item_preview_refresh_session = ProjectItemPreviewRefreshSession::new(Some(Vec::new()));

        refresh_project_item_display_values(&engine_execution_context, &mut opened_project_items, &mut project_item_preview_refresh_session);

        let captured_memory_read_requests = captured_memory_read_requests
            .lock()
            .expect("Expected captured preview-filter memory reads.");

        assert!(captured_memory_read_requests.is_empty());
    }

    #[test]
    fn refresh_project_item_display_values_deduplicates_duplicate_address_reads() {
        let mock_memory_read_bindings = MockMemoryReadBindings::new(|memory_read_request| {
            assert_eq!(memory_read_request.address, 0x1234);
            assert_eq!(memory_read_request.module_name, "game.exe");

            create_value_memory_read_response(DataTypeU16::get_value_from_primitive(0x1234), true)
        });
        let captured_memory_read_requests = mock_memory_read_bindings.captured_memory_read_requests();
        let engine_execution_context = create_execution_context(mock_memory_read_bindings);
        let mut opened_project_items = vec![
            (
                ProjectItemRef::new(PathBuf::from("project/health.json")),
                ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU16::get_value_from_primitive(0)),
            ),
            (
                ProjectItemRef::new(PathBuf::from("project/health_copy.json")),
                ProjectItemTypeAddress::new_project_item("Health Copy", 0x1234, "game.exe", "", DataTypeU16::get_value_from_primitive(0)),
            ),
        ];
        let mut project_item_preview_refresh_session = ProjectItemPreviewRefreshSession::new(None);

        refresh_project_item_display_values(&engine_execution_context, &mut opened_project_items, &mut project_item_preview_refresh_session);

        let captured_memory_read_requests = captured_memory_read_requests
            .lock()
            .expect("Expected captured duplicate address preview memory reads.");

        assert_eq!(
            *captured_memory_read_requests,
            vec![CapturedMemoryReadRequest {
                address: 0x1234,
                module_name: "game.exe".to_string(),
            }]
        );

        let first_preview_value = ProjectItemTypeAddress::get_field_freeze_data_value_interpreter(&mut opened_project_items[0].1);
        let second_preview_value = ProjectItemTypeAddress::get_field_freeze_data_value_interpreter(&mut opened_project_items[1].1);

        assert_eq!(first_preview_value, "4660");
        assert_eq!(second_preview_value, "4660");
    }

    #[test]
    fn refresh_project_item_display_values_deduplicates_duplicate_pointer_reads() {
        let mock_memory_read_bindings =
            MockMemoryReadBindings::new(
                |memory_read_request| match (memory_read_request.address, memory_read_request.module_name.as_str()) {
                    (0x1000, "game.exe") => create_pointer_memory_read_response(0x2000, PointerScanPointerSize::Pointer64, true),
                    (0x2020, "") => create_pointer_memory_read_response(0x3000, PointerScanPointerSize::Pointer64, true),
                    (0x2FF0, "") => create_value_memory_read_response(DataTypeU16::get_value_from_primitive(0x1234), true),
                    unexpected_request => panic!("Unexpected duplicate pointer memory read request: {unexpected_request:?}"),
                },
            );
        let captured_memory_read_requests = mock_memory_read_bindings.captured_memory_read_requests();
        let engine_execution_context = create_execution_context(mock_memory_read_bindings);
        let pointer = Pointer::new_with_size(0x1000, vec![0x20, -0x10], "game.exe".to_string(), PointerScanPointerSize::Pointer64);
        let mut opened_project_items = vec![
            (
                ProjectItemRef::new(PathBuf::from("project/ammo.json")),
                ProjectItemTypePointer::new_project_item("Ammo", &pointer, "", "u16"),
            ),
            (
                ProjectItemRef::new(PathBuf::from("project/ammo_copy.json")),
                ProjectItemTypePointer::new_project_item("Ammo Copy", &pointer, "", "u16"),
            ),
        ];
        let mut project_item_preview_refresh_session = ProjectItemPreviewRefreshSession::new(None);

        refresh_project_item_display_values(&engine_execution_context, &mut opened_project_items, &mut project_item_preview_refresh_session);

        let captured_memory_read_requests = captured_memory_read_requests
            .lock()
            .expect("Expected captured duplicate pointer preview memory reads.");

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
                CapturedMemoryReadRequest {
                    address: 0x2FF0,
                    module_name: String::new(),
                },
            ]
        );

        assert_eq!(
            ProjectItemTypePointer::get_field_freeze_data_value_interpreter(&opened_project_items[0].1),
            "4660"
        );
        assert_eq!(
            ProjectItemTypePointer::get_field_freeze_data_value_interpreter(&opened_project_items[1].1),
            "4660"
        );
        assert_eq!(
            ProjectItemTypePointer::get_field_evaluated_pointer_path(&opened_project_items[0].1),
            "game.exe+0x1000 -> 0x2020 -> 0x2FF0"
        );
        assert_eq!(
            ProjectItemTypePointer::get_field_evaluated_pointer_path(&opened_project_items[1].1),
            "game.exe+0x1000 -> 0x2020 -> 0x2FF0"
        );
    }

    #[test]
    fn build_project_item_preview_read_definition_truncates_large_fixed_arrays() {
        let engine_execution_context = create_execution_context(MockMemoryReadBindings::new(|_memory_read_request| {
            create_pointer_memory_read_response(0, PointerScanPointerSize::Pointer64, false)
        }));

        let project_item_preview_read_definition =
            build_project_item_preview_read_definition(&engine_execution_context, "u8[128]").expect("Expected preview read definition for fixed array type.");

        assert!(project_item_preview_read_definition.preview_was_truncated);
        assert_eq!(
            project_item_preview_read_definition.symbolic_field_container_type,
            ContainerType::ArrayFixed(128)
        );
        assert!(
            project_item_preview_read_definition
                .layout_key
                .contains("preview")
        );
        assert!(
            project_item_preview_read_definition
                .layout_key
                .contains(&MAX_ARRAY_PREVIEW_ELEMENT_COUNT.to_string())
        );
    }

    #[test]
    fn format_project_item_preview_value_appends_ellipsis_for_truncated_array_reads() {
        let preview_value = format_project_item_preview_value(
            &AnonymousValueString::new("1, 2, 3, ".to_string(), AnonymousValueStringFormat::Decimal, ContainerType::ArrayFixed(3)),
            ContainerType::None,
            true,
        );

        assert_eq!(preview_value, "[1, 2, 3...]");
    }

    #[test]
    fn format_project_item_preview_value_wraps_array_values_in_brackets() {
        let preview_value = format_project_item_preview_value(
            &AnonymousValueString::new("1, 2".to_string(), AnonymousValueStringFormat::Decimal, ContainerType::ArrayFixed(2)),
            ContainerType::None,
            false,
        );

        assert_eq!(preview_value, "[1, 2]");
    }

    #[test]
    fn format_project_item_preview_value_uses_symbolic_container_for_single_element_arrays() {
        let preview_value = format_project_item_preview_value(
            &AnonymousValueString::new("1".to_string(), AnonymousValueStringFormat::Decimal, ContainerType::None),
            ContainerType::ArrayFixed(1),
            false,
        );

        assert_eq!(preview_value, "[1]");
    }

    #[test]
    fn format_project_item_preview_value_truncates_long_array_previews() {
        let long_array_preview = (0..80)
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let preview_value = format_project_item_preview_value(
            &AnonymousValueString::new(long_array_preview, AnonymousValueStringFormat::Decimal, ContainerType::ArrayFixed(80)),
            ContainerType::None,
            false,
        );

        assert!(preview_value.starts_with('['));
        assert!(preview_value.ends_with("...]"));
        assert!(preview_value.len() <= MAX_ARRAY_PREVIEW_CHARACTER_COUNT + 5);
    }

    #[test]
    fn format_project_item_preview_value_does_not_truncate_scalar_previews() {
        let long_scalar_preview = "1234567890".repeat(20);

        let preview_value = format_project_item_preview_value(
            &AnonymousValueString::new(long_scalar_preview.clone(), AnonymousValueStringFormat::Decimal, ContainerType::None),
            ContainerType::None,
            false,
        );

        assert_eq!(preview_value, long_scalar_preview);
    }
}
