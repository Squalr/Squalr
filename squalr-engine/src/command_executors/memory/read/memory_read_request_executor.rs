use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
use squalr_engine_api::structures::structs::symbolic_struct_ref::SymbolicStructRef;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for MemoryReadRequest {
    type ResponseType = MemoryReadResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            let os_providers = engine_privileged_state.get_os_providers();

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
                    os_providers.memory_query.get_modules(&opened_process_info)
                } else {
                    vec![]
                };
                let module_address = os_providers
                    .memory_query
                    .resolve_module(&modules, &self.module_name);
                let success = os_providers
                    .memory_read
                    .read_struct(&process_info, module_address.saturating_add(self.address), &mut out_valued_struct);

                MemoryReadResponse {
                    valued_struct: out_valued_struct,
                    address: self.address,
                    success,
                }
            } else {
                let success = os_providers
                    .memory_read
                    .read_struct(&process_info, self.address, &mut out_valued_struct);

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
