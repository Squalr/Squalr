use crate::events::debugger::debugger_event::DebuggerEvent;
use crate::events::engine_event::{EngineEvent, EngineEventRequest};
use crate::structures::debugger::DebuggerSessionState;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerSessionStateChangedEvent {
    pub session_state: DebuggerSessionState,
    pub active_plugin_id: Option<String>,
}

impl EngineEventRequest for DebuggerSessionStateChangedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::Debugger(DebuggerEvent::SessionStateChanged {
            debugger_session_state_changed_event: self.clone(),
        })
    }
}
