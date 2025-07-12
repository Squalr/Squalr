use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use olorin_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
use olorin_engine_api::structures::structs::symbolic_struct_ref::SymbolicStructRef;
use olorin_engine_api::structures::structs::valued_struct::ValuedStruct;
use olorin_engine_memory::memory_reader::MemoryReader;
use olorin_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use std::sync::Arc;

impl EngineCommandRequestExecutor for MemoryReadRequest {
    type ResponseType = MemoryReadResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            log::info!("Reading value from address {}", self.address);

            let mut out_valued_struct = self.valued_struct.clone();
            let success = MemoryReader::get_instance().read_struct(&process_info, self.address, &mut out_valued_struct);

            MemoryReadResponse {
                valued_struct: out_valued_struct,
                address: self.address,
                success,
            }
        } else {
            log::error!("No opened process available.");

            MemoryReadResponse {
                valued_struct: ValuedStruct::new(SymbolicStructRef::new(String::new()), vec![]),
                address: self.address,
                success: false,
            }
        }
    }
}
