use crate::constants::NATIVE_DEBUGGERS_PLUGIN_ID;
use squalr_engine_api::plugins::debugger::DebuggerTraceEventSink;
use squalr_engine_api::structures::debugger::{DebuggerBreakpointDescriptor, DebuggerBreakpointKind, DebuggerRegisterSnapshot};
use squalr_engine_api::{plugins::debugger::DebuggerPluginError, structures::processes::opened_process_info::OpenedProcessInfo};

pub(crate) struct NativeDebuggerBackend {
    _process_info: OpenedProcessInfo,
}

impl NativeDebuggerBackend {
    pub(crate) fn new(
        process_info: OpenedProcessInfo,
        _trace_event_sink: DebuggerTraceEventSink,
    ) -> Self {
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

    pub(crate) fn set_breakpoint_enabled(
        &self,
        _breakpoint_id: &str,
        _is_enabled: bool,
    ) -> Result<(), DebuggerPluginError> {
        Err(self.unavailable_error())
    }

    pub(crate) fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        Err(self.unavailable_error())
    }

    pub(crate) fn unavailable_error(&self) -> DebuggerPluginError {
        DebuggerPluginError::new(NATIVE_DEBUGGERS_PLUGIN_ID, "No native debugger backend is available for this platform yet.")
    }
}
