use crate::plugins::debugger::debugger_error::DebuggerPluginError;
use crate::structures::debugger::{
    debugger_breakpoint_descriptor::DebuggerBreakpointDescriptor, debugger_breakpoint_kind::DebuggerBreakpointKind,
    debugger_register_snapshot::DebuggerRegisterSnapshot, debugger_session_state::DebuggerSessionState,
};

pub trait DebuggerSession: Send {
    fn plugin_id(&self) -> &str;

    fn get_state(&self) -> DebuggerSessionState;

    fn attach(&mut self) -> Result<DebuggerSessionState, DebuggerPluginError>;

    fn detach(&mut self) -> Result<DebuggerSessionState, DebuggerPluginError>;

    fn pause(&mut self) -> Result<DebuggerSessionState, DebuggerPluginError>;

    fn resume(&mut self) -> Result<DebuggerSessionState, DebuggerPluginError>;

    fn set_breakpoint(
        &mut self,
        address: u64,
        kind: DebuggerBreakpointKind,
        label: Option<String>,
    ) -> Result<DebuggerBreakpointDescriptor, DebuggerPluginError>;

    fn remove_breakpoint(
        &mut self,
        breakpoint_id: &str,
    ) -> Result<(), DebuggerPluginError>;

    fn set_breakpoint_enabled(
        &mut self,
        breakpoint_id: &str,
        is_enabled: bool,
    ) -> Result<(), DebuggerPluginError>;

    fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError>;

    fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError>;

    fn write_register(
        &mut self,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError>;
}
