use crate::command_executors::privileged_command_executor::PrivilegedCommandExecutor;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::patches::{
    apply::{patch_apply_request::PatchApplyRequest, patch_apply_response::PatchApplyResponse},
    list::{patch_list_request::PatchListRequest, patch_list_response::PatchListResponse},
    patches_command::PatchesCommand,
    restore::{patch_restore_request::PatchRestoreRequest, patch_restore_response::PatchRestoreResponse},
    restore_address::{patch_restore_address_request::PatchRestoreAddressRequest, patch_restore_address_response::PatchRestoreAddressResponse},
};
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::structures::patches::PatchCommandStatus;
use std::sync::Arc;

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
        Err(_) => Vec::new(),
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
    let absolute_address = if module_name.trim().is_empty() {
        address
    } else {
        let modules = engine_privileged_state
            .get_os_providers()
            .memory_query
            .get_modules(&opened_process_info);
        let Some(module) = modules
            .iter()
            .find(|module| module.get_module_name().eq_ignore_ascii_case(module_name))
        else {
            return Ok(());
        };

        module.get_base_address().saturating_add(address)
    };
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
