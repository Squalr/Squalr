use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
// use squalr_engine_api::structures::dynamic_struct::dynamic_struct::DynamicStruct;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use std::sync::Arc;

impl EngineCommandRequestExecutor for MemoryReadRequest {
    type ResponseType = MemoryReadResponse;

    fn execute(
        &self,
        execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = execution_context.get_opened_process() {
            log::info!("Reading value from address {}", self.address);

            // let mut out_value = self.value.clone();
            // let success = MemoryReader::get_instance().read_struct(&process_info, self.address, &mut out_value);

            MemoryReadResponse {
                // value: out_value,
                address: self.address,
                // success: success,
                success: false,
            }
        } else {
            log::error!("No opened process available.");

            MemoryReadResponse {
                // value: DynamicStruct::new(),
                address: self.address,
                success: false,
            }
        }
    }
}
