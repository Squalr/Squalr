use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::engine::engine_binding_error::EngineBindingError;
use crate::events::engine_event::EngineEvent;
use crossbeam_channel::Receiver;

/// Defines functionality that can be invoked by the engine for the GUI or CLI to handle (engine -> gui).
pub trait EngineApiPrivilegedBindings: Send + Sync {
    /// Emits an event from the engine, sending it to all listeners that are currently subscribed.
    fn emit_event(
        &self,
        event: EngineEvent,
    ) -> Result<(), EngineBindingError>;

    /// Dispatches an engine command to the engine to handle.
    fn dispatch_internal_command(
        &self,
        engine_command: PrivilegedCommand,
        callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError>;

    /// Requests to listen to all engine events.
    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, EngineBindingError>;
}
