use crate::command_executors::privileged_command_executor::PrivilegedCommandExecutor;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::debugger::{
    attach::{debugger_attach_request::DebuggerAttachRequest, debugger_attach_response::DebuggerAttachResponse},
    breakpoint_list::{debugger_breakpoint_list_request::DebuggerBreakpointListRequest, debugger_breakpoint_list_response::DebuggerBreakpointListResponse},
    breakpoint_remove::{
        debugger_breakpoint_remove_request::DebuggerBreakpointRemoveRequest, debugger_breakpoint_remove_response::DebuggerBreakpointRemoveResponse,
    },
    breakpoint_set::{debugger_breakpoint_set_request::DebuggerBreakpointSetRequest, debugger_breakpoint_set_response::DebuggerBreakpointSetResponse},
    debugger_command::DebuggerCommand,
    detach::{debugger_detach_request::DebuggerDetachRequest, debugger_detach_response::DebuggerDetachResponse},
    pause::{debugger_pause_request::DebuggerPauseRequest, debugger_pause_response::DebuggerPauseResponse},
    register_write::{debugger_register_write_request::DebuggerRegisterWriteRequest, debugger_register_write_response::DebuggerRegisterWriteResponse},
    registers_read::{debugger_registers_read_request::DebuggerRegistersReadRequest, debugger_registers_read_response::DebuggerRegistersReadResponse},
    resume::{debugger_resume_request::DebuggerResumeRequest, debugger_resume_response::DebuggerResumeResponse},
};
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::structures::debugger::{DebuggerCommandStatus, DebuggerSessionState};
use std::sync::Arc;

const DEBUGGER_SERVICE_UNAVAILABLE: &str = "Debugger service is not wired yet.";

impl PrivilegedCommandExecutor for DebuggerCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
        match self {
            DebuggerCommand::Attach { debugger_attach_request } => debugger_attach_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            DebuggerCommand::Detach { debugger_detach_request } => debugger_detach_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            DebuggerCommand::Pause { debugger_pause_request } => debugger_pause_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            DebuggerCommand::Resume { debugger_resume_request } => debugger_resume_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            DebuggerCommand::BreakpointSet {
                debugger_breakpoint_set_request,
            } => debugger_breakpoint_set_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            DebuggerCommand::BreakpointRemove {
                debugger_breakpoint_remove_request,
            } => debugger_breakpoint_remove_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            DebuggerCommand::BreakpointList {
                debugger_breakpoint_list_request,
            } => debugger_breakpoint_list_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            DebuggerCommand::RegistersRead {
                debugger_registers_read_request,
            } => debugger_registers_read_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            DebuggerCommand::RegisterWrite {
                debugger_register_write_request,
            } => debugger_register_write_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}

fn unavailable_status() -> DebuggerCommandStatus {
    DebuggerCommandStatus::failure(DEBUGGER_SERVICE_UNAVAILABLE)
}

impl PrivilegedCommandRequestExecutor for DebuggerAttachRequest {
    type ResponseType = DebuggerAttachResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        DebuggerAttachResponse {
            status: unavailable_status(),
            session_state: DebuggerSessionState::Detached,
            active_plugin_id: None,
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerDetachRequest {
    type ResponseType = DebuggerDetachResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        DebuggerDetachResponse {
            status: unavailable_status(),
            session_state: DebuggerSessionState::Detached,
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerPauseRequest {
    type ResponseType = DebuggerPauseResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        DebuggerPauseResponse {
            status: unavailable_status(),
            session_state: DebuggerSessionState::Detached,
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerResumeRequest {
    type ResponseType = DebuggerResumeResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        DebuggerResumeResponse {
            status: unavailable_status(),
            session_state: DebuggerSessionState::Detached,
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerBreakpointSetRequest {
    type ResponseType = DebuggerBreakpointSetResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        DebuggerBreakpointSetResponse {
            status: unavailable_status(),
            breakpoint: None,
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerBreakpointRemoveRequest {
    type ResponseType = DebuggerBreakpointRemoveResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        DebuggerBreakpointRemoveResponse { status: unavailable_status() }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerBreakpointListRequest {
    type ResponseType = DebuggerBreakpointListResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        DebuggerBreakpointListResponse {
            status: unavailable_status(),
            breakpoints: Vec::new(),
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerRegistersReadRequest {
    type ResponseType = DebuggerRegistersReadResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        DebuggerRegistersReadResponse {
            status: unavailable_status(),
            register_snapshot: None,
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerRegisterWriteRequest {
    type ResponseType = DebuggerRegisterWriteResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        DebuggerRegisterWriteResponse {
            status: unavailable_status(),
            register_snapshot: None,
        }
    }
}
