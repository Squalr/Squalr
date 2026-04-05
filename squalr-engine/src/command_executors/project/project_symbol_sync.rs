use crossbeam_channel::bounded;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::registry::set_project_symbols::registry_set_project_symbols_request::RegistrySetProjectSymbolsRequest;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use std::{sync::Arc, time::Duration};

pub fn sync_project_symbol_catalog(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    project_symbol_catalog: ProjectSymbolCatalog,
) -> bool {
    let registry_set_project_symbols_request = RegistrySetProjectSymbolsRequest { project_symbol_catalog };
    let (completion_sender, completion_receiver) = bounded(1);
    let did_send = match engine_execution_context.get_bindings().read() {
        Ok(engine_bindings) => registry_set_project_symbols_request.send_unprivileged(&*engine_bindings, move |registry_set_project_symbols_response| {
            let _ = completion_sender.send(registry_set_project_symbols_response.success);
        }),
        Err(error) => {
            log::error!("Failed to acquire engine bindings while syncing project symbol catalog: {}", error);
            false
        }
    };

    if !did_send {
        return false;
    }

    match completion_receiver.recv_timeout(Duration::from_secs(1)) {
        Ok(success) => success,
        Err(error) => {
            log::error!("Timed out waiting for project symbol catalog sync response: {}", error);
            false
        }
    }
}
