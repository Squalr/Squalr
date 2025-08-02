use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_response::EngineCommandResponse;
use crate::events::engine_event::EngineEvent;
use crossbeam_channel::Receiver;

/// Defines functionality that can be invoked by the engine for the GUI or CLI to handle.
pub trait EngineApiPrivilegedBindings: Send + Sync {
    /// Emits an event from the engine, sending it to all listeners that are currently subscribed.
    fn emit_event(
        &self,
        event: EngineEvent,
    ) -> Result<(), String>;

    /// Dispatches an engine command to the engine to handle.
    fn dispatch_command(
        &self,
        engine_command: EngineCommand,
        callback: Box<dyn FnOnce(EngineCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), String>;

    /// Requests to listen to all engine events.
    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String>;
}
