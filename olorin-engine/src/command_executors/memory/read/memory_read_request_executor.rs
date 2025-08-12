use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use olorin_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
use olorin_engine_api::structures::structs::symbolic_struct_ref::SymbolicStructRef;
use olorin_engine_api::structures::structs::valued_struct::ValuedStruct;
use olorin_engine_memory::memory_queryer::memory_queryer::MemoryQueryer;
use olorin_engine_memory::memory_queryer::memory_queryer_trait::IMemoryQueryer;
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

            let symbol_registry = engine_privileged_state.get_registries().get_symbol_registry();
            let mut out_valued_struct = self
                .symbolic_struct_definition
                .get_default_valued_struct(&symbol_registry);

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
                let success = MemoryReader::get_instance().read_struct(&process_info, module_address.saturating_add(self.address), &mut out_valued_struct);

                MemoryReadResponse {
                    valued_struct: out_valued_struct,
                    address: self.address,
                    success,
                }
            } else {
                let success = MemoryReader::get_instance().read_struct(&process_info, self.address, &mut out_valued_struct);

                MemoryReadResponse {
                    valued_struct: out_valued_struct,
                    address: self.address,
                    success,
                }
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
