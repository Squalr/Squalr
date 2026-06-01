use crate::events::debugger::{
    session_state_changed::debugger_session_state_changed_event::DebuggerSessionStateChangedEvent,
    trace_recorded::debugger_trace_recorded_event::DebuggerTraceRecordedEvent,
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
}
