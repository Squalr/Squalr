use crate::engine_bindings::engine_ingress::ExecutableRequest;
use crate::engine_bindings::engine_unprivileged_bindings::EngineUnprivilegedBindings;
use crate::engine_execution_context::EngineExecutionContext;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::engine_command::EngineCommand;
use squalr_engine_api::commands::engine_response::EngineResponse;
use squalr_engine_api::events::engine_event::EngineEvent;
use std::sync::Arc;

pub struct IntraProcessUnprivilegedInterface {
    // The instance of the engine privileged state. Since this is an intra-process implementation, we invoke commands using this state directly.
    engine_privileged_state: Option<Arc<EnginePrivilegedState>>,
}

impl EngineUnprivilegedBindings for IntraProcessUnprivilegedInterface {
    fn initialize(
        &mut self,
        engine_privileged_state: &Option<Arc<EnginePrivilegedState>>,
        _engine_execution_context: &Option<Arc<EngineExecutionContext>>,
    ) -> Result<(), String> {
        if let Some(engine_privileged_state) = engine_privileged_state {
            self.engine_privileged_state = Some(engine_privileged_state.clone());
            Ok(())
        } else {
            Err("No privileged state provided! Engine command dispatching will be non-functional without this.".to_string())
        }
    }

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
}

impl IntraProcessUnprivilegedInterface {
    pub fn new<F>(callback: F) -> IntraProcessUnprivilegedInterface
    where
        F: Fn(EngineEvent) + Send + Sync + 'static,
    {
        let instance = IntraProcessUnprivilegedInterface { engine_privileged_state: None };

        instance
    }
}
