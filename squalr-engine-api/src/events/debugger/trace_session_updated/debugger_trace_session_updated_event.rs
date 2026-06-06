use crate::events::debugger::debugger_event::DebuggerEvent;
use crate::events::engine_event::{EngineEvent, EngineEventRequest};
use crate::structures::debugger::{DebuggerTraceInstructionRecord, DebuggerTraceSessionDescriptor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerTraceSessionUpdatedEvent {
    pub trace_session: DebuggerTraceSessionDescriptor,
    pub instruction_records: Vec<DebuggerTraceInstructionRecord>,
}

impl EngineEventRequest for DebuggerTraceSessionUpdatedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::Debugger(DebuggerEvent::TraceSessionUpdated {
            debugger_trace_session_updated_event: self.clone(),
        })
    }
}
