use crate::engine_bindings::engine_ingress::ExecutableCommand;
use crate::engine_bindings::engine_unprivileged_bindings::EngineUnprivilegedBindings;
use crate::engine_privileged_state::EnginePrivilegedState;
use crossbeam_channel::Receiver;
use squalr_engine_api::commands::engine_command::EngineCommand;
use squalr_engine_api::commands::engine_response::EngineResponse;
use squalr_engine_api::events::engine_event::EngineEvent;
use std::sync::Arc;

pub struct StandaloneUnprivilegedInterface {
    // The instance of the engine privileged state. Since this is an intra-process implementation, we invoke commands using this state directly.
    engine_privileged_state: Option<Arc<EnginePrivilegedState>>,
}

impl EngineUnprivilegedBindings for StandaloneUnprivilegedInterface {
    /// Initialize unprivileged bindings. For standalone builds, the privileged engine state is passed to allow direct communcation.
    fn initialize(
        &mut self,
        engine_privileged_state: &Option<Arc<EnginePrivilegedState>>,
    ) -> Result<(), String> {
        if let Some(engine_privileged_state) = engine_privileged_state {
            self.engine_privileged_state = Some(engine_privileged_state.clone());
            Ok(())
        } else {
            Err("No privileged state provided! Engine command dispatching will be non-functional without this.".to_string())
        }
    }

    /// Dispatches an engine command to the engine to handle.
    fn dispatch_command(
        &self,
        command: EngineCommand,
        callback: Box<dyn FnOnce(EngineResponse) + Send + Sync + 'static>,
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

impl StandaloneUnprivilegedInterface {
    pub fn new() -> StandaloneUnprivilegedInterface {
        let instance = StandaloneUnprivilegedInterface { engine_privileged_state: None };

        instance
    }
}
