use crate::engine_bindings::executable_command_privileged::ExecutableCommandPrivileged;
use crate::engine_bindings::executable_command_unprivileged::ExecutableCommandUnprivleged;
use crate::engine_privileged_state::EnginePrivilegedState;
use crate::general_settings_config::GeneralSettingsConfig;
use crossbeam_channel::Receiver;
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_response::PrivilegedCommandResponse;
use squalr_engine_api::commands::privileged_command_result::PrivilegedCommandResult;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_response::UnprivilegedCommandResponse;
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

pub struct StandaloneEngineApiUnprivilegedBindings {
    // The instance of the engine privileged state. Since this is an intra-process implementation, we invoke commands using this state directly.
    engine_privileged_state: Arc<EnginePrivilegedState>,
}

impl StandaloneEngineApiUnprivilegedBindings {
    /// Initialize unprivileged bindings. For standalone builds, the privileged engine state is passed to allow direct communcation.
    pub fn new(engine_privileged_state: &Arc<EnginePrivilegedState>) -> Self {
        Self {
            engine_privileged_state: engine_privileged_state.clone(),
        }
    }
}

impl EngineApiUnprivilegedBindings for StandaloneEngineApiUnprivilegedBindings {
    /// Dispatches an unprivileged command to be immediately handled on the client side.
    fn dispatch_privileged_command(
        &self,
        privileged_command: PrivilegedCommand,
        callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        let engine_request_delay = GeneralSettingsConfig::get_debug_engine_request_delay_ms();

        // Execute the request either immediately, or on an artificial delay if a debug request delay is set.
        if engine_request_delay <= 0 {
            let privileged_command_result = Self::create_privileged_command_result(&self.engine_privileged_state, privileged_command);
            callback(privileged_command_result.into_privileged_command_response());
        } else {
            let engine_privileged_state = self.engine_privileged_state.clone();

            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(engine_request_delay as u64));
                let privileged_command_result = Self::create_privileged_command_result(&engine_privileged_state, privileged_command);
                callback(privileged_command_result.into_privileged_command_response());
            });
        }

        Ok(())
    }

    /// Dispatches an unprivileged command to be immediately handled on the client side.
    fn dispatch_unprivileged_command(
        &self,
        unprivileged_command: UnprivilegedCommand,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
        callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        let response = unprivileged_command.execute(engine_unprivileged_state);

        callback(response);

        Ok(())
    }

    /// Requests to listen to all engine events.
    fn subscribe_to_engine_events(&self) -> Result<Receiver<squalr_engine_api::engine::engine_event_envelope::EngineEventEnvelope>, EngineBindingError> {
        self.engine_privileged_state.subscribe_to_engine_events()
    }
}

impl StandaloneEngineApiUnprivilegedBindings {
    fn create_privileged_command_result(
        engine_privileged_state: &Arc<EnginePrivilegedState>,
        privileged_command: PrivilegedCommand,
    ) -> PrivilegedCommandResult {
        let should_include_privileged_registry_catalog = privileged_command.should_include_privileged_registry_catalog();
        let privileged_command_response = privileged_command.execute(engine_privileged_state);
        let privileged_registry_catalog = if should_include_privileged_registry_catalog {
            Some(engine_privileged_state.get_privileged_registry_catalog())
        } else {
            None
        };

        PrivilegedCommandResult::new(privileged_command_response, privileged_registry_catalog)
    }
}

