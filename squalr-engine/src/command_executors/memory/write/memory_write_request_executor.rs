use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_memory::memory_writer::MemoryWriter;
use squalr_engine_memory::memory_writer::memory_writer_trait::IMemoryWriter;
use std::sync::Arc;

impl EngineCommandRequestExecutor for MemoryWriteRequest {
    type ResponseType = MemoryWriteResponse;

    fn execute(
        &self,
        execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = execution_context.get_opened_process() {
            /*
            // Log the memory write operation
            log::info!("Writing value {:?} to address {}", self.value, self.address);

            // Convert value to bytes and write to memory
            let value_bytes = self.value.to_bytes();

            // Perform the memory write operation
            let success = MemoryWriter::get_instance().write_bytes(process_info.handle, self.address, &value_bytes);

            MemoryWriteResponse { success }*/
            MemoryWriteResponse { success: false }
        } else {
            log::error!("No process is opened to write to.");
            MemoryWriteResponse { success: false }
        }
    }
}
