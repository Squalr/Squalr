use crate::commands::debugger::{
    attach::debugger_attach_request::DebuggerAttachRequest, breakpoint_list::debugger_breakpoint_list_request::DebuggerBreakpointListRequest,
    breakpoint_remove::debugger_breakpoint_remove_request::DebuggerBreakpointRemoveRequest,
    breakpoint_set::debugger_breakpoint_set_request::DebuggerBreakpointSetRequest, detach::debugger_detach_request::DebuggerDetachRequest,
    pause::debugger_pause_request::DebuggerPauseRequest, register_write::debugger_register_write_request::DebuggerRegisterWriteRequest,
    registers_read::debugger_registers_read_request::DebuggerRegistersReadRequest, resume::debugger_resume_request::DebuggerResumeRequest,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DebuggerCommand {
    Attach {
        debugger_attach_request: DebuggerAttachRequest,
    },
    Detach {
        debugger_detach_request: DebuggerDetachRequest,
    },
    Pause {
        debugger_pause_request: DebuggerPauseRequest,
    },
    Resume {
        debugger_resume_request: DebuggerResumeRequest,
    },
    BreakpointSet {
        debugger_breakpoint_set_request: DebuggerBreakpointSetRequest,
    },
    BreakpointRemove {
        debugger_breakpoint_remove_request: DebuggerBreakpointRemoveRequest,
    },
    BreakpointList {
        debugger_breakpoint_list_request: DebuggerBreakpointListRequest,
    },
    RegistersRead {
        debugger_registers_read_request: DebuggerRegistersReadRequest,
    },
    RegisterWrite {
        debugger_register_write_request: DebuggerRegisterWriteRequest,
    },
}
