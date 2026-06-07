use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::memory::write::memory_write_request::{MemoryWriteMode, MemoryWriteRequest};
use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_api::structures::debugger::DebuggerSessionState;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::sync::Arc;

fn run_checked_code_write_with_debugger_paused(
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    opened_process_info: &OpenedProcessInfo,
    code_write: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    let debugger_service = engine_privileged_state.get_debugger_service();
    let should_resume_after_write = debugger_service.get_active_session_state_for_process(opened_process_info) == Some(DebuggerSessionState::Running);

    if should_resume_after_write {
        debugger_service
            .pause()
            .map_err(|error_message| format!("Failed to pause debugger before checked code write: {}", error_message))?;
    }

    let write_result = code_write();
    let resume_result = if should_resume_after_write {
        debugger_service
            .resume()
            .map(|_| ())
            .map_err(|error_message| format!("Failed to resume debugger after checked code write: {}", error_message))
    } else {
        Ok(())
    };

    match (write_result, resume_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error_message), _) => Err(error_message),
        (Ok(()), Err(error_message)) => Err(error_message),
    }
}

impl PrivilegedCommandRequestExecutor for MemoryWriteRequest {
    type ResponseType = MemoryWriteResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            let os_providers = engine_privileged_state.get_os_providers();
            if self.write_mode == MemoryWriteMode::CheckedCode {
                return match run_checked_code_write_with_debugger_paused(engine_privileged_state, &process_info, || {
                    engine_privileged_state
                        .get_patch_service()
                        .write_code_bytes_checked(&process_info, os_providers, self.address, &self.module_name, &self.value)
                }) {
                    Ok(()) => MemoryWriteResponse { success: true, error: None },
                    Err(error) => {
                        log::warn!("Checked code write failed: {}", error);
                        MemoryWriteResponse {
                            success: false,
                            error: Some(error),
                        }
                    }
                };
            }

            if !self.module_name.is_empty() {
                let modules = if let Some(opened_process_info) = engine_privileged_state
                    .get_process_manager()
                    .get_opened_process()
                {
                    os_providers.memory_query.get_modules(&opened_process_info)
                } else {
                    vec![]
                };
                let module_address = os_providers
                    .memory_query
                    .resolve_module_address(&modules, &self.module_name, self.address);
                let success = os_providers
                    .memory_write
                    .write_bytes(&process_info, module_address.unwrap_or(0), &self.value);

                MemoryWriteResponse {
                    success: module_address.is_some() && success,
                    error: if module_address.is_some() && success {
                        None
                    } else {
                        Some(String::from("Memory write failed."))
                    },
                }
            } else {
                let success = os_providers
                    .memory_write
                    .write_bytes(&process_info, self.address, &self.value);

                MemoryWriteResponse {
                    success,
                    error: if success { None } else { Some(String::from("Memory write failed.")) },
                }
            }
        } else {
            // log::error!("No process is opened to write to.");
            MemoryWriteResponse {
                success: false,
                error: Some(String::from("No process is opened to write to.")),
            }
        }
    }
}
