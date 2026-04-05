use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use std::sync::Arc;

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
                }
            } else {
                let success = os_providers
                    .memory_write
                    .write_bytes(&process_info, self.address, &self.value);

                MemoryWriteResponse { success }
            }
        } else {
            // log::error!("No process is opened to write to.");
            MemoryWriteResponse { success: false }
        }
    }
}
