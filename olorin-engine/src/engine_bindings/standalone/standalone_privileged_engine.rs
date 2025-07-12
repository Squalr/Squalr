use crate::engine_bindings::engine_priviliged_bindings::EnginePrivilegedBindings;
use crate::engine_privileged_state::EnginePrivilegedState;
use crossbeam_channel::{Receiver, Sender};
use olorin_engine_api::events::engine_event::EngineEvent;
use std::sync::{Arc, RwLock};

pub struct StandalonePrivilegedEngine {
    /// The list of subscribers to which we send engine events.
    event_senders: Arc<RwLock<Vec<Sender<EngineEvent>>>>,
}

impl EnginePrivilegedBindings for StandalonePrivilegedEngine {
    fn initialize(
        &mut self,
        _engine_privileged_state: &Option<Arc<EnginePrivilegedState>>,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Emits an event from the engine to all listeners.
    fn emit_event(
        &self,
        engine_event: EngineEvent,
    ) -> Result<(), String> {
        if let Ok(senders) = self.event_senders.read() {
            for sender in senders.iter() {
                if let Err(error) = sender.send(engine_event.clone()) {
                    log::error!("Error emitting engine event: {}", error);
                }
            }
        }

        Ok(())
    }

    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let mut sender_lock = self.event_senders.write().map_err(|error| error.to_string())?;
        sender_lock.push(sender);

        Ok(receiver)
    }
}

impl StandalonePrivilegedEngine {
    pub fn new() -> StandalonePrivilegedEngine {
        let instance = StandalonePrivilegedEngine {
            event_senders: Arc::new(RwLock::new(vec![])),
        };

        instance
    }
}
