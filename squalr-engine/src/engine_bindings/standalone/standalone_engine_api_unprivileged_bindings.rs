use crate::engine_bindings::executable_command_privileged::ExecutableCommandPrivileged;
use crate::engine_bindings::executable_command_unprivileged::ExecutableCommandUnprivleged;
use crate::engine_privileged_state::EnginePrivilegedState;
use crate::general_settings_config::GeneralSettingsConfig;
use crossbeam_channel::Receiver;
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_response::PrivilegedCommandResponse;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_response::UnprivilegedCommandResponse;
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
use squalr_engine_api::engine::engine_unprivileged_state::EngineUnprivilegedState;
use squalr_engine_api::events::engine_event::EngineEvent;
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
        let engine_request_delay = GeneralSettingsConfig::get_engine_request_delay_ms();

        // Execute the request either immediately, or on an artificial delay if a debug request delay is set.
        if engine_request_delay <= 0 {
            callback(privileged_command.execute(&self.engine_privileged_state));
        } else {
            let engine_privileged_state = self.engine_privileged_state.clone();

            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(engine_request_delay as u64));
                let response = privileged_command.execute(&engine_privileged_state);
                callback(response);
            });
        }

        Ok(())
    }

    /// Dispatches an unprivileged command to be immediately handled on the client side.
    fn dispatch_unprivileged_command(
        &self,
        unprivileged_command: UnprivilegedCommand,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        let response = unprivileged_command.execute(engine_unprivileged_state);

        callback(response);

        Ok(())
    }

    /// Requests to listen to all engine events.
    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, EngineBindingError> {
        self.engine_privileged_state.subscribe_to_engine_events()
    }
}
