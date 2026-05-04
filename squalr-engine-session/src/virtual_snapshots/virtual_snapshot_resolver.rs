use crate::virtual_snapshots::{virtual_snapshot_query::VirtualSnapshotQuery, virtual_snapshot_query_result::VirtualSnapshotQueryResult};
use squalr_engine_api::{
    commands::{
        memory::read::{memory_read_request::MemoryReadRequest, memory_read_response::MemoryReadResponse},
        privileged_command_request::PrivilegedCommandRequest,
        privileged_command_response::TypedPrivilegedCommandResponse,
    },
    engine::engine_execution_context::EngineExecutionContext,
    structures::{
        memory::pointer::Pointer, pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        structs::symbolic_struct_definition::SymbolicStructDefinition,
    },
};
use std::{
    collections::HashMap,
    sync::{Arc, mpsc},
    time::Duration,
};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct CachedMemoryReadKey {
    address: u64,
    module_name: String,
    layout_key: String,
}

#[derive(Clone, Debug)]
struct PointerQueryEvaluation {
    resolved_target_address: Option<(u64, String)>,
    evaluated_path: String,
}

#[derive(Default)]
struct VirtualSnapshotRefreshSession {
    cached_memory_read_responses: HashMap<CachedMemoryReadKey, Option<MemoryReadResponse>>,
    cached_pointer_query_evaluations: HashMap<Pointer, PointerQueryEvaluation>,
}

pub fn materialize_virtual_snapshot_queries(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    queries: &[VirtualSnapshotQuery],
) -> HashMap<String, VirtualSnapshotQueryResult> {
    let mut query_results = HashMap::new();
    let mut virtual_snapshot_refresh_session = VirtualSnapshotRefreshSession::default();

    for query in queries {
        let query_result = match query {
            VirtualSnapshotQuery::Address {
                address,
                module_name,
                symbolic_struct_definition,
                ..
            } => VirtualSnapshotQueryResult {
                memory_read_response: dispatch_memory_read_request(
                    engine_execution_context,
                    &mut virtual_snapshot_refresh_session,
                    *address,
                    module_name,
                    &build_symbolic_struct_layout_key(symbolic_struct_definition),
                    symbolic_struct_definition,
                ),
                resolved_address: Some(*address),
                resolved_module_name: module_name.clone(),
                evaluated_pointer_path: String::new(),
            },
            VirtualSnapshotQuery::Pointer {
                pointer,
                symbolic_struct_definition,
                ..
            } => {
                let pointer_query_evaluation = evaluate_pointer_query(engine_execution_context, pointer, &mut virtual_snapshot_refresh_session);
                let (_, resolved_module_name) = pointer_query_evaluation
                    .resolved_target_address
                    .clone()
                    .unwrap_or((0, String::new()));

                VirtualSnapshotQueryResult {
                    memory_read_response: pointer_query_evaluation
                        .resolved_target_address
                        .as_ref()
                        .and_then(|(resolved_address, resolved_module_name)| {
                            dispatch_memory_read_request(
                                engine_execution_context,
                                &mut virtual_snapshot_refresh_session,
                                *resolved_address,
                                resolved_module_name,
                                &build_symbolic_struct_layout_key(symbolic_struct_definition),
                                symbolic_struct_definition,
                            )
                        }),
                    resolved_address: pointer_query_evaluation
                        .resolved_target_address
                        .as_ref()
                        .map(|(resolved_address, _)| *resolved_address),
                    resolved_module_name,
                    evaluated_pointer_path: pointer_query_evaluation.evaluated_path,
                }
            }
        };

        query_results.insert(query.get_query_id().to_string(), query_result);
    }

    query_results
}

fn build_symbolic_struct_layout_key(symbolic_struct_definition: &SymbolicStructDefinition) -> String {
    format!("{:?}", symbolic_struct_definition)
}

