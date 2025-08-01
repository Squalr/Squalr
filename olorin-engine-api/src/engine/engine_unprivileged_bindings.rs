use crate::{
    commands::{engine_command::EngineCommand, engine_command_response::EngineCommandResponse},
    events::engine_event::EngineEvent,
};
use crossbeam_channel::Receiver;

/// Defines the functionality that is invoked from a GUI, CLI, etc. and handled by the engine.
pub trait EngineUnprivilegedBindings: Send + Sync {
    /// Initialize unprivileged bindings.
    fn initialize(&mut self) -> Result<(), String>;

    /// Dispatches an engine command to the engine to handle.
    fn dispatch_command(
        &self,
        engine_command: EngineCommand,
        callback: Box<dyn FnOnce(EngineCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), String>;

    /// Requests to listen to all engine events.
    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String>;
}
