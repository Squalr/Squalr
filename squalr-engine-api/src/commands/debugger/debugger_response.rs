use crate::commands::debugger::{
    attach::debugger_attach_response::DebuggerAttachResponse, breakpoint_list::debugger_breakpoint_list_response::DebuggerBreakpointListResponse,
    breakpoint_remove::debugger_breakpoint_remove_response::DebuggerBreakpointRemoveResponse,
    breakpoint_set::debugger_breakpoint_set_response::DebuggerBreakpointSetResponse, detach::debugger_detach_response::DebuggerDetachResponse,
    pause::debugger_pause_response::DebuggerPauseResponse, register_write::debugger_register_write_response::DebuggerRegisterWriteResponse,
    registers_read::debugger_registers_read_response::DebuggerRegistersReadResponse, resume::debugger_resume_response::DebuggerResumeResponse,
    trace_list::debugger_trace_list_response::DebuggerTraceListResponse, trace_start::debugger_trace_start_response::DebuggerTraceStartResponse,
    trace_stop::debugger_trace_stop_response::DebuggerTraceStopResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DebuggerResponse {
    Attach {
        debugger_attach_response: DebuggerAttachResponse,
    },
    Detach {
        debugger_detach_response: DebuggerDetachResponse,
    },
    Pause {
        debugger_pause_response: DebuggerPauseResponse,
    },
    Resume {
        debugger_resume_response: DebuggerResumeResponse,
    },
    BreakpointSet {
        debugger_breakpoint_set_response: DebuggerBreakpointSetResponse,
    },
    BreakpointRemove {
        debugger_breakpoint_remove_response: DebuggerBreakpointRemoveResponse,
    },
    BreakpointList {
        debugger_breakpoint_list_response: DebuggerBreakpointListResponse,
    },
    RegistersRead {
        debugger_registers_read_response: DebuggerRegistersReadResponse,
    },
    RegisterWrite {
        debugger_register_write_response: DebuggerRegisterWriteResponse,
    },
    TraceStart {
        debugger_trace_start_response: DebuggerTraceStartResponse,
    },
    TraceStop {
        debugger_trace_stop_response: DebuggerTraceStopResponse,
    },
    TraceList {
        debugger_trace_list_response: DebuggerTraceListResponse,
    },
}
