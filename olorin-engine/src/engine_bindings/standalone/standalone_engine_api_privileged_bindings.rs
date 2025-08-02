use crossbeam_channel::{Receiver, Sender};
use olorin_engine_api::commands::engine_command::EngineCommand;
use olorin_engine_api::commands::engine_command_response::EngineCommandResponse;
use olorin_engine_api::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use olorin_engine_api::events::engine_event::EngineEvent;
use std::sync::{Arc, RwLock};

pub struct StandalonePrivilegedEngine {
    /// The list of subscribers to which we send engine events.
    event_senders: Arc<RwLock<Vec<Sender<EngineEvent>>>>,
}

impl EngineApiPrivilegedBindings for StandalonePrivilegedEngine {
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

    fn dispatch_command(
        &self,
        engine_command: EngineCommand,
        callback: Box<dyn FnOnce(EngineCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), String> {
        Err("haha".to_string())
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
