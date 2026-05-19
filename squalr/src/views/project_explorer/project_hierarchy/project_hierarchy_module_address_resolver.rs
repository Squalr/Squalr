use squalr_engine_api::commands::memory::query::memory_query_request::MemoryQueryRequest;
use squalr_engine_api::commands::memory::query::memory_query_response::MemoryQueryResponse;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::memory::address_display::try_resolve_virtual_module_address;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

pub struct ProjectHierarchyModuleAddressResolver;

impl ProjectHierarchyModuleAddressResolver {
    pub fn resolve_pointer_scanner_target(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        address: u64,
        module_name: &str,
    ) -> (u64, String) {
        if module_name.trim().is_empty() {
            return (address, String::new());
        }

        if try_resolve_virtual_module_address(module_name, address).is_some() {
            return (address, module_name.to_string());
        }

        if let Some(resolved_absolute_address) = Self::dispatch_memory_query_request(engine_unprivileged_state)
            .and_then(|memory_query_response| Self::resolve_module_relative_address(&memory_query_response.modules, address, module_name))
        {
            return (resolved_absolute_address, String::new());
        }

        log::warn!(
            "Failed to resolve pointer scanner target for module-relative address {}+0x{:X}; falling back to unresolved offset.",
            module_name,
            address
        );

        (address, module_name.to_string())
    }

    fn dispatch_memory_query_request(engine_unprivileged_state: &Arc<EngineUnprivilegedState>) -> Option<MemoryQueryResponse> {
        let memory_query_request = MemoryQueryRequest::default();
        let memory_query_command = memory_query_request.to_engine_command();
        let (memory_query_response_sender, memory_query_response_receiver) = mpsc::channel();

        let dispatch_result = match engine_unprivileged_state.get_bindings().read() {
            Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
                memory_query_command,
                Box::new(move |engine_response| {
                    let conversion_result = match MemoryQueryResponse::from_engine_response(engine_response) {
                        Ok(memory_query_response) => Ok(memory_query_response),
                        Err(unexpected_response) => Err(format!(
                            "Unexpected response variant for project hierarchy memory query request: {:?}",
                            unexpected_response
                        )),
                    };
                    let _ = memory_query_response_sender.send(conversion_result);
                }),
            ),
            Err(error) => {
                log::error!("Failed to acquire engine bindings lock for project hierarchy memory query request: {}", error);
                return None;
            }
        };

        if let Err(error) = dispatch_result {
            log::error!("Failed to dispatch project hierarchy memory query request: {}", error);
            return None;
        }

        match memory_query_response_receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(memory_query_response)) => Some(memory_query_response),
            Ok(Err(error)) => {
                log::error!("Failed to convert project hierarchy memory query response: {}", error);
                None
            }
            Err(error) => {
                log::error!("Timed out waiting for project hierarchy memory query response: {}", error);
                None
            }
        }
    }

    fn resolve_module_relative_address(
        modules: &[NormalizedModule],
        address: u64,
        module_name: &str,
    ) -> Option<u64> {
        modules
            .iter()
            .find(|normalized_module| {
                normalized_module
                    .get_module_name()
                    .eq_ignore_ascii_case(module_name)
            })
            .and_then(|normalized_module| normalized_module.get_base_address().checked_add(address))
    }
}