#[cfg(test)]
mod tests {
    use super::StandaloneEngineApiUnprivilegedBindings;
    use crate::engine_mode::EngineMode;
    use crate::engine_privileged_state::create_engine_privileged_state_with_os_providers;
    use crossbeam_channel::unbounded;
    use squalr_engine_api::{
        commands::{
            privileged_command_request::PrivilegedCommandRequest,
            privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse},
            process::list::process_list_request::ProcessListRequest,
            registry::get_metadata::{registry_get_metadata_request::RegistryGetMetadataRequest, registry_get_metadata_response::RegistryGetMetadataResponse},
        },
        engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings,
        registries::symbols::data_type_descriptor::DataTypeDescriptor,
        structures::processes::{opened_process_info::OpenedProcessInfo, process_info::ProcessInfo},
    };
    use squalr_engine_session::os::{
        ProcessQueryError, ProcessQueryOptions,
        engine_os_provider::{EngineOsProviders, ProcessQueryProvider},
    };
    use std::sync::Arc;

    struct NoOpProcessQueryProvider;

    impl ProcessQueryProvider for NoOpProcessQueryProvider {
        fn start_monitoring(&self) -> Result<(), ProcessQueryError> {
            Ok(())
        }

        fn get_processes(
            &self,
            _process_query_options: ProcessQueryOptions,
        ) -> Vec<ProcessInfo> {
            vec![]
        }

        fn open_process(
            &self,
            _process_info: &ProcessInfo,
        ) -> Result<OpenedProcessInfo, ProcessQueryError> {
            Err(ProcessQueryError::internal("open_process", "not implemented in no-op provider"))
        }

        fn close_process(
            &self,
            _handle: u64,
        ) -> Result<(), ProcessQueryError> {
            Ok(())
        }
    }

    #[test]
    fn dispatch_registry_command_returns_metadata_in_callback_response() {
        let mut engine_os_providers = EngineOsProviders::default();
        engine_os_providers.process_query = Arc::new(NoOpProcessQueryProvider);
        let engine_privileged_state = create_engine_privileged_state_with_os_providers(EngineMode::Standalone, engine_os_providers)
            .expect("Standalone privileged state should initialize with a no-op process query provider.");
        let standalone_engine_api_unprivileged_bindings = StandaloneEngineApiUnprivilegedBindings::new(&engine_privileged_state);
        let (callback_sender, callback_receiver) = unbounded();

        standalone_engine_api_unprivileged_bindings
            .dispatch_privileged_command(
                RegistryGetMetadataRequest::default().to_engine_command(),
                Box::new(move |response| {
                    let registry_get_metadata_response = RegistryGetMetadataResponse::from_engine_response(response)
                        .expect("Registry command should deserialize into RegistryGetMetadataResponse.");
                    callback_sender
                        .send(
                            registry_get_metadata_response
                                .privileged_registry_catalog
                                .get_data_type_descriptors()
                                .iter()
                                .any(|data_type_descriptor: &DataTypeDescriptor| data_type_descriptor.get_data_type_id() == "remote.test.type"),
                        )
                        .expect("Callback should be able to send assertion result.");
                }),
            )
            .expect("Standalone privileged command dispatch should succeed.");

        assert_eq!(
            callback_receiver
                .recv()
                .expect("Callback should report registry state."),
            false
        );
    }

    #[test]
    fn dispatch_non_registry_command_does_not_attach_privileged_registry_catalog_to_callback_response() {
        let mut engine_os_providers = EngineOsProviders::default();
        engine_os_providers.process_query = Arc::new(NoOpProcessQueryProvider);
        let engine_privileged_state = create_engine_privileged_state_with_os_providers(EngineMode::Standalone, engine_os_providers)
            .expect("Standalone privileged state should initialize with a no-op process query provider.");
        let standalone_engine_api_unprivileged_bindings = StandaloneEngineApiUnprivilegedBindings::new(&engine_privileged_state);
        let (callback_sender, callback_receiver) = unbounded();

        standalone_engine_api_unprivileged_bindings
            .dispatch_privileged_command(
                ProcessListRequest {
                    require_windowed: false,
                    search_name: None,
                    match_case: false,
                    limit: None,
                    fetch_icons: false,
                }
                .to_engine_command(),
                Box::new(move |response| {
                    callback_sender
                        .send(matches!(response, PrivilegedCommandResponse::Process(_)))
                        .expect("Callback should be able to send assertion result.");
                }),
            )
            .expect("Standalone privileged command dispatch should succeed.");

        assert_eq!(
            callback_receiver
                .recv()
                .expect("Callback should report registry state."),
            true
        );
    }
}