fn evaluate_pointer_query(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    pointer: &Pointer,
    virtual_snapshot_refresh_session: &mut VirtualSnapshotRefreshSession,
) -> PointerQueryEvaluation {
    if let Some(pointer_query_evaluation) = virtual_snapshot_refresh_session
        .cached_pointer_query_evaluations
        .get(pointer)
    {
        return pointer_query_evaluation.clone();
    }

    let mut evaluated_path_segments = vec![pointer.get_root_display_text()];
    let mut current_address = pointer.get_address();
    let mut current_module_name = pointer.get_module_name().to_string();

    for pointer_chain_segment in pointer.get_offset_segments() {
        let Some(pointer_offset) = pointer_chain_segment.as_offset() else {
            evaluated_path_segments.push(String::from("??"));

            let pointer_query_evaluation = PointerQueryEvaluation {
                resolved_target_address: None,
                evaluated_path: evaluated_path_segments.join(" -> "),
            };

            virtual_snapshot_refresh_session
                .cached_pointer_query_evaluations
                .insert(pointer.clone(), pointer_query_evaluation.clone());

            return pointer_query_evaluation;
        };
        let Some(pointer_value) = read_pointer_value(
            engine_execution_context,
            virtual_snapshot_refresh_session,
            current_address,
            &current_module_name,
            pointer.get_pointer_size(),
        ) else {
            evaluated_path_segments.push(String::from("??"));

            let pointer_query_evaluation = PointerQueryEvaluation {
                resolved_target_address: None,
                evaluated_path: evaluated_path_segments.join(" -> "),
            };

            virtual_snapshot_refresh_session
                .cached_pointer_query_evaluations
                .insert(pointer.clone(), pointer_query_evaluation.clone());

            return pointer_query_evaluation;
        };
        let Some(next_address) = Pointer::apply_pointer_offset(pointer_value, pointer_offset) else {
            evaluated_path_segments.push(String::from("??"));

            let pointer_query_evaluation = PointerQueryEvaluation {
                resolved_target_address: None,
                evaluated_path: evaluated_path_segments.join(" -> "),
            };

            virtual_snapshot_refresh_session
                .cached_pointer_query_evaluations
                .insert(pointer.clone(), pointer_query_evaluation.clone());

            return pointer_query_evaluation;
        };

        current_address = next_address;
        current_module_name.clear();
        evaluated_path_segments.push(format!("0x{:X}", current_address));
    }

    let pointer_query_evaluation = PointerQueryEvaluation {
        resolved_target_address: Some((current_address, current_module_name)),
        evaluated_path: evaluated_path_segments.join(" -> "),
    };

    virtual_snapshot_refresh_session
        .cached_pointer_query_evaluations
        .insert(pointer.clone(), pointer_query_evaluation.clone());

    pointer_query_evaluation
}

fn read_pointer_value(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    virtual_snapshot_refresh_session: &mut VirtualSnapshotRefreshSession,
    address: u64,
    module_name: &str,
    pointer_size: PointerScanPointerSize,
) -> Option<u64> {
    let symbolic_struct_definition = engine_execution_context.resolve_struct_layout_definition(pointer_size.to_data_type_ref().get_data_type_id())?;
    let memory_read_response = dispatch_memory_read_request(
        engine_execution_context,
        virtual_snapshot_refresh_session,
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
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    virtual_snapshot_refresh_session: &mut VirtualSnapshotRefreshSession,
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

    if let Some(cached_memory_read_response) = virtual_snapshot_refresh_session
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

    let dispatch_result = match engine_execution_context.get_bindings().read() {
        Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
            memory_read_command,
            Box::new(move |engine_response| {
                let conversion_result = match MemoryReadResponse::from_engine_response(engine_response) {
                    Ok(memory_read_response) => Ok(memory_read_response),
                    Err(unexpected_response) => Err(format!(
                        "Unexpected response variant for virtual snapshot memory read request: {:?}",
                        unexpected_response
                    )),
                };
                let _ = memory_read_response_sender.send(conversion_result);
            }),
        ),
        Err(error) => {
            log::error!("Failed to acquire engine bindings lock for virtual snapshot memory read request: {}", error);
            return None;
        }
    };

    if let Err(error) = dispatch_result {
        log::error!("Failed to dispatch virtual snapshot memory read request: {}", error);
        return None;
    }

    let memory_read_response = match memory_read_response_receiver.recv_timeout(Duration::from_secs(2)) {
        Ok(Ok(memory_read_response)) => Some(memory_read_response),
        Ok(Err(error)) => {
            log::error!("Failed to convert virtual snapshot memory read response: {}", error);
            None
        }
        Err(error) => {
            log::error!("Timed out waiting for virtual snapshot memory read response: {}", error);
            None
        }
    };

    virtual_snapshot_refresh_session
        .cached_memory_read_responses
        .insert(cached_memory_read_key, memory_read_response.clone());

    memory_read_response
}
