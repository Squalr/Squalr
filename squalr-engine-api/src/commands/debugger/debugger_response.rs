use crate::commands::debugger::{
    attach::debugger_attach_response::DebuggerAttachResponse, breakpoint_list::debugger_breakpoint_list_response::DebuggerBreakpointListResponse,
    breakpoint_remove::debugger_breakpoint_remove_response::DebuggerBreakpointRemoveResponse,
    breakpoint_set::debugger_breakpoint_set_response::DebuggerBreakpointSetResponse, detach::debugger_detach_response::DebuggerDetachResponse,
    pause::debugger_pause_response::DebuggerPauseResponse, register_write::debugger_register_write_response::DebuggerRegisterWriteResponse,
    registers_read::debugger_registers_read_response::DebuggerRegistersReadResponse, resume::debugger_resume_response::DebuggerResumeResponse,
    trace_list::debugger_trace_list_response::DebuggerTraceListResponse, trace_pause::debugger_trace_pause_response::DebuggerTracePauseResponse,
    trace_resume::debugger_trace_resume_response::DebuggerTraceResumeResponse, trace_start::debugger_trace_start_response::DebuggerTraceStartResponse,
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
    TracePause {
        debugger_trace_pause_response: DebuggerTracePauseResponse,
    },
    TraceResume {
        debugger_trace_resume_response: DebuggerTraceResumeResponse,
    },
    TraceList {
        debugger_trace_list_response: DebuggerTraceListResponse,
    },
}

impl DebuggerResponse {
    pub fn failure_message(&self) -> Option<&str> {
        match self {
            Self::Attach {
                debugger_attach_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::Detach {
                debugger_detach_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::Pause {
                debugger_pause_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::Resume {
                debugger_resume_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::BreakpointSet {
                debugger_breakpoint_set_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::BreakpointRemove {
                debugger_breakpoint_remove_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::BreakpointList {
                debugger_breakpoint_list_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::RegistersRead {
                debugger_registers_read_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::RegisterWrite {
                debugger_register_write_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::TraceStart {
                debugger_trace_start_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::TraceStop {
                debugger_trace_stop_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::TracePause {
                debugger_trace_pause_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::TraceResume {
                debugger_trace_resume_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::TraceList {
                debugger_trace_list_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            _ => None,
        }
    }
}
