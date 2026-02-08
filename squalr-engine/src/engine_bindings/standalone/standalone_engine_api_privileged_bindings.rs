use crate::{engine_bindings::executable_command_privileged::ExecutableCommandPrivileged, engine_privileged_state::EnginePrivilegedState};
use crossbeam_channel::{Receiver, Sender};
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_response::PrivilegedCommandResponse;
use squalr_engine_api::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
use squalr_engine_api::events::engine_event::EngineEvent;
use std::sync::{Arc, RwLock};

pub struct StandalonePrivilegedEngine {
    /// The list of subscribers to which we send engine events.
    event_senders: Arc<RwLock<Vec<Sender<EngineEvent>>>>,

    engine_privileged_state: Option<Arc<EnginePrivilegedState>>,
}

impl EngineApiPrivilegedBindings for StandalonePrivilegedEngine {
    /// Emits an event from the engine to all listeners.
    fn emit_event(
        &self,
        engine_event: EngineEvent,
    ) -> Result<(), EngineBindingError> {
        if let Ok(senders) = self.event_senders.read() {
            for sender in senders.iter() {
                if let Err(error) = sender.send(engine_event.clone()) {
                    log::error!("Error emitting engine event: {}", error);
                }
            }
        }

        Ok(())
    }

    fn dispatch_internal_command(
        &self,
        privileged_command: PrivilegedCommand,
        callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        // For standalone builds, we can instantly execute the command and return the response.
        if let Some(engine_privileged_state) = &self.engine_privileged_state {
            let response = privileged_command.execute(&engine_privileged_state);

            callback(response);

            Ok(())
        } else {
            Err(EngineBindingError::unavailable("dispatching internal command in standalone mode"))
        }
    }

    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, EngineBindingError> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let mut sender_lock = self
            .event_senders
            .write()
            .map_err(|error| EngineBindingError::lock_failure("subscribing to standalone engine events", error.to_string()))?;
        sender_lock.push(sender);

        Ok(receiver)
    }
}

impl StandalonePrivilegedEngine {
    pub fn new() -> StandalonePrivilegedEngine {
        let instance = StandalonePrivilegedEngine {
            event_senders: Arc::new(RwLock::new(vec![])),
            engine_privileged_state: None,
        };

        instance
    }

    pub fn initialize(
        &mut self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> Result<(), EngineBindingError> {
        self.engine_privileged_state = Some(engine_privileged_state.clone());

        Ok(())
    }
}
