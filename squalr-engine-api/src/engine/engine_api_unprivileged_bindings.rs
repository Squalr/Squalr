use crate::{
    commands::{
        privileged_command::PrivilegedCommand, privileged_command_response::PrivilegedCommandResponse, unprivileged_command::UnprivilegedCommand,
        unprivileged_command_response::UnprivilegedCommandResponse,
    },
    engine::engine_binding_error::EngineBindingError,
    engine::engine_unprivileged_state::EngineUnprivilegedState,
    events::engine_event::EngineEvent,
};
use crossbeam_channel::Receiver;
use std::sync::Arc;

/// Defines the functionality that is invoked from a GUI, CLI, etc. and handled by the engine (gui -> engine).
pub trait EngineApiUnprivilegedBindings: Send + Sync {
    /// Dispatches an engine command to the engine to handle.
    fn dispatch_privileged_command(
        &self,
        engine_command: PrivilegedCommand,
        callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError>;

    /// Dispatches an unprivileged command to be immediately handled on the client side.
    fn dispatch_unprivileged_command(
        &self,
        engine_command: UnprivilegedCommand,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError>;

    /// Requests to listen to all engine events.
    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, EngineBindingError>;
}
