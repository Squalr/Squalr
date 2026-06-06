use crate::events::debugger::{
    session_state_changed::debugger_session_state_changed_event::DebuggerSessionStateChangedEvent,
    trace_recorded::debugger_trace_recorded_event::DebuggerTraceRecordedEvent,
    trace_session_updated::debugger_trace_session_updated_event::DebuggerTraceSessionUpdatedEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DebuggerEvent {
    SessionStateChanged {
        debugger_session_state_changed_event: DebuggerSessionStateChangedEvent,
    },
    TraceRecorded {
        debugger_trace_recorded_event: DebuggerTraceRecordedEvent,
    },
    TraceSessionUpdated {
        debugger_trace_session_updated_event: DebuggerTraceSessionUpdatedEvent,
    },
}
