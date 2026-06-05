use crate::command_executors::privileged_command_executor::PrivilegedCommandExecutor;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::patches::{
    apply::{patch_apply_request::PatchApplyRequest, patch_apply_response::PatchApplyResponse},
    list::{patch_list_request::PatchListRequest, patch_list_response::PatchListResponse},
    no_operation::{patch_no_operation_request::PatchNoOperationRequest, patch_no_operation_response::PatchNoOperationResponse},
    patches_command::PatchesCommand,
    restore::{patch_restore_request::PatchRestoreRequest, patch_restore_response::PatchRestoreResponse},
    restore_address::{patch_restore_address_request::PatchRestoreAddressRequest, patch_restore_address_response::PatchRestoreAddressResponse},
};
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::structures::patches::PatchCommandStatus;
use squalr_engine_api::structures::patches::PatchKind;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::sync::Arc;

const INSTRUCTION_READ_BYTE_COUNT: usize = 16;

fn failure_status(error_message: impl Into<String>) -> PatchCommandStatus {
    PatchCommandStatus::failure(error_message)
}

fn no_opened_process_status() -> PatchCommandStatus {
    failure_status("No opened process to patch.")
}

impl PrivilegedCommandExecutor for PatchesCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
        match self {
            PatchesCommand::Apply { patch_apply_request } => patch_apply_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            PatchesCommand::NoOperation { patch_no_operation_request } => patch_no_operation_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            PatchesCommand::Restore { patch_restore_request } => patch_restore_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            PatchesCommand::RestoreAddress { patch_restore_address_request } => patch_restore_address_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            PatchesCommand::List { patch_list_request } => patch_list_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}

