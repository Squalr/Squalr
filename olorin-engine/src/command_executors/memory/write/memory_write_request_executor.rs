use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use olorin_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use olorin_engine_memory::memory_writer::MemoryWriter;
use olorin_engine_memory::memory_writer::memory_writer_trait::IMemoryWriter;
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
            log::info!("Writing value {:?} to address {}", self.value, self.address);

            let success = MemoryWriter::get_instance().write_bytes(&process_info, self.address, &self.value);

            MemoryWriteResponse { success }
        } else {
            log::error!("No process is opened to write to.");
            MemoryWriteResponse { success: false }
        }
    }
}
