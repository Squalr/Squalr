use crate::events::debugger::debugger_event::DebuggerEvent;
use crate::events::engine_event::{EngineEvent, EngineEventRequest};
use crate::structures::debugger::DebuggerTraceEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerTraceRecordedEvent {
    pub trace_event: DebuggerTraceEvent,
}

impl EngineEventRequest for DebuggerTraceRecordedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::Debugger(DebuggerEvent::TraceRecorded {
            debugger_trace_recorded_event: self.clone(),
        })
    }
}
