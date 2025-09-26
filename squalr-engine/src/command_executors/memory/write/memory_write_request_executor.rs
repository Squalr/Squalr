use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_memory::memory_queryer::memory_queryer::MemoryQueryer;
use squalr_engine_memory::memory_queryer::memory_queryer_trait::IMemoryQueryer;
use squalr_engine_memory::memory_writer::MemoryWriter;
use squalr_engine_memory::memory_writer::memory_writer_trait::IMemoryWriter;
use std::sync::Arc;

impl EngineCommandRequestExecutor for MemoryWriteRequest {
    type ResponseType = MemoryWriteResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            if !self.module_name.is_empty() {
                let modules = if let Some(opened_process_info) = engine_privileged_state
                    .get_process_manager()
                    .get_opened_process()
                {
                    MemoryQueryer::get_instance().get_modules(&opened_process_info)
                } else {
                    vec![]
                };
                let module_address = MemoryQueryer::get_instance().resolve_module(&modules, &self.module_name);
                let success = MemoryWriter::get_instance().write_bytes(&process_info, module_address.saturating_add(self.address), &self.value);

                MemoryWriteResponse { success }
            } else {
                let success = MemoryWriter::get_instance().write_bytes(&process_info, self.address, &self.value);

                MemoryWriteResponse { success }
            }
        } else {
            // log::error!("No process is opened to write to.");
            MemoryWriteResponse { success: false }
        }
    }
}