impl PrivilegedCommandRequestExecutor for PatchNoOperationRequest {
    type ResponseType = PatchNoOperationResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return PatchNoOperationResponse {
                status: no_opened_process_status(),
                patch: None,
            };
        };

        let instruction_bytes = match resolve_no_operation_instruction_bytes(self, engine_privileged_state, &opened_process_info) {
            Ok(instruction_bytes) => instruction_bytes,
            Err(error_message) => {
                return PatchNoOperationResponse {
                    status: failure_status(error_message),
                    patch: None,
                };
            }
        };
        let instruction_set_id = opened_process_info
            .get_target_architecture()
            .get_instruction_set_id()
            .to_string();
        let Some(instruction_set) = engine_privileged_state
            .get_plugin_registry()
            .find_instruction_set(&instruction_set_id)
        else {
            return PatchNoOperationResponse {
                status: failure_status(format!(
                    "No instruction set plugin is enabled for target architecture '{}'.",
                    instruction_set_id
                )),
                patch: None,
            };
        };
        let instruction_byte_count = match instruction_set
            .disassemble_block(&instruction_bytes, 0)
            .ok()
            .and_then(|instructions| instructions.into_iter().next())
            .map(|instruction| instruction.length)
            .filter(|instruction_byte_count| *instruction_byte_count > 0 && *instruction_byte_count <= instruction_bytes.len())
        {
            Some(instruction_byte_count) => instruction_byte_count,
            None => {
                return PatchNoOperationResponse {
                    status: failure_status("The instruction length could not be decoded."),
                    patch: None,
                };
            }
        };
        let patched_bytes = match instruction_set.build_no_operation_fill(instruction_byte_count) {
            Ok(patched_bytes) if patched_bytes.len() == instruction_byte_count => patched_bytes,
            Ok(patched_bytes) => {
                return PatchNoOperationResponse {
                    status: failure_status(format!(
                        "Instruction set plugin produced {} no-operation bytes for a {} byte instruction.",
                        patched_bytes.len(),
                        instruction_byte_count
                    )),
                    patch: None,
                };
            }
            Err(error_message) => {
                return PatchNoOperationResponse {
                    status: failure_status(error_message),
                    patch: None,
                };
            }
        };

        if let Err(error_message) = reject_overlapping_software_breakpoint(engine_privileged_state, self.address, &self.module_name, patched_bytes.len()) {
            return PatchNoOperationResponse {
                status: failure_status(error_message),
                patch: None,
            };
        }

        match engine_privileged_state.get_patch_service().apply_patch(
            &opened_process_info,
            engine_privileged_state.get_os_providers(),
            self.address,
            &self.module_name,
            &patched_bytes,
            PatchKind::NoOperation,
            self.label.clone(),
        ) {
            Ok(patch) => PatchNoOperationResponse {
                status: PatchCommandStatus::success(),
                patch: Some(patch),
            },
            Err(error_message) => PatchNoOperationResponse {
                status: failure_status(error_message),
                patch: None,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for PatchApplyRequest {
    type ResponseType = PatchApplyResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return PatchApplyResponse {
                status: no_opened_process_status(),
                patch: None,
            };
        };

        if let Err(error_message) = reject_overlapping_software_breakpoint(engine_privileged_state, self.address, &self.module_name, self.patched_bytes.len()) {
            return PatchApplyResponse {
                status: failure_status(error_message),
                patch: None,
            };
        }

        match engine_privileged_state.get_patch_service().apply_patch(
            &opened_process_info,
            engine_privileged_state.get_os_providers(),
            self.address,
            &self.module_name,
            &self.patched_bytes,
            self.kind,
            self.label.clone(),
        ) {
            Ok(patch) => PatchApplyResponse {
                status: PatchCommandStatus::success(),
                patch: Some(patch),
            },
            Err(error_message) => PatchApplyResponse {
                status: failure_status(error_message),
                patch: None,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for PatchRestoreRequest {
    type ResponseType = PatchRestoreResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return PatchRestoreResponse {
                status: no_opened_process_status(),
                patch: None,
            };
        };

        match engine_privileged_state
            .get_patch_service()
            .restore_patch(&opened_process_info, engine_privileged_state.get_os_providers(), &self.patch_id)
        {
            Ok(patch) => PatchRestoreResponse {
                status: PatchCommandStatus::success(),
                patch: Some(patch),
            },
            Err(error_message) => PatchRestoreResponse {
                status: failure_status(error_message),
                patch: None,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for PatchRestoreAddressRequest {
    type ResponseType = PatchRestoreAddressResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return PatchRestoreAddressResponse {
                status: no_opened_process_status(),
                patch: None,
            };
        };

        match engine_privileged_state
            .get_patch_service()
            .restore_patch_at_address(
                &opened_process_info,
                engine_privileged_state.get_os_providers(),
                self.address,
                &self.module_name,
                self.expected_kind,
            ) {
            Ok(patch) => PatchRestoreAddressResponse {
                status: PatchCommandStatus::success(),
                patch: Some(patch),
            },
            Err(error_message) => PatchRestoreAddressResponse {
                status: failure_status(error_message),
                patch: None,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for PatchListRequest {
    type ResponseType = PatchListResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return PatchListResponse {
                status: no_opened_process_status(),
                patches: Vec::new(),
            };
        };

        match engine_privileged_state
            .get_patch_service()
            .list_patches(&opened_process_info)
        {
            Ok(patches) => PatchListResponse {
                status: PatchCommandStatus::success(),
                patches,
            },
            Err(error_message) => PatchListResponse {
                status: failure_status(error_message),
                patches: Vec::new(),
            },
        }
    }
}

fn reject_overlapping_software_breakpoint(
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    address: u64,
    module_name: &str,
    patch_size: usize,
) -> Result<(), String> {
    let software_breakpoints = match engine_privileged_state
        .get_debugger_service()
        .list_breakpoints()
    {
        Ok(breakpoints) => breakpoints
            .into_iter()
            .filter(|breakpoint| {
                breakpoint.get_is_enabled() && matches!(breakpoint.get_kind(), squalr_engine_api::structures::debugger::DebuggerBreakpointKind::Software)
            })
            .collect::<Vec<_>>(),
        Err(error_message) if error_message == "No active debugger session." => Vec::new(),
        Err(error_message) => {
            return Err(format!(
                "Patch range could not be checked against active software breakpoints: {}",
                error_message
            ));
        }
    };

    if software_breakpoints.is_empty() {
        return Ok(());
    }

    let Some(opened_process_info) = engine_privileged_state
        .get_process_manager()
        .get_opened_process()
    else {
        return Ok(());
    };
    let absolute_address = resolve_absolute_address(engine_privileged_state, &opened_process_info, address, module_name)?;
    let patch_end_address = absolute_address.saturating_add(patch_size as u64);

    for breakpoint in software_breakpoints {
        let breakpoint_address = breakpoint.get_address();
        let breakpoint_end_address = breakpoint_address.saturating_add(1);

        if absolute_address < breakpoint_end_address && breakpoint_address < patch_end_address {
            return Err(format!(
                "Patch range 0x{:X}-0x{:X} overlaps enabled software breakpoint '{}'.",
                absolute_address,
                patch_end_address,
                breakpoint.get_breakpoint_id()
            ));
        }
    }

    Ok(())
}

fn resolve_no_operation_instruction_bytes(
    patch_no_operation_request: &PatchNoOperationRequest,
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    opened_process_info: &OpenedProcessInfo,
) -> Result<Vec<u8>, String> {
    if let Some(instruction_bytes_hint) = patch_no_operation_request
        .instruction_bytes_hint
        .as_ref()
        .filter(|instruction_bytes_hint| !instruction_bytes_hint.is_empty())
    {
        return Ok(instruction_bytes_hint.clone());
    }

    let absolute_address = resolve_absolute_address(
        engine_privileged_state,
        opened_process_info,
        patch_no_operation_request.address,
        &patch_no_operation_request.module_name,
    )?;
    let mut instruction_bytes = vec![0_u8; INSTRUCTION_READ_BYTE_COUNT];
    if !engine_privileged_state
        .get_os_providers()
        .memory_read
        .read_bytes(opened_process_info, absolute_address, &mut instruction_bytes)
    {
        return Err(format!("Failed to read instruction bytes at 0x{:X}.", absolute_address));
    }

    Ok(instruction_bytes)
}

fn resolve_absolute_address(
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    opened_process_info: &OpenedProcessInfo,
    address: u64,
    module_name: &str,
) -> Result<u64, String> {
    if module_name.trim().is_empty() {
        return Ok(address);
    }

    let modules = engine_privileged_state
        .get_os_providers()
        .memory_query
        .get_modules(opened_process_info);
    let Some(module) = modules
        .iter()
        .find(|module| module.get_module_name().eq_ignore_ascii_case(module_name))
    else {
        return Err(format!("Module '{}' is not loaded in the opened process.", module_name));
    };

    module
        .get_base_address()
        .checked_add(address)
        .ok_or_else(|| format!("Module-relative address {}+0x{:X} overflowed.", module_name, address))
}
