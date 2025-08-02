use crate::engine_bindings::engine_ingress::ExecutableCommand;
use crate::engine_privileged_state::EnginePrivilegedState;
use crossbeam_channel::Receiver;
use olorin_engine_api::commands::engine_command::EngineCommand;
use olorin_engine_api::commands::engine_command_response::EngineCommandResponse;
use olorin_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use olorin_engine_api::events::engine_event::EngineEvent;
use std::sync::Arc;

pub struct StandaloneEngineApiUnprivilegedBindings {
    // The instance of the engine privileged state. Since this is an intra-process implementation, we invoke commands using this state directly.
    engine_privileged_state: Option<Arc<EnginePrivilegedState>>,
}

impl StandaloneEngineApiUnprivilegedBindings {
    /// Initialize unprivileged bindings. For standalone builds, the privileged engine state is passed to allow direct communcation.
    pub fn new(engine_privileged_state: &Option<Arc<EnginePrivilegedState>>) -> Self {
        let engine_privileged_state = if let Some(engine_privileged_state) = engine_privileged_state {
            Some(engine_privileged_state.clone())
        } else {
            log::error!("No privileged state provided! Engine command dispatching will be non-functional without this.");

            None
        };

        Self { engine_privileged_state }
    }
}

impl EngineApiUnprivilegedBindings for StandaloneEngineApiUnprivilegedBindings {
    /// Dispatches an engine command to the engine to handle.
    fn dispatch_command(
        &self,
        command: EngineCommand,
        callback: Box<dyn FnOnce(EngineCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), String> {
        if let Some(engine_privileged_state) = &self.engine_privileged_state {
            callback(command.execute(engine_privileged_state));
        }

        Ok(())
    }

    /// Requests to listen to all engine events.
    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String> {
        // If we are in standalone mode, then we can just directly subscribe to the engine events.
        if let Some(engine_privileged_state) = &self.engine_privileged_state {
            engine_privileged_state.subscribe_to_engine_events()
        } else {
            Err("Failed to subscribe to engine events.".to_string())
        }
    }
}
