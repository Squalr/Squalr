use crate::engine_bindings::engine_priviliged_bindings::EnginePrivilegedBindings;
use crate::engine_execution_context::EngineExecutionContext;
use crate::engine_privileged_state::EnginePrivilegedState;
use crossbeam_channel::{Receiver, Sender};
use squalr_engine_api::events::engine_event::EngineEvent;
use std::sync::{Arc, RwLock};

pub struct IntraProcessPrivilegedEngine {
    // The instance of the engine unprivileged state. Since this is an intra-process implementation, we send events using this state directly.
    engine_execution_context: Option<Arc<EngineExecutionContext>>,

    /// The list of subscribers to which we send engine events.
    event_senders: Arc<RwLock<Vec<Sender<EngineEvent>>>>,
}

impl EnginePrivilegedBindings for IntraProcessPrivilegedEngine {
    fn initialize(
        &mut self,
        _engine_privileged_state: &Option<Arc<EnginePrivilegedState>>,
        engine_execution_context: &Option<Arc<EngineExecutionContext>>,
    ) -> Result<(), String> {
        if let Some(engine_execution_context) = engine_execution_context {
            self.engine_execution_context = Some(engine_execution_context.clone());
            Ok(())
        } else {
            Err("No engine execution context provided! Engine event dispatching will be non-functional without this.".to_string())
        }
    }

    /// Emits an event from the engine to all listeners.
    fn emit_event(
        &self,
        engine_event: EngineEvent,
    ) -> Result<(), String> {
        if let Ok(senders) = self.event_senders.read() {
            for sender in senders.iter() {
                if let Err(err) = sender.send(engine_event.clone()) {
                    log::error!("Error emitting engine event: {}", err);
                }
            }
        }

        Ok(())
    }

    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let mut sender_lock = self.event_senders.write().map_err(|err| err.to_string())?;
        sender_lock.push(sender);

        Ok(receiver)
    }
}

impl IntraProcessPrivilegedEngine {
    pub fn new() -> IntraProcessPrivilegedEngine {
        let instance = IntraProcessPrivilegedEngine {
            engine_execution_context: None,
            event_senders: Arc::new(RwLock::new(vec![])),
        };

        instance
    }
}
