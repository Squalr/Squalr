use crate::{backend::DbgEngBackend, constants::DBGENG_DEBUGGER_PLUGIN_ID};
use squalr_engine_api::{
    plugins::debugger::{DebuggerPluginError, DebuggerSession, DebuggerTraceEventSink},
    structures::debugger::{DebuggerBreakpointDescriptor, DebuggerBreakpointKind, DebuggerRegisterSnapshot, DebuggerSessionState},
    structures::processes::opened_process_info::OpenedProcessInfo,
};

pub(crate) struct DbgEngDebuggerSession {
    backend: DbgEngBackend,
    session_state: DebuggerSessionState,
}

impl DbgEngDebuggerSession {
    pub(crate) fn new(
        process_info: OpenedProcessInfo,
        trace_event_sink: DebuggerTraceEventSink,
    ) -> Self {
        Self {
            backend: DbgEngBackend::new(process_info, trace_event_sink),
            session_state: DebuggerSessionState::Detached,
        }
    }
}

impl DebuggerSession for DbgEngDebuggerSession {
    fn plugin_id(&self) -> &str {
        DBGENG_DEBUGGER_PLUGIN_ID
    }

    fn get_state(&self) -> DebuggerSessionState {
        self.session_state
    }

    fn attach(&mut self) -> Result<DebuggerSessionState, DebuggerPluginError> {
        self.backend.attach()?;
        self.session_state = DebuggerSessionState::Attached;

        Ok(self.session_state)
    }

    fn detach(&mut self) -> Result<DebuggerSessionState, DebuggerPluginError> {
        self.backend.detach()?;
        self.session_state = DebuggerSessionState::Detached;

        Ok(self.session_state)
    }

    fn pause(&mut self) -> Result<DebuggerSessionState, DebuggerPluginError> {
        self.backend.pause()?;
        self.session_state = DebuggerSessionState::Paused;

        Ok(self.session_state)
    }

    fn resume(&mut self) -> Result<DebuggerSessionState, DebuggerPluginError> {
        self.backend.resume()?;
        self.session_state = DebuggerSessionState::Running;

        Ok(self.session_state)
    }

    fn set_breakpoint(
        &mut self,
        address: u64,
        kind: DebuggerBreakpointKind,
        label: Option<String>,
    ) -> Result<DebuggerBreakpointDescriptor, DebuggerPluginError> {
        self.backend.set_breakpoint(address, kind, label)
    }

    fn remove_breakpoint(
        &mut self,
        breakpoint_id: &str,
    ) -> Result<(), DebuggerPluginError> {
        self.backend.remove_breakpoint(breakpoint_id)
    }

    fn set_breakpoint_enabled(
        &mut self,
        breakpoint_id: &str,
        is_enabled: bool,
    ) -> Result<(), DebuggerPluginError> {
        self.backend.set_breakpoint_enabled(breakpoint_id, is_enabled)
    }

    fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        self.backend.list_breakpoints()
    }

    fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        self.backend.read_registers()
    }

    fn write_register(
        &mut self,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        self.backend.write_register(register_name, value)
    }
}
