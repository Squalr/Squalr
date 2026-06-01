use crate::command_executors::privileged_command_executor::PrivilegedCommandExecutor;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::debugger::attach::debugger_attach_request::DebuggerAttachRequest;
use squalr_engine_api::commands::debugger::attach::debugger_attach_response::DebuggerAttachResponse;
use squalr_engine_api::commands::debugger::breakpoint_list::debugger_breakpoint_list_request::DebuggerBreakpointListRequest;
use squalr_engine_api::commands::debugger::breakpoint_list::debugger_breakpoint_list_response::DebuggerBreakpointListResponse;
use squalr_engine_api::commands::debugger::breakpoint_remove::debugger_breakpoint_remove_request::DebuggerBreakpointRemoveRequest;
use squalr_engine_api::commands::debugger::breakpoint_remove::debugger_breakpoint_remove_response::DebuggerBreakpointRemoveResponse;
use squalr_engine_api::commands::debugger::breakpoint_set::debugger_breakpoint_set_request::DebuggerBreakpointSetRequest;
use squalr_engine_api::commands::debugger::breakpoint_set::debugger_breakpoint_set_response::DebuggerBreakpointSetResponse;
use squalr_engine_api::commands::debugger::debugger_command::DebuggerCommand;
use squalr_engine_api::commands::debugger::detach::debugger_detach_request::DebuggerDetachRequest;
use squalr_engine_api::commands::debugger::detach::debugger_detach_response::DebuggerDetachResponse;
use squalr_engine_api::commands::debugger::pause::debugger_pause_request::DebuggerPauseRequest;
use squalr_engine_api::commands::debugger::pause::debugger_pause_response::DebuggerPauseResponse;
use squalr_engine_api::commands::debugger::register_write::debugger_register_write_request::DebuggerRegisterWriteRequest;
use squalr_engine_api::commands::debugger::register_write::debugger_register_write_response::DebuggerRegisterWriteResponse;
use squalr_engine_api::commands::debugger::registers_read::debugger_registers_read_request::DebuggerRegistersReadRequest;
use squalr_engine_api::commands::debugger::registers_read::debugger_registers_read_response::DebuggerRegistersReadResponse;
use squalr_engine_api::commands::debugger::resume::debugger_resume_request::DebuggerResumeRequest;
use squalr_engine_api::commands::debugger::resume::debugger_resume_response::DebuggerResumeResponse;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::structures::debugger::{DebuggerCommandStatus, DebuggerSessionState};
use std::sync::Arc;

fn failure_status(error_message: impl Into<String>) -> DebuggerCommandStatus {
    DebuggerCommandStatus::failure(error_message)
}

fn no_opened_process_status() -> DebuggerCommandStatus {
    failure_status("No opened process to debug.")
}

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

impl PrivilegedCommandRequestExecutor for DebuggerAttachRequest {
    type ResponseType = DebuggerAttachResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return DebuggerAttachResponse {
                status: no_opened_process_status(),
                session_state: DebuggerSessionState::Detached,
                active_plugin_id: None,
            };
        };

        match engine_privileged_state
            .get_debugger_service()
            .attach(&opened_process_info, self.plugin_id.as_deref())
        {
            Ok(operation_status) => DebuggerAttachResponse {
                status: DebuggerCommandStatus::success(),
                session_state: operation_status.get_session_state(),
                active_plugin_id: operation_status.get_active_plugin_id().map(ToString::to_string),
            },
            Err(error_message) => DebuggerAttachResponse {
                status: failure_status(error_message),
                session_state: DebuggerSessionState::Detached,
                active_plugin_id: None,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerDetachRequest {
    type ResponseType = DebuggerDetachResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        match engine_privileged_state.get_debugger_service().detach() {
            Ok(operation_status) => DebuggerDetachResponse {
                status: DebuggerCommandStatus::success(),
                session_state: operation_status.get_session_state(),
            },
            Err(error_message) => DebuggerDetachResponse {
                status: failure_status(error_message),
                session_state: DebuggerSessionState::Detached,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerPauseRequest {
    type ResponseType = DebuggerPauseResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        match engine_privileged_state.get_debugger_service().pause() {
            Ok(operation_status) => DebuggerPauseResponse {
                status: DebuggerCommandStatus::success(),
                session_state: operation_status.get_session_state(),
            },
            Err(error_message) => DebuggerPauseResponse {
                status: failure_status(error_message),
                session_state: DebuggerSessionState::Detached,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerResumeRequest {
    type ResponseType = DebuggerResumeResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        match engine_privileged_state.get_debugger_service().resume() {
            Ok(operation_status) => DebuggerResumeResponse {
                status: DebuggerCommandStatus::success(),
                session_state: operation_status.get_session_state(),
            },
            Err(error_message) => DebuggerResumeResponse {
                status: failure_status(error_message),
                session_state: DebuggerSessionState::Detached,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerBreakpointSetRequest {
    type ResponseType = DebuggerBreakpointSetResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        match engine_privileged_state
            .get_debugger_service()
            .set_breakpoint(self.address, self.kind.clone(), self.label.clone())
        {
            Ok(breakpoint) => DebuggerBreakpointSetResponse {
                status: DebuggerCommandStatus::success(),
                breakpoint: Some(breakpoint),
            },
            Err(error_message) => DebuggerBreakpointSetResponse {
                status: failure_status(error_message),
                breakpoint: None,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerBreakpointRemoveRequest {
    type ResponseType = DebuggerBreakpointRemoveResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        match engine_privileged_state
            .get_debugger_service()
            .remove_breakpoint(&self.breakpoint_id)
        {
            Ok(()) => DebuggerBreakpointRemoveResponse {
                status: DebuggerCommandStatus::success(),
            },
            Err(error_message) => DebuggerBreakpointRemoveResponse {
                status: failure_status(error_message),
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerBreakpointListRequest {
    type ResponseType = DebuggerBreakpointListResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        match engine_privileged_state
            .get_debugger_service()
            .list_breakpoints()
        {
            Ok(breakpoints) => DebuggerBreakpointListResponse {
                status: DebuggerCommandStatus::success(),
                breakpoints,
            },
            Err(error_message) => DebuggerBreakpointListResponse {
                status: failure_status(error_message),
                breakpoints: Vec::new(),
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerRegistersReadRequest {
    type ResponseType = DebuggerRegistersReadResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        match engine_privileged_state.get_debugger_service().read_registers() {
            Ok(register_snapshot) => DebuggerRegistersReadResponse {
                status: DebuggerCommandStatus::success(),
                register_snapshot: Some(register_snapshot),
            },
            Err(error_message) => DebuggerRegistersReadResponse {
                status: failure_status(error_message),
                register_snapshot: None,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for DebuggerRegisterWriteRequest {
    type ResponseType = DebuggerRegisterWriteResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        match engine_privileged_state
            .get_debugger_service()
            .write_register(&self.register_name, self.value)
        {
            Ok(register_snapshot) => DebuggerRegisterWriteResponse {
                status: DebuggerCommandStatus::success(),
                register_snapshot: Some(register_snapshot),
            },
            Err(error_message) => DebuggerRegisterWriteResponse {
                status: failure_status(error_message),
                register_snapshot: None,
            },
        }
    }
}
