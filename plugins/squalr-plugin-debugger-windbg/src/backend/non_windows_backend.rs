use crate::constants::WINDBG_DEBUGGER_PLUGIN_ID;
use squalr_engine_api::structures::debugger::{DebuggerBreakpointDescriptor, DebuggerBreakpointKind, DebuggerRegisterSnapshot};
use squalr_engine_api::{plugins::debugger::DebuggerPluginError, structures::processes::opened_process_info::OpenedProcessInfo};

pub(crate) struct WindbgBackend {
    _process_info: OpenedProcessInfo,
}

impl WindbgBackend {
    pub(crate) fn new(process_info: OpenedProcessInfo) -> Self {
        Self { _process_info: process_info }
    }

    pub(crate) fn attach(&self) -> Result<(), DebuggerPluginError> {
        Err(self.unavailable_error())
    }

    pub(crate) fn detach(&self) -> Result<(), DebuggerPluginError> {
        Ok(())
    }

    pub(crate) fn pause(&self) -> Result<(), DebuggerPluginError> {
        Err(self.unavailable_error())
    }

    pub(crate) fn resume(&self) -> Result<(), DebuggerPluginError> {
        Err(self.unavailable_error())
    }

    pub(crate) fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        Err(self.unavailable_error())
    }

    pub(crate) fn write_register(
        &self,
        _register_name: &str,
        _value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        Err(self.unavailable_error())
    }

    pub(crate) fn set_breakpoint(
        &self,
        _address: u64,
        _kind: DebuggerBreakpointKind,
        _label: Option<String>,
    ) -> Result<DebuggerBreakpointDescriptor, DebuggerPluginError> {
        Err(self.unavailable_error())
    }

    pub(crate) fn remove_breakpoint(
        &self,
        _breakpoint_id: &str,
    ) -> Result<(), DebuggerPluginError> {
        Err(self.unavailable_error())
    }

    pub(crate) fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        Err(self.unavailable_error())
    }

    pub(crate) fn unavailable_error(&self) -> DebuggerPluginError {
        DebuggerPluginError::new(WINDBG_DEBUGGER_PLUGIN_ID, "WinDbg DbgEng debugger backend is only available on Windows.")
    }
}
