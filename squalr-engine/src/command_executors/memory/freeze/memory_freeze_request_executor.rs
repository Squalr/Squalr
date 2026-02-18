use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::memory::freeze::memory_freeze_request::MemoryFreezeRequest;
use squalr_engine_api::commands::memory::freeze::memory_freeze_response::MemoryFreezeResponse;
use squalr_engine_api::structures::memory::pointer::Pointer;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for MemoryFreezeRequest {
    type ResponseType = MemoryFreezeResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let freeze_list_registry = engine_privileged_state.get_freeze_list_registry();
        let mut freeze_list_registry_guard = match freeze_list_registry.write() {
            Ok(freeze_list_registry_guard) => freeze_list_registry_guard,
            Err(error) => {
                log::error!("Failed to acquire write lock on freeze registry for memory freeze request: {}", error);
                return MemoryFreezeResponse {
                    failed_freeze_target_count: self.freeze_targets.len() as u64,
                };
            }
        };

        if !self.is_frozen {
            for freeze_target in &self.freeze_targets {
                let pointer = Pointer::new(freeze_target.address, vec![], freeze_target.module_name.clone());
                freeze_list_registry_guard.set_address_unfrozen(&pointer);
            }

            return MemoryFreezeResponse::default();
        }

        let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            log::warn!("Cannot freeze memory targets without an opened process.");
            return MemoryFreezeResponse {
                failed_freeze_target_count: self.freeze_targets.len() as u64,
            };
        };

        let os_providers = engine_privileged_state.get_os_providers();
        let modules = os_providers.memory_query.get_modules(&opened_process_info);
        let symbol_registry = engine_privileged_state.get_symbol_registry();
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(symbol_registry_guard) => symbol_registry_guard,
            Err(error) => {
                log::error!("Failed to acquire symbol registry read lock for memory freeze request: {}", error);
                return MemoryFreezeResponse {
                    failed_freeze_target_count: self.freeze_targets.len() as u64,
                };
            }
        };
        let mut failed_freeze_target_count = 0u64;

        for freeze_target in &self.freeze_targets {
            let symbolic_struct_definition = match symbol_registry_guard.get(&freeze_target.data_type_id) {
                Some(symbolic_struct_definition) => symbolic_struct_definition,
                None => {
                    failed_freeze_target_count = failed_freeze_target_count.saturating_add(1);
                    continue;
                }
            };

            let mut valued_struct = symbolic_struct_definition.get_default_valued_struct(&symbol_registry);
            let module_base_address = os_providers
                .memory_query
                .resolve_module(&modules, &freeze_target.module_name);
            let absolute_address = module_base_address.saturating_add(freeze_target.address);
            if !os_providers
                .memory_read
                .read_struct(&opened_process_info, absolute_address, &mut valued_struct)
            {
                failed_freeze_target_count = failed_freeze_target_count.saturating_add(1);
                continue;
            }

            let pointer = Pointer::new(freeze_target.address, vec![], freeze_target.module_name.clone());
            freeze_list_registry_guard.set_address_frozen(pointer, valued_struct.get_bytes());
        }

        MemoryFreezeResponse { failed_freeze_target_count }
    }
}
