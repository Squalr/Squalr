use crate::engine_privileged_state::EnginePrivilegedState;
use crossbeam_channel::Receiver;
use olorin_engine_api::events::engine_event::EngineEvent;
use std::sync::Arc;

/// Defines functionality that can be invoked by the engine for the GUI or CLI to handle.
pub trait EnginePrivilegedBindings: Send + Sync {
    /// Initialize privileged bindings.
    fn initialize(
        &mut self,
        engine_privileged_state: &Option<Arc<EnginePrivilegedState>>,
    ) -> Result<(), String>;

    /// Emits an event from the engine, sending it to all listeners that are currently subscribed.
    fn emit_event(
        &self,
        event: EngineEvent,
    ) -> Result<(), String>;

    /// Requests to listen to all engine events.
    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String>;
}
