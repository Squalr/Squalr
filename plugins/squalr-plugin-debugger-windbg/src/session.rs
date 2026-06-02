use crate::{backend::WindbgBackend, constants::WINDBG_DEBUGGER_PLUGIN_ID};
use squalr_engine_api::{
    plugins::debugger::{DebuggerPluginError, DebuggerSession},
    structures::debugger::{DebuggerBreakpointDescriptor, DebuggerBreakpointKind, DebuggerRegisterSnapshot, DebuggerSessionState},
    structures::processes::opened_process_info::OpenedProcessInfo,
};

pub(crate) struct WindbgDebuggerSession {
    backend: WindbgBackend,
    session_state: DebuggerSessionState,
}

impl WindbgDebuggerSession {
    pub(crate) fn new(process_info: OpenedProcessInfo) -> Self {
        Self {
            backend: WindbgBackend::new(process_info),
            session_state: DebuggerSessionState::Detached,
        }
    }

    fn unavailable(&self) -> DebuggerPluginError {
        self.backend.unavailable_error()
    }
}

impl DebuggerSession for WindbgDebuggerSession {
    fn plugin_id(&self) -> &str {
        WINDBG_DEBUGGER_PLUGIN_ID
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
        _address: u64,
        _kind: DebuggerBreakpointKind,
        _label: Option<String>,
    ) -> Result<DebuggerBreakpointDescriptor, DebuggerPluginError> {
        Err(self.unavailable())
    }

    fn remove_breakpoint(
        &mut self,
        _breakpoint_id: &str,
    ) -> Result<(), DebuggerPluginError> {
        Err(self.unavailable())
    }

    fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        Err(self.unavailable())
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
